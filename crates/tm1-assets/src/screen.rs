//! SCREEN.c framebuffers and FONT.c layout for the disc font atlases.
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
pub const CARD_FILES: [&str; 12] = [
    "ICE_CARD", "CAB_CARD", "SEMI_CAR", "COP_CARD", "LOWRIDER", "LAMBO_CA", "DUNEB_CA", "HUMVEE_C",
    "HARLEY_C", "VETTE_CA", "MONSTERT", "MADMAX_C",
];
pub fn load_card(disc: &Disc, index: usize) -> Result<Vec<[u8; 4]>> {
    vehicle_card(&disc.read(&format!("UATIM/{}.TIM", CARD_FILES[index]))?)
}
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
    let palette: [[u8; 4]; 256] =
        std::array::from_fn(|i| card_color(u16::from_le_bytes([colors[i * 2], colors[i * 2 + 1]])));
    Ok(bytes(bitmap, 12, 128 * 64)?
        .iter()
        .map(|&index| palette[usize::from(index)])
        .collect())
}

fn card_color(word: u16) -> [u8; 4] {
    // The FT4 multiplies each five-bit component by 127/128 before framebuffer storage.
    let lit = |shift: u32| (((word >> shift) & 31) * 127) >> 7;
    let mut color = rgb555(lit(0) | (lit(5) << 5) | (lit(10) << 10));
    color[3] = if word == 0 { 0 } else { 255 };
    color
}

#[test]
fn opponent_card_modulation_preserves_texel_coverage() {
    assert_eq!(card_color(0), [0; 4]);
    assert_eq!(card_color(0x8000), [0, 0, 0, 255]);
    assert_eq!(card_color(1), [0, 0, 0, 255]);
    assert_eq!(card_color(0x7fff), [247, 247, 247, 255]);
    assert_eq!(card_color(0x400), [0, 0, 0, 255]);
    assert_eq!(card_color(0x40), [0, 8, 0, 255]);
}

mod font_layout;
use font_layout::{CAR_NAMES, CAR_SIGNS, PASSWORD, SIGN, SPEED_DIGITS, TEXT};

const FALLBACK_GLYPH: usize = 47;

pub struct Fonts {
    pixels: Vec<[u8; 4]>,
    width: usize,
    signs: Vec<[u8; 4]>,
    sign_width: usize,
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
        for (metrics, w, h) in [
            (&TEXT[..], width, height),
            (&PASSWORD[..], width, height),
            (&SPEED_DIGITS[..], width, height),
            (&SIGN[..=FALLBACK_GLYPH - SPEED_DIGITS.len()], width, height),
            (&SIGN[..], sign_width, sign_height),
        ] {
            ensure!(
                metrics.iter().all(|g| g.u + g.w <= w && g.v + g.h <= h),
                "glyph outside font TIM"
            );
        }
        Ok(Self {
            pixels,
            width,
            signs,
            sign_width,
        })
    }
    pub fn car_name(&self, card: usize) -> &str {
        CAR_NAMES[card]
    }
    pub fn car_sign(&self, card: usize) -> &str {
        CAR_SIGNS[card]
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
            FALLBACK_GLYPH
        };
        match face {
            Face::Text => TEXT[idx],
            Face::Password
            | Face::Choice
            | Face::Selected
            | Face::PasswordLabel
            | Face::PasswordSelected => PASSWORD[idx],
            Face::Sign | Face::Control | Face::ArrowActive => SIGN[idx],
            Face::Speed => {
                // The JP speed lookup continues into the adjacent sign table after digit 9.
                // Slot 47 supplies the advance for the spaces before a short speed.
                let slot = idx.min(FALLBACK_GLYPH);
                if slot < SPEED_DIGITS.len() {
                    SPEED_DIGITS[slot]
                } else {
                    SIGN[slot - SPEED_DIGITS.len()]
                }
            }
        }
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
        let width = self.width(face, text);
        self.at(frame, face, center - (width >> 1), y, text);
    }
    pub fn width(&self, face: Face, text: &str) -> i32 {
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
        text.bytes().fold(0, |width, c| match c {
            b'\n' => 0,
            b'\r' => width,
            8 => width - 1,
            _ => width + self.glyph(face, c).w as i32 + spacing,
        })
    }
    pub fn sprite_centered_at(
        &self,
        frame: &mut [u8],
        face: Face,
        center: i32,
        y: i32,
        text: &str,
    ) {
        self.sprite_at(frame, face, center - (self.width(face, text) >> 1), y, text);
    }
    pub fn sprite_at(&self, frame: &mut [u8], face: Face, x: i32, y: i32, text: &str) {
        self.draw_text(frame, face, [x, y], text, true);
    }
    pub fn at(&self, frame: &mut [u8], face: Face, x: i32, y: i32, text: &str) {
        self.draw_text(frame, face, [x, y], text, false);
    }
    fn draw_text(
        &self,
        frame: &mut [u8],
        face: Face,
        [mut x, mut y]: [i32; 2],
        text: &str,
        sprites: bool,
    ) {
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
        let (pixels, width) = if matches!(face, Face::Sign | Face::Control | Face::ArrowActive) {
            (&self.signs, self.sign_width)
        } else {
            (&self.pixels, self.width)
        };
        let mut draw = |(x, y, g): (i32, i32, Glyph)| {
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
                            let shaded = ((u16::from(pixel[i] >> 3) * tint[i]) >> 7).min(31) as u8;
                            frame[at + i] = (shaded << 3) | (shaded >> 2);
                        }
                        frame[at + 3] = 255;
                    }
                }
            }
        };
        let glyphs = text.bytes().filter_map(|c| match c {
            b'\n' | b'\r' => {
                if c == b'\n' {
                    x = 0;
                }
                y += self.glyph(face, b'[').h as i32;
                None
            }
            8 => {
                x -= 1;
                None
            }
            _ => {
                let g = self.glyph(face, c);
                let position = (x, y, g);
                x += g.w as i32 + spacing;
                (c != b' ').then_some(position)
            }
        });
        if sprites {
            // FONT.c links sprites at the head of the ordering table.
            glyphs.collect::<Vec<_>>().into_iter().rev().for_each(draw);
        } else {
            glyphs.for_each(&mut draw);
        }
    }
}

#[derive(Clone, Copy)]
pub struct SteeringQuad {
    pub xy: [[i32; 2]; 4],
    pub uv: [[i32; 2]; 4],
}

impl SteeringQuad {
    pub fn handlebars(self) -> [Self; 2] {
        [
            SteeringQuad {
                xy: [[114, 126], [14, 126], [114, -130], [14, -130]],
                uv: [3, 1, 2, 0].map(|i| self.uv[i]),
            },
            SteeringQuad {
                xy: [[-14, 126], [-114, 126], [-14, -130], [-114, -130]],
                uv: [1, 3, 0, 2].map(|i| self.uv[i]),
            },
        ]
    }
    pub fn project(&self, [cx, cy]: [i32; 2], [sin, cos]: [i16; 2]) -> [[i32; 2]; 4] {
        let (sin, cos) = (i32::from(sin), i32::from(cos));
        self.xy.map(|[x, y]| {
            [
                cx + ((x * cos - y * sin) >> 12),
                cy + ((x * sin + y * cos) >> 12),
            ]
        })
    }
}

/// The car-specific UADASH TMS replaces seven cockpit panels and a steering
/// wheel. Mr. Grimm uses three instrument panels and two mirrored handlebars.
pub struct Cockpit {
    panels: Vec<u8>,
    steering: Vec<u8>,
    motorcycle: bool,
    quads: Vec<SteeringQuad>,
}
impl Cockpit {
    pub fn load(
        data: &[u8],
        layout: [[i32; 4]; 7],
        wheel_face: SteeringQuad,
        motorcycle: bool,
    ) -> Result<Self> {
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
            quads: if motorcycle {
                wheel_face.handlebars().to_vec()
            } else {
                vec![wheel_face]
            },
            motorcycle,
        })
    }

    pub fn frame(&self, [sin, cos]: [i16; 2]) -> Vec<u8> {
        let mut pixels = self.panels.clone();
        let (cx, cy) = if self.motorcycle {
            (159, 202)
        } else {
            (125, 230)
        };
        for quad in &self.quads {
            let points = quad.project([cx, cy], [sin, cos]);
            for tri in [[0, 1, 2], [1, 3, 2]] {
                self.draw_triangle(&mut pixels, tri.map(|i| points[i]), tri.map(|i| quad.uv[i]));
            }
        }
        pixels
    }

    fn draw_triangle(&self, pixels: &mut [u8], xy: [[i32; 2]; 3], uv: [[i32; 2]; 3]) {
        raster_triangle(xy, uv, |dst, [u, v]| {
            let src = 4 * (v * 256 + u);
            let alpha = self.steering[src + 3];
            if alpha == 0 {
                return;
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
        });
    }
}

/// Visit covered pixel offsets and affine texture coordinates in a 320 by 240 frame.
pub fn raster_triangle(
    xy: [[i32; 2]; 3],
    uv: [[i32; 2]; 3],
    mut pixel: impl FnMut(usize, [usize; 2]),
) {
    let edge = |[ax, ay]: [i32; 2], [bx, by]: [i32; 2], x: i32, y: i32| {
        (bx - ax) * (y - 2 * ay) - (by - ay) * (x - 2 * ax)
    };
    let area = edge(xy[0], xy[1], 2 * xy[2][0], 2 * xy[2][1]);
    if area == 0 {
        return;
    }
    let [xs, ys] = [0, 1].map(|axis| xy.map(|p| p[axis]));
    let bounds = |v: [i32; 3], limit| {
        (*v.iter().min().unwrap()).max(0)..(*v.iter().max().unwrap()).min(limit)
    };
    for y in bounds(ys, HEIGHT as i32) {
        for x in bounds(xs, WIDTH as i32) {
            let edges = [(1, 2), (2, 0), (0, 1)];
            let weights =
                edges.map(|(a, b)| edge(xy[a], xy[b], 2 * x + 1, 2 * y + 1) * area.signum());
            // Include top and left edges only, so translucent shared edges blend once.
            if edges.iter().zip(weights).any(|(&(a, b), w)| {
                let dx = (xy[b][0] - xy[a][0]) * area.signum();
                let dy = (xy[b][1] - xy[a][1]) * area.signum();
                w < 0 || (w == 0 && !(dy < 0 || dy == 0 && dx > 0))
            }) {
                continue;
            }
            let [u, v] = [0, 1].map(|axis| {
                (0..3)
                    .map(|i| i64::from(weights[i]) * i64::from(uv[i][axis]))
                    .sum::<i64>()
                    / i64::from(area.abs())
            });
            pixel(
                4 * (y as usize * WIDTH + x as usize),
                [u as usize, v as usize],
            );
        }
    }
}
