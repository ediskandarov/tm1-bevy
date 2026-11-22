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
    Ok(n[0] * 4500 + n[1] * 75 + n[2])
}
fn parse_cue(cue: &Path, text: &str) -> Result<Vec<Track>> {
    let parent = cue.parent().context("CUE needs a parent directory")?;
    let mut tracks = Vec::new();
    let mut file = None;
    let mut current: Option<Track> = None;
    let mut index_seen = false;
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
        } else if line.starts_with("INDEX ") {
            let fields: Vec<_> = line.split_whitespace().collect();
            ensure!(fields.len() == 3, "invalid INDEX");
            let t = current.as_mut().context("INDEX without TRACK")?;
            let time = timestamp(fields[2])?;
            match fields[1] {
                "00" => ensure!(t.number > 1 && time == 0, "unsupported pregap layout"),
                "01" => {
                    ensure!(!index_seen, "duplicate INDEX 01");
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
    ensure!(
        tracks.len() == 9,
        "expected nine tracks for supported JP revision"
    );
    for t in &tracks {
        ensure!(
            t.index01 == if t.number == 1 { 0 } else { 150 },
            "unsupported INDEX 01 for track {}",
            t.number
        );
    }
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
        ensure!(expected.len() == tracks.len(), "invalid built-in manifest");
        for (t, row) in tracks.iter_mut().zip(expected) {
            let fields: Vec<_> = row.split_whitespace().collect();
            let mut file = File::open(&t.path)
                .with_context(|| format!("open track {} at {}", t.number, t.path.display()))?;
            t.size = file.metadata()?.len();
            ensure!(
                t.size % SECTOR == 0 && t.size == fields[1].parse::<u64>()?,
                "track {}: unsupported size",
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
            ensure!(
                t.sha256 == fields[2],
                "track {}: unsupported revision or corrupt image",
                t.number
            );
        }
        let mut disc = Self {
            tracks,
            files: BTreeMap::new(),
        };
        let mut f = File::open(&disc.tracks[0].path)?;
        let pvd = read_sectors(&mut f, 16, 2048)?;
        ensure!(
            bytes(&pvd, 0, 7)? == b"\x01CD001\x01",
            "missing ISO9660 primary volume descriptor"
        );
        disc.directory(
            &mut f,
            u32le(&pvd, 158)?,
            u32le(&pvd, 166)?,
            "",
            &mut Vec::new(),
        )?;
        let executable = disc.read("SIPS_600.07")?;
        ensure!(
            format!("{:x}", Sha256::digest(&executable))
                == "552dbdb7afbcdaced9be90074036d59a2c3eb6fe31ade623679842346b3ffa36",
            "executable does not match baseline"
        );
        Ok(disc)
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
            let block = u32le(r, 2)?;
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
    fn reject_cue_traversal_and_missing_tracks() {
        assert!(parse_cue(Path::new("/disc/test.cue"), "FILE \"../secret.bin\" BINARY").is_err());
        assert!(
            parse_cue(
                Path::new("/disc/test.cue"),
                "FILE \"track.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00"
            )
            .is_err()
        );
    }
}
