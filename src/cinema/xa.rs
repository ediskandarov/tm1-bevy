//! CD-XA stereo audio from the supported disc's raw STR sectors.
use anyhow::{Result, ensure};
use std::io::Read;

pub(super) const RATE: u32 = 37_800;

pub(super) fn decode(mut reader: impl Read, sectors: u64) -> Result<Vec<u8>> {
    let mut pcm = Vec::new();
    let mut sector = [0; 2352];
    let mut history = [[0i32; 2]; 2];
    let mut stream = None;
    for _ in 0..sectors {
        reader.read_exact(&mut sector)?;
        if sector[18] & 4 == 0 {
            continue;
        }
        ensure!(
            sector[15] == 2 && sector[16..20] == sector[20..24],
            "invalid XA sector"
        );
        let id = (sector[16], sector[17]);
        if *stream.get_or_insert(id) != id {
            continue;
        }
        ensure!(sector[19] == 1, "unsupported STR audio coding");
        for group in sector[24..2328].chunks_exact(128) {
            for pair in 0..4 {
                let mut params = [(0, 0, 0); 2];
                for (ch, param) in params.iter_mut().enumerate() {
                    let header = group[4 + 2 * pair + ch];
                    let shift = header & 15;
                    let filter = usize::from(header >> 4);
                    ensure!(shift <= 12 && filter < 4, "invalid XA sound unit");
                    let (a, b) = [(0, 0), (60, 0), (115, -52), (98, -55)][filter];
                    *param = (shift, a, b);
                }
                for row in group[16..].chunks_exact(4) {
                    for ch in 0..2 {
                        let (shift, a, b) = params[ch];
                        let nibble = (row[pair] >> (4 * ch)) & 15;
                        let residual = i32::from((i16::from(nibble) << 12) >> shift);
                        let [prev, older] = history[ch];
                        let sample =
                            (residual + ((prev * a + older * b + 32) >> 6)).clamp(-32768, 32767);
                        history[ch] = [sample, prev];
                        pcm.extend_from_slice(&(sample as i16).to_le_bytes());
                    }
                }
            }
        }
    }
    ensure!(!pcm.is_empty(), "STR has no XA audio");
    Ok(pcm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom};
    use tm1_assets::disc::Disc;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn jp_movie_pcm_and_demo_interleave() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut track = std::fs::File::open(&disc.tracks[0].path).unwrap();
        // FNV-1a of FFmpeg's native-rate stereo s16le output for each whole stream.
        for (name, samples, checksum) in [
            ("MOVIES/SLOGO15S.STR", 376992, 0xa6ed2ce505b13041),
            ("UAMOVIES/TMTTL15S.STR", 147168, 0x156cf7447c4c235b),
            ("UAMOVIES/TMDMO15S.STR", 2522016, 0x1c974dd56c66d5e5),
        ] {
            let entry = &disc.files[name];
            let sectors = u64::from(entry.size.div_ceil(2048));
            track
                .seek(SeekFrom::Start(u64::from(entry.sector) * 2352))
                .unwrap();
            let mut raw = vec![0; sectors as usize * 2352];
            track.read_exact(&mut raw).unwrap();
            let pcm = decode(raw.as_slice(), sectors).unwrap();
            assert_eq!(pcm.len(), samples * 4);
            assert_eq!(
                pcm.iter()
                    .fold(0xcbf29ce484222325u64, |h, b| (h ^ u64::from(*b))
                        .wrapping_mul(0x100000001b3)),
                checksum
            );
            if name.ends_with("TMDMO15S.STR") {
                let mut audio_frames = 0;
                let mut video_frames = 0;
                for sector in raw.chunks_exact(2352) {
                    if sector[18] & 4 != 0 {
                        audio_frames += 2016;
                    } else if sector[24..30] == [0x60, 1, 1, 0x80, 0, 0] {
                        let expected = video_frames * RATE / 15;
                        assert!(expected.abs_diff(audio_frames) < 2016);
                        video_frames += 1;
                    }
                }
                assert_eq!(video_frames, 1000);
                assert_eq!(audio_frames, video_frames * RATE / 15 + 2016);
            }
        }
    }
}
