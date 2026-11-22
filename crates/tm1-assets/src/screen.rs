//! SCREEN.c framebuffers and FONT.c glyphs from the JP disc.
use crate::{bytes, disc::Disc, texture::rgb555, u16le, u32le};
use anyhow::{Result, ensure};

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;

pub fn background(data: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        u32le(data, 0)? == 16 && u32le(data, 4)? == 2,
        "expected a direct-color TIM"
    );
    let block = bytes(data, 8, u32le(data, 8)? as usize)?;
    let w = usize::from(u16le(block, 8)?);
    let h = usize::from(u16le(block, 10)?);
    ensure!(
        w == WIDTH && h > 0 && h <= HEIGHT,
        "unexpected screen dimensions {w}x{h}"
    );
    let mut out = [0, 0, 0, 255].repeat(WIDTH * HEIGHT);
    for (pixel, word) in out
        .chunks_exact_mut(4)
        .zip(bytes(block, 12, w * h * 2)?.chunks_exact(2))
    {
        pixel[..3].copy_from_slice(&rgb555(u16::from_le_bytes([word[0], word[1]]))[..3]);
    }
    Ok(out)
}

pub struct Fonts {
    pixels: Vec<[u8; 4]>,
    width: usize,
    metrics: [Vec<Glyph>; 2],
}
#[derive(Clone, Copy)]
struct Glyph {
    u: usize,
    v: usize,
    w: usize,
    h: usize,
    y: i32,
}
#[derive(Clone, Copy)]
pub enum Face {
    Text,
    Password,
}
impl Fonts {
    pub fn load(disc: &Disc) -> Result<Self> {
        let exe = disc.read("SIPS_600.07")?;
        let tim = disc.read("UATIM/UAFONT1.TIM")?;
        ensure!(
            u32le(&tim, 0)? == 16 && u32le(&tim, 4)? == 8,
            "expected indexed font TIM"
        );
        let palette = bytes(&tim, 8, u32le(&tim, 8)? as usize)?;
        let colors = bytes(palette, 12, 32)?;
        let offset = 8 + palette.len();
        let bitmap = bytes(&tim, offset, u32le(&tim, offset)? as usize)?;
        let width = usize::from(u16le(bitmap, 8)?) * 4;
        let height = usize::from(u16le(bitmap, 10)?);
        let pixels = bytes(bitmap, 12, width * height / 2)?
            .iter()
            .flat_map(|b| [b & 15, b >> 4])
            .map(|i| {
                let at = usize::from(i) * 2;
                rgb555(u16::from_le_bytes([colors[at], colors[at + 1]]))
            })
            .collect();
        let metrics = |address: u32| -> Result<Vec<Glyph>> {
            let at = 2048
                + address
                    .checked_sub(u32le(&exe, 24)?)
                    .ok_or_else(|| anyhow::anyhow!("invalid executable base"))?
                    as usize;
            bytes(&exe, at, 77 * 6)?
                .chunks_exact(6)
                .map(|g| {
                    let glyph = Glyph {
                        u: g[0].into(),
                        v: g[1].into(),
                        w: g[2].into(),
                        h: g[3].into(),
                        y: i16::from_le_bytes([g[4], g[5]]).into(),
                    };
                    ensure!(
                        glyph.u + glyph.w <= width && glyph.v + glyph.h <= height,
                        "glyph outside font TIM"
                    );
                    Ok(glyph)
                })
                .collect()
        };
        Ok(Self {
            pixels,
            width,
            metrics: [metrics(0x801709b8)?, metrics(0x801707e8)?],
        })
    }
    fn glyph(&self, face: Face, c: u8) -> Glyph {
        let mapped = match c {
            b'.' => b';',
            b'!' => 92,
            b',' => b'^',
            39 => 96,
            b'*' => b'{',
            v => v,
        };
        let idx = if (48..125).contains(&mapped) {
            usize::from(mapped - 48)
        } else {
            47
        };
        self.metrics[face as usize][idx]
    }
    pub fn line_height(&self) -> i32 {
        self.glyph(Face::Text, b'M').h as i32
    }
    pub fn centered(&self, frame: &mut [u8], face: Face, y: i32, text: &str) {
        let spacing = match face {
            Face::Text => -1,
            Face::Password => 7,
        };
        let advance = |c| self.glyph(face, c).w as i32 + spacing;
        let width: i32 = text.bytes().map(advance).sum();
        self.at(frame, face, 159 - (width >> 1), y, text);
    }
    pub fn at(&self, frame: &mut [u8], face: Face, mut x: i32, y: i32, text: &str) {
        let (spacing, tint) = match face {
            Face::Text => (-1, [115, 84, 35]),
            Face::Password => (7, [50, 50, 50]),
        };
        let advance = |c| self.glyph(face, c).w as i32 + spacing;
        for c in text.bytes() {
            let g = self.glyph(face, c);
            if c != b' ' {
                for dy in 0..g.h {
                    let row = y - g.h as i32 + g.y + dy as i32;
                    if !(0..HEIGHT as i32).contains(&row) {
                        continue;
                    }
                    for dx in 0..g.w {
                        let col = x + dx as i32;
                        if !(0..WIDTH as i32).contains(&col) {
                            continue;
                        }
                        let pixel = self.pixels[(g.v + dy) * self.width + g.u + dx];
                        if pixel[3] != 0 {
                            let at = 4 * (row as usize * WIDTH + col as usize);
                            for i in 0..3 {
                                frame[at + i] =
                                    (u16::from(pixel[i]) * tint[i] / 128).min(255) as u8;
                            }
                            frame[at + 3] = 255;
                        }
                    }
                }
            }
            x += advance(c);
        }
    }
}
