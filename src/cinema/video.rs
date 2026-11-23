//! STR frame assembly and host-side MDEC reconstruction for the JP disc.
mod bitstream;
mod stream;
mod tables;

use anyhow::{Context, Result, ensure};
use std::{
    fs::File,
    io::{BufReader, Read, Take},
};
use tm1_assets::screen::{HEIGHT, WIDTH};

pub(super) struct Decoder {
    track: Take<BufReader<File>>,
    assembler: stream::FrameAssembler,
    compressed: Vec<u8>,
    words: Vec<u16>,
}

impl Decoder {
    pub(super) fn new(track: Take<BufReader<File>>) -> Self {
        Self {
            track,
            assembler: stream::FrameAssembler::new(),
            compressed: vec![0; usize::from(stream::MAX_CHUNKS) * stream::CHUNK_PAYLOAD_BYTES],
            words: vec![0; 2 * 65536],
        }
    }

    pub(super) fn read_frame(&mut self, pixels: &mut [u8]) -> Result<()> {
        let mut sector = [0; 2352];
        loop {
            self.track
                .read_exact(&mut sector)
                .context("incomplete STR video frame")?;
            if sector[18] & 2 == 0 {
                continue;
            }
            let frame = self.assembler.add(&sector[24..2072], &mut self.compressed);
            ensure!(self.assembler.dropped == 0, "missing STR video chunk");
            let Some(frame) = frame else { continue };
            ensure!(
                usize::from(frame.width) == WIDTH && usize::from(frame.height) == HEIGHT,
                "unexpected STR frame dimensions"
            );
            let bytes = self
                .compressed
                .get(..frame.size as usize)
                .context("invalid STR frame size")?;
            let len = bitstream::decode_frame(
                bytes,
                &mut self.words,
                (WIDTH * HEIGHT / 256) as u32,
                (HEIGHT / 16) as u32,
                &mut || {},
            )
            .map_err(|e| anyhow::anyhow!("STR frame {}: {e:?}", frame.number))?;
            return render(&self.words[..2 * len], pixels);
        }
    }
}

fn signed_level(word: u16) -> i32 {
    i32::from((word << 6) as i16 >> 6)
}

fn decode_block(words: &mut std::slice::Iter<'_, u16>) -> Result<[u8; 64]> {
    let dc = *words.next().context("missing MDEC block")?;
    ensure!(dc != bitstream::END_OF_BLOCK, "empty MDEC block");
    let scale = i32::from(dc >> 10);
    let mut coefficients = [0i32; 64];
    coefficients[0] = 2 * signed_level(dc);
    let mut index = 0;
    loop {
        let ac = *words.next().context("unterminated MDEC block")?;
        if ac == bitstream::END_OF_BLOCK {
            break;
        }
        index += 1 + usize::from(ac >> 10);
        let pos = usize::from(
            *tables::ZIGZAG_TO_ROW_MAJOR
                .get(index)
                .context("invalid MDEC coefficient run")?,
        );
        coefficients[pos] = signed_level(ac) * scale * i32::from(tables::QUANT_ROW_MAJOR[pos]) / 8;
    }
    let mut rows = [[0i64; 8]; 8];
    for (row, input) in rows.iter_mut().zip(coefficients.chunks_exact(8)) {
        for (&value, basis) in input.iter().zip(tables::BASIS.chunks_exact(8)) {
            if value != 0 {
                for (dest, &weight) in row.iter_mut().zip(basis) {
                    *dest += i64::from(value) * i64::from(weight);
                }
            }
        }
    }
    let mut block = [0; 64];
    for (y, line) in block.chunks_exact_mut(8).enumerate() {
        for (x, sample) in line.iter_mut().enumerate() {
            let sum: i64 = rows
                .iter()
                .enumerate()
                .map(|(v, row)| row[x] * i64::from(tables::BASIS[8 * v + y]))
                .sum();
            *sample = (128 + ((sum + (1 << 31)) >> 32)).clamp(0, 255) as u8;
        }
    }
    Ok(block)
}

fn render(data: &[u16], pixels: &mut [u8]) -> Result<()> {
    ensure!(
        pixels.len() == WIDTH * HEIGHT * 4,
        "invalid movie image size"
    );
    let mut words = data.iter();
    for left in (0..WIDTH).step_by(16) {
        for top in (0..HEIGHT).step_by(16) {
            let cr = decode_block(&mut words)?;
            let cb = decode_block(&mut words)?;
            for quadrant in 0..4 {
                let luma = decode_block(&mut words)?;
                let dx = (quadrant % 2) * 8;
                let dy = (quadrant / 2) * 8;
                for (i, &y) in luma.iter().enumerate() {
                    let x = dx + i % 8;
                    let row = dy + i / 8;
                    let chroma = row / 2 * 8 + x / 2;
                    let u = i32::from(cb[chroma]) - 128;
                    let v = i32::from(cr[chroma]) - 128;
                    let y = i32::from(y);
                    let rgb = [
                        y + ((91881 * v + 32768) >> 16),
                        y - ((22554 * u + 46802 * v + 32768) >> 16),
                        y + ((116130 * u + 32768) >> 16),
                    ];
                    let at = 4 * ((top + row) * WIDTH + left + x);
                    let [r, g, b] = rgb.map(|n| n.clamp(0, 255) as u8);
                    pixels[at..at + 4].copy_from_slice(&[r, g, b, 255]);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom};
    use tm1_assets::disc::Disc;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn complete_jp_streams_and_reference_colors() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        // RGB averages of eight 80x120 regions in frame 30 from the reference decoder.
        for (name, frames, reference) in [
            (
                "MOVIES/SLOGO15S.STR",
                149,
                [
                    [41, 29, 20],
                    [41, 29, 20],
                    [43, 31, 22],
                    [41, 28, 20],
                    [37, 25, 17],
                    [42, 29, 21],
                    [41, 28, 19],
                    [37, 24, 16],
                ],
            ),
            (
                "UAMOVIES/TMTTL15S.STR",
                67,
                [
                    [45, 41, 40],
                    [80, 73, 85],
                    [79, 71, 85],
                    [111, 53, 55],
                    [116, 123, 39],
                    [186, 144, 70],
                    [190, 155, 117],
                    [92, 98, 135],
                ],
            ),
            (
                "UAMOVIES/TMDMO15S.STR",
                1000,
                [
                    [55, 51, 54],
                    [82, 75, 87],
                    [80, 73, 88],
                    [118, 59, 62],
                    [131, 131, 44],
                    [190, 145, 73],
                    [191, 155, 115],
                    [107, 114, 154],
                ],
            ),
        ] {
            let entry = &disc.files[name];
            let mut reader = BufReader::new(File::open(&disc.tracks[0].path).unwrap());
            reader
                .seek(SeekFrom::Start(u64::from(entry.sector) * 2352))
                .unwrap();
            let mut decoder =
                Decoder::new(reader.take(u64::from(entry.size.div_ceil(2048)) * 2352));
            let mut rgba = vec![0; WIDTH * HEIGHT * 4];
            for i in 0..frames {
                decoder.read_frame(&mut rgba).unwrap();
                if i == 30 {
                    let mut sums = [[0i32; 3]; 8];
                    for (p, color) in rgba.chunks_exact(4).enumerate() {
                        assert_eq!(color[3], 255);
                        let region = p / WIDTH / 120 * 4 + p % WIDTH / 80;
                        for (sum, &c) in sums[region].iter_mut().zip(color) {
                            *sum += i32::from(c);
                        }
                    }
                    for (region, expected) in sums.iter().zip(reference) {
                        for (&sum, target) in region.iter().zip(expected) {
                            assert!(
                                (sum / 9600 - target).abs() <= 2,
                                "{name}: {} vs {target}",
                                sum / 9600
                            );
                        }
                    }
                }
            }
            assert!(decoder.read_frame(&mut rgba).is_err());
        }
    }
}
