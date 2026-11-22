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

/// Indexed 128 × 64 TIM used for each entry in screenCarPictures.
pub fn vehicle_card(data: &[u8]) -> Result<Vec<[u8; 4]>> {
    ensure!(
        u32le(data, 0)? == 16 && u32le(data, 4)? == 9,
        "expected 8-bit vehicle card"
    );
    let palette = bytes(data, 8, u32le(data, 8)? as usize)?;
    let colors = bytes(palette, 12, 512)?;
    let start = 8 + palette.len();
    let bitmap = bytes(data, start, u32le(data, start)? as usize)?;
    ensure!(
        u16le(bitmap, 8)? == 64 && u16le(bitmap, 10)? == 64,
        "unexpected vehicle card dimensions"
    );
    Ok(bytes(bitmap, 12, 128 * 64)?
        .iter()
        .map(|&index| {
            let at = usize::from(index) * 2;
            let mut color = rgb555(u16::from_le_bytes([colors[at], colors[at + 1]]));
            for v in &mut color[..3] {
                *v = (u16::from(*v) * 127 / 128) as u8;
            }
            color
        })
        .collect())
}

pub struct Fonts {
    pixels: Vec<[u8; 4]>,
    width: usize,
    signs: Vec<[u8; 4]>,
    sign_width: usize,
    metrics: [Vec<Glyph>; 4],
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
    Speed,
    Sign,
    Choice,
    Selected,
    Control,
    ArrowActive,
    PasswordLabel,
    PasswordSelected,
}
impl Fonts {
    pub fn load(disc: &Disc) -> Result<Self> {
        let exe = disc.read("SIPS_600.07")?;
        let bitmap = |path: &str| -> Result<(Vec<[u8; 4]>, usize, usize)> {
            let tim = disc.read(path)?;
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
            Ok((pixels, width, height))
        };
        let (pixels, width, height) = bitmap("UATIM/UAFONT1.TIM")?;
        let (signs, sign_width, sign_height) = bitmap("TIM/CPFONT.TIM")?;
        let metrics =
            |address: u32, count: usize, width: usize, height: usize| -> Result<Vec<Glyph>> {
                let at = 2048
                    + address
                        .checked_sub(u32le(&exe, 24)?)
                        .ok_or_else(|| anyhow::anyhow!("invalid executable base"))?
                        as usize;
                bytes(&exe, at, count * 6)?
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
            signs,
            sign_width,
            metrics: [
                metrics(0x801709b8, 77, width, height)?,
                metrics(0x801707e8, 77, width, height)?,
                metrics(0x80170b88, 48, width, height)?,
                metrics(0x80170bc4, 77, sign_width, sign_height)?,
            ],
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
        let metrics = &self.metrics[match face {
            Face::Choice | Face::Selected | Face::PasswordLabel | Face::PasswordSelected => 1,
            Face::Control | Face::ArrowActive => 3,
            _ => face as usize,
        }];
        metrics.get(idx).copied().unwrap_or(metrics[47])
    }
    pub fn password_stride(&self) -> i32 {
        self.glyph(Face::Password, b'=').w as i32 + 7
    }
    pub fn line_height(&self) -> i32 {
        self.glyph(Face::Text, b'M').h as i32
    }
    pub fn centered(&self, frame: &mut [u8], face: Face, y: i32, text: &str) {
        let center = if matches!(face, Face::Sign) { 146 } else { 159 };
        self.centered_at(frame, face, center, y, text);
    }
    pub fn centered_at(&self, frame: &mut [u8], face: Face, center: i32, y: i32, text: &str) {
        let spacing = match face {
            Face::Text
            | Face::Choice
            | Face::Selected
            | Face::PasswordLabel
            | Face::PasswordSelected => -1,
            Face::Password => 7,
            Face::Speed => 3,
            Face::Sign | Face::Control | Face::ArrowActive => 0,
        };
        let advance = |c| self.glyph(face, c).w as i32 + spacing;
        let width: i32 = text
            .bytes()
            .map(|c| if c == 8 { -1 } else { advance(c) })
            .sum();
        self.at(frame, face, center - (width >> 1), y, text);
    }
    pub fn at(&self, frame: &mut [u8], face: Face, mut x: i32, y: i32, text: &str) {
        let (spacing, tint) = match face {
            Face::PasswordLabel => (-1, [55, 88, 127]),
            Face::PasswordSelected => (-1, [100; 3]),
            Face::Choice => (-1, [93, 30, 23]),
            Face::Selected => (-1, [115, 61, 27]),
            Face::Control => (0, [128; 3]),
            Face::ArrowActive => (0, [115, 25, 8]),
            Face::Text => (-1, [115, 84, 35]),
            Face::Password => (7, [50, 50, 50]),
            Face::Speed => (3, [100, 0, 30]),
            Face::Sign => (0, [36, 150, 36]),
        };
        let advance = |c| self.glyph(face, c).w as i32 + spacing;
        let (pixels, width) = if matches!(face, Face::Sign | Face::Control | Face::ArrowActive) {
            (&self.signs, self.sign_width)
        } else {
            (&self.pixels, self.width)
        };
        for c in text.bytes() {
            if c == 8 {
                x -= 1;
                continue;
            }
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
                        let pixel = pixels[(g.v + dy) * width + g.u + dx];
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

/// The car-specific UADASH TMS replaces seven cockpit panels and a steering
/// wheel. Mr. Grimm uses three instrument panels and two mirrored handlebars.
pub struct Cockpit {
    panels: Vec<u8>,
    steering: Vec<u8>,
    motorcycle: bool,
}
impl Cockpit {
    pub fn load(data: &[u8], layout: [[i32; 4]; 7], motorcycle: bool) -> Result<Self> {
        let count = if motorcycle { 4 } else { 8 };
        ensure!(
            u32le(data, 0)? == 0x50535854 && u32le(data, 4)? == 0x43 && u32le(data, 12)? == count,
            "invalid cockpit TMS"
        );
        let mut panels = vec![0; WIDTH * HEIGHT * 4];
        let mut wheel = Vec::new();
        let mut at = 16;
        for i in 0..count as usize {
            let len = u32le(data, at)? as usize;
            let tim = bytes(data, at + 4, len)?;
            at += 4 + len;
            let indexed = motorcycle || !matches!(i, 0 | 6);
            ensure!(
                u32le(tim, 0)? == 16 && u32le(tim, 4)? == if indexed { 9 } else { 2 },
                "unexpected cockpit TIM format"
            );
            let palette = if indexed {
                bytes(tim, 8, u32le(tim, 8)? as usize)?
            } else {
                &[]
            };
            let p = 8 + palette.len();
            let img = bytes(tim, p, u32le(tim, p)? as usize)?;
            let w = usize::from(u16le(img, 8)?) * if indexed { 2 } else { 1 };
            let h = usize::from(u16le(img, 10)?);
            let bitmap = bytes(img, 12, w * h * if indexed { 1 } else { 2 })?;
            let mut rgba = Vec::with_capacity(w * h * 4);
            for j in 0..w * h {
                let word = if indexed {
                    u16le(palette, 12 + 2 * usize::from(bitmap[j]))?
                } else {
                    u16le(bitmap, 2 * j)?
                };
                let mut color = rgb555(word);
                if !indexed {
                    color[3] = 255;
                } else if i + 1 == count as usize && word & 0x8000 != 0 {
                    color[3] = 128;
                }
                rgba.extend(color);
            }
            if i + 1 == count as usize {
                ensure!(
                    (w, h) == (256, 128),
                    "unexpected steering texture dimensions"
                );
                wheel = rgba;
                continue;
            }
            let [x, y, rw, rh] = if motorcycle {
                [[117, 132, 64, 36], [85, 168, 128, 36], [85, 204, 128, 36]][i]
            } else {
                layout[i]
            };
            ensure!(
                x >= 0
                    && y >= 0
                    && rw == w as i32
                    && rh == h as i32
                    && x as usize + w <= WIDTH
                    && y as usize + h <= HEIGHT,
                "cockpit panel outside display"
            );
            for row in 0..h {
                let dst = ((row + y as usize) * WIDTH + x as usize) * 4;
                panels[dst..dst + w * 4].copy_from_slice(&rgba[row * w * 4..(row + 1) * w * 4]);
            }
        }
        Ok(Self {
            panels,
            steering: wheel,
            motorcycle,
        })
    }

    pub fn frame(&self, degrees: i32) -> Vec<u8> {
        let mut pixels = self.panels.clone();
        let (sin, cos) = (degrees as f32).to_radians().sin_cos();
        let (cx, cy) = if self.motorcycle {
            (159., 202.)
        } else {
            (125., 230.)
        };
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let lx = cos * dx + sin * dy;
                let ly = -sin * dx + cos * dy;
                let (u, v) = if self.motorcycle {
                    if !(14.0..114.0).contains(&lx.abs()) || !(-130.0..126.0).contains(&ly) {
                        continue;
                    }
                    (126. - ly, (114. - lx.abs()) * 128. / 100.)
                } else {
                    (lx + 120., ly + 64.)
                };
                if !(0.0..256.0).contains(&u) || !(0.0..128.0).contains(&v) {
                    continue;
                }
                let src = 4 * (v as usize * 256 + u as usize);
                let dst = 4 * (y * WIDTH + x);
                let alpha = self.steering[src + 3];
                if alpha == 0 {
                    continue;
                }
                let blend = alpha == 128 && pixels[dst + 3] != 0;
                for c in 0..3 {
                    let front = (u16::from(self.steering[src + c]) * 255 / 128).min(255);
                    pixels[dst + c] = if blend {
                        ((front + u16::from(pixels[dst + c])) / 2) as u8
                    } else {
                        front as u8
                    };
                }
                pixels[dst + 3] = if blend { 255 } else { alpha };
            }
        }
        pixels
    }
}
