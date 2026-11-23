use crate::{bytes, u32le};
use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
};

const SECTOR: u64 = 2352;
const MANIFEST: &str = include_str!("../../../data/jp-disc.tsv");

#[derive(Debug)]
pub struct Track {
    pub number: usize,
    pub path: PathBuf,
    pub mode: String,
    pub index01: u64,
    pub size: u64,
    pub sha256: String,
}
#[derive(Debug, Clone)]
pub struct Entry {
    pub sector: u32,
    pub size: u32,
}
#[derive(Debug)]
pub struct Disc {
    pub tracks: Vec<Track>,
    pub files: BTreeMap<String, Entry>,
}

fn timestamp(value: &str) -> Result<u64> {
    let n: Vec<u64> = value.split(':').map(str::parse).collect::<Result<_, _>>()?;
    ensure!(
        n.len() == 3 && n[1] < 60 && n[2] < 75,
        "invalid CUE timestamp {value}"
    );
    n[0].checked_mul(4500)
        .and_then(|v| v.checked_add(n[1] * 75 + n[2]))
        .context("CUE timestamp too large")
}
fn parse_cue(cue: &Path, text: &str) -> Result<Vec<Track>> {
    let parent = cue.parent().context("CUE needs a parent directory")?;
    let mut tracks = Vec::new();
    let mut file = None;
    let mut current: Option<Track> = None;
    let mut index_seen = false;
    let mut pregap = None;
    for line in text.lines().map(str::trim).filter(|s| !s.is_empty()) {
        if line.starts_with("REM ") {
            continue;
        }
        if line.starts_with("FILE ") {
            if let Some(t) = current.take() {
                ensure!(index_seen, "missing INDEX 01");
                tracks.push(t);
            }
            ensure!(file.is_none(), "FILE without TRACK");
            let quoted = line
                .strip_prefix("FILE \"")
                .context("expected quoted CUE filename")?;
            let (name, kind) = quoted
                .rsplit_once('"')
                .context("unterminated CUE filename")?;
            ensure!(kind.trim() == "BINARY", "only BINARY CUE files supported");
            let p = Path::new(name);
            ensure!(
                !p.is_absolute() && p.components().all(|c| matches!(c, Component::Normal(_))),
                "CUE filename must stay inside its directory"
            );
            file = Some(parent.join(p));
        } else if line.starts_with("TRACK ") {
            ensure!(
                current.is_none(),
                "only one track per BIN is supported for this revision"
            );
            let fields: Vec<_> = line.split_whitespace().collect();
            ensure!(fields.len() == 3, "invalid TRACK");
            let number = fields[1].parse::<usize>()?;
            ensure!(number == tracks.len() + 1, "tracks must be sequential");
            let expected = if number == 1 { "MODE2/2352" } else { "AUDIO" };
            ensure!(fields[2] == expected, "track {number}: expected {expected}");
            current = Some(Track {
                number,
                path: file.take().context("TRACK without FILE")?,
                mode: fields[2].into(),
                index01: 0,
                size: 0,
                sha256: String::new(),
            });
            index_seen = false;
            pregap = None;
        } else if line.starts_with("INDEX ") {
            let fields: Vec<_> = line.split_whitespace().collect();
            ensure!(fields.len() == 3, "invalid INDEX");
            let t = current.as_mut().context("INDEX without TRACK")?;
            let time = timestamp(fields[2])?;
            match fields[1] {
                "00" => {
                    ensure!(
                        !index_seen && pregap.replace(time).is_none(),
                        "invalid INDEX 00"
                    );
                }
                "01" => {
                    ensure!(!index_seen, "duplicate INDEX 01");
                    ensure!(
                        pregap.is_none_or(|start| start <= time),
                        "INDEX 00 follows INDEX 01"
                    );
                    t.index01 = time;
                    index_seen = true;
                }
                _ => bail!("unsupported CUE index"),
            }
        } else {
            bail!("unsupported CUE directive: {line}");
        }
    }
    if let Some(t) = current {
        ensure!(index_seen, "missing INDEX 01");
        tracks.push(t);
    }
    ensure!(file.is_none(), "FILE without TRACK");
    ensure!(!tracks.is_empty(), "CUE has no data track");
    Ok(tracks)
}

impl Disc {
    pub fn open(cue: impl AsRef<Path>) -> Result<Self> {
        let cue = cue.as_ref();
        let mut tracks = parse_cue(
            cue,
            &std::fs::read_to_string(cue).with_context(|| format!("read {}", cue.display()))?,
        )?;
        let expected: Vec<_> = MANIFEST
            .lines()
            .filter(|s| !s.starts_with('#') && !s.is_empty())
            .collect();
        if tracks.len() != expected.len() {
            eprintln!(
                "warning: CUE lists {} tracks; JP reference has {}",
                tracks.len(),
                expected.len()
            );
        }
        for t in &mut tracks {
            let mut file = File::open(&t.path)
                .with_context(|| format!("open track {} at {}", t.number, t.path.display()))?;
            t.size = file.metadata()?.len();
            ensure!(
                t.size % SECTOR == 0,
                "track {}: incomplete 2352-byte sector",
                t.number
            );
            ensure!(
                t.index01 < t.size / SECTOR,
                "track {}: INDEX 01 outside BIN",
                t.number
            );
            let mut hash = Sha256::new();
            let mut buf = [0; 65536];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hash.update(&buf[..n]);
            }
            t.sha256 = format!("{:x}", hash.finalize());
            if let Some(row) = expected.get(t.number - 1) {
                let fields: Vec<_> = row.split_whitespace().collect();
                if t.size != fields[1].parse::<u64>()? || t.sha256 != fields[2] {
                    eprintln!(
                        "warning: track {} differs from the JP reference ({} bytes, SHA-256 {}); using CUE track",
                        t.number, t.size, t.sha256
                    );
                }
            }
        }
        let mut disc = Self {
            tracks,
            files: BTreeMap::new(),
        };
        let mut f = File::open(&disc.tracks[0].path)?;
        let pvd = read_sectors(&mut f, disc.data_sector(16)?, 2048)?;
        ensure!(
            bytes(&pvd, 0, 7)? == b"\x01CD001\x01",
            "missing ISO9660 primary volume descriptor"
        );
        disc.directory(
            &mut f,
            disc.data_sector(u32le(&pvd, 158)?)?,
            u32le(&pvd, 166)?,
            "",
            &mut Vec::new(),
        )?;
        if disc.files.contains_key("SIPS_600.07") {
            let executable = disc.read("SIPS_600.07")?;
            if format!("{:x}", Sha256::digest(&executable))
                != "552dbdb7afbcdaced9be90074036d59a2c3eb6fe31ade623679842346b3ffa36"
            {
                eprintln!("warning: SIPS_600.07 differs from the JP reference");
            }
        } else {
            eprintln!("warning: JP reference executable SIPS_600.07 is absent");
        }
        Ok(disc)
    }
    // Keep file entry sectors relative to the BIN for ISO reads and raw STR streaming.
    fn data_sector(&self, iso_sector: u32) -> Result<u32> {
        u32::try_from(self.tracks[0].index01)?
            .checked_add(iso_sector)
            .context("ISO sector overflow")
    }
    fn directory(
        &mut self,
        f: &mut File,
        sector: u32,
        size: u32,
        prefix: &str,
        ancestors: &mut Vec<u32>,
    ) -> Result<()> {
        ensure!(
            ancestors.len() < 16 && !ancestors.contains(&sector),
            "cyclic/deep ISO directory"
        );
        ensure!(size <= 4 * 1024 * 1024, "oversized ISO directory");
        ancestors.push(sector);
        let d = read_sectors(f, sector, size)?;
        let mut offset = 0;
        while offset < d.len() {
            let len = d[offset] as usize;
            if len == 0 {
                offset = (offset / 2048 + 1) * 2048;
                continue;
            }
            let r = bytes(&d, offset, len)?;
            ensure!(len >= 34, "short directory record");
            let name = bytes(r, 33, r[32] as usize)?;
            offset += len;
            if name == [0] || name == [1] {
                continue;
            }
            let name = std::str::from_utf8(name)?
                .split(';')
                .next()
                .context("missing ISO name")?;
            ensure!(
                !name.contains('/') && !name.contains('\\'),
                "invalid ISO filename"
            );
            let path = format!("{prefix}{name}");
            let block = self.data_sector(u32le(r, 2)?)?;
            let size = u32le(r, 10)?;
            if r[25] & 2 != 0 {
                self.directory(f, block, size, &format!("{path}/"), ancestors)?;
            } else {
                ensure!(
                    self.files
                        .insert(
                            path,
                            Entry {
                                sector: block,
                                size
                            }
                        )
                        .is_none(),
                    "duplicate ISO path"
                );
            }
        }
        ancestors.pop();
        Ok(())
    }
    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let key = name
            .replace('\\', "/")
            .trim_start_matches('/')
            .split(';')
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        let entry = self
            .files
            .get(&key)
            .with_context(|| format!("disc file not found: {key}"))?;
        read_sectors(
            &mut File::open(&self.tracks[0].path)?,
            entry.sector,
            entry.size,
        )
    }
    /// Signed little-endian stereo PCM at 44,100 Hz, starting after INDEX 00.
    pub fn audio_track(&self, number: usize) -> Result<Vec<u8>> {
        let track = self
            .tracks
            .get(number.checked_sub(1).context("track numbers start at 1")?)
            .context("track not found")?;
        ensure!(track.mode == "AUDIO", "not an audio track");
        let mut file = File::open(&track.path)?;
        file.seek(SeekFrom::Start(track.index01 * SECTOR))?;
        let mut pcm = Vec::new();
        file.read_to_end(&mut pcm)?;
        Ok(pcm)
    }
}
fn read_sectors(file: &mut File, first: u32, size: u32) -> Result<Vec<u8>> {
    let sectors = u64::from(size).div_ceil(2048);
    ensure!(
        (u64::from(first) + sectors) * SECTOR <= file.metadata()?.len(),
        "ISO extent outside track"
    );
    let mut output = Vec::with_capacity(size as usize);
    for sector in u64::from(first)..u64::from(first) + sectors {
        file.seek(SeekFrom::Start(sector * SECTOR))?;
        let mut raw = [0; 2352];
        file.read_exact(&mut raw)?;
        ensure!(
            raw[15] == 2 && raw[18] & 0x20 == 0 && raw[16..20] == raw[20..24],
            "expected Mode 2 Form 1 sector"
        );
        let remaining = size as usize - output.len();
        output.extend_from_slice(&raw[24..24 + remaining.min(2048)]);
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cue_time_validation() {
        assert_eq!(timestamp("01:02:03").unwrap(), 4653);
        assert!(timestamp("00:60:00").is_err());
        assert!(timestamp("00:00:75").is_err());
        assert!(timestamp("1:2").is_err());
    }
    #[test]
    fn modified_disc_uses_cue_indexes_and_pcm() {
        let dir = std::env::temp_dir().join(format!("tm1-cue-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(dir.clone());
        // One leading sector before track 1 INDEX 01. ISO extents start there.
        let mut data = vec![0; 20 * SECTOR as usize];
        let mut sector = |n: usize| {
            let start = (1 + n) * SECTOR as usize;
            data[start + 15] = 2;
            start + 24
        };
        let pvd = sector(16);
        let root = sector(17);
        let exe = sector(18);
        data[pvd..pvd + 7].copy_from_slice(b"\x01CD001\x01");
        data[pvd + 158..pvd + 162].copy_from_slice(&17u32.to_le_bytes());
        data[pvd + 166..pvd + 170].copy_from_slice(&2048u32.to_le_bytes());
        data[root] = 46;
        data[root + 2..root + 6].copy_from_slice(&18u32.to_le_bytes());
        data[root + 10..root + 14].copy_from_slice(&3u32.to_le_bytes());
        let name = b"SIPS_600.07;1";
        data[root + 32] = name.len() as u8;
        data[root + 33..root + 33 + name.len()].copy_from_slice(name);
        data[exe..exe + 3].copy_from_slice(b"alt");
        std::fs::write(dir.join("data.bin"), data).unwrap();
        let mut music = vec![0; SECTOR as usize];
        music.extend(vec![42; SECTOR as usize]);
        std::fs::write(dir.join("music.bin"), &music).unwrap();
        let cue = dir.join("disc.cue");
        let layout = "FILE \"data.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:01\nFILE \"music.bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:01";
        std::fs::write(&cue, layout).unwrap();
        let disc = Disc::open(&cue).unwrap();
        assert_eq!(disc.read("SIPS_600.07").unwrap(), b"alt");
        assert_eq!(disc.audio_track(2).unwrap(), music[SECTOR as usize..]);
        std::fs::write(
            &cue,
            layout.replace(
                "TRACK 02 AUDIO\nINDEX 01 00:00:01",
                "TRACK 02 AUDIO\nINDEX 01 00:00:02",
            ),
        )
        .unwrap();
        assert!(
            Disc::open(&cue)
                .unwrap_err()
                .to_string()
                .contains("INDEX 01 outside BIN")
        );
    }

    #[test]
    fn reject_cue_traversal_and_missing_index() {
        assert!(parse_cue(Path::new("/disc/test.cue"), "FILE \"../secret.bin\" BINARY").is_err());
        assert!(
            parse_cue(
                Path::new("/disc/test.cue"),
                "FILE \"track.bin\" BINARY\nTRACK 01 MODE2/2352"
            )
            .is_err()
        );
    }
}
