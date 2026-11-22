use crate::{bytes, scene::TextureKey, u16le, u32le};
use anyhow::{Result, ensure};

pub struct Vram {
    words: Vec<u16>,
    pub version: u32,
    pub tim_count: usize,
}
impl Vram {
    pub fn from_tms(data: &[u8]) -> Result<Self> {
        ensure!(
            u32le(data, 0)? == 0x50535854 && u32le(data, 4)? == 0x43,
            "unsupported TMS header"
        );
        let count = u32le(data, 12)? as usize;
        ensure!(count < 4096, "unreasonable TIM count");
        let mut v = Self {
            words: vec![0; 1024 * 512],
            version: u32le(data, 8)?,
            tim_count: count,
        };
        let mut p = 16;
        for _ in 0..count {
            let size = u32le(data, p)? as usize;
            let tim = bytes(data, p + 4, size)?;
            ensure!(u32le(tim, 0)? == 0x10, "invalid TIM magic");
            let flags = u32le(tim, 4)?;
            ensure!(flags & 7 <= 2, "unsupported TIM format");
            let mut block = 8;
            if flags & 8 != 0 {
                block += v.upload(tim, block)?;
            }
            block += v.upload(tim, block)?;
            ensure!(block <= size, "TIM block exceeds container");
            p += 4 + size;
        }
        Ok(v)
    }
    fn upload(&mut self, tim: &[u8], p: usize) -> Result<usize> {
        let size = u32le(tim, p)? as usize;
        let b = bytes(tim, p, size)?;
        let x = u16le(b, 4)? as usize;
        let y = u16le(b, 6)? as usize;
        let w = u16le(b, 8)? as usize;
        let h = u16le(b, 10)? as usize;
        ensure!(x + w <= 1024 && y + h <= 512, "TIM rectangle exceeds VRAM");
        bytes(b, 12, w * h * 2)?;
        for row in 0..h {
            for col in 0..w {
                self.words[(y + row) * 1024 + x + col] = u16le(b, 12 + 2 * (row * w + col))?;
            }
        }
        Ok(size)
    }
    pub fn sample(&self, key: TextureKey, u: u8, v: u8) -> Result<[u8; 4]> {
        self.sample_primitive(key, u, v, false)
    }
    pub fn sample_primitive(&self, key: TextureKey, u: u8, v: u8, semi: bool) -> Result<[u8; 4]> {
        let mask_x = (key.window & 31) * 8;
        let mask_y = ((key.window >> 5) & 31) * 8;
        let offset_x = ((key.window >> 10) & 31) * 8;
        let offset_y = ((key.window >> 15) & 31) * 8;
        let u = ((u as u32 & !mask_x) | (offset_x & mask_x)) as usize;
        let v = ((v as u32 & !mask_y) | (offset_y & mask_y)) as usize;
        let x = (key.page as usize & 15) * 64;
        let y = ((key.page as usize >> 4) & 1) * 256;
        let mode = (key.page >> 7) & 3;
        ensure!(mode <= 2, "unsupported texture depth");
        let read = |x: usize, y: usize| self.words[(y % 512) * 1024 + x % 1024];
        let word = match mode {
            0 => {
                let index = (read(x + u / 4, y + v) >> ((u % 4) * 4)) & 15;
                read(
                    (key.clut as usize & 63) * 16 + index as usize,
                    key.clut as usize >> 6,
                )
            }
            1 => {
                let index = (read(x + u / 2, y + v) >> ((u % 2) * 8)) & 255;
                read(
                    (key.clut as usize & 63) * 16 + index as usize,
                    key.clut as usize >> 6,
                )
            }
            _ => read(x + u, y + v),
        };
        let mut color = rgb555(word);
        if semi && word & 0x8000 != 0 {
            color[3] = 128;
        }
        Ok(color)
    }
}
fn rgb555(word: u16) -> [u8; 4] {
    let expand = |v: u16| ((v << 3) | (v >> 2)) as u8;
    [
        expand(word & 31),
        expand((word >> 5) & 31),
        expand((word >> 10) & 31),
        if word == 0 { 0 } else { 255 },
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_transparency_and_window() {
        let mut v = Vram {
            words: vec![0; 1024 * 512],
            version: 0,
            tim_count: 0,
        };
        v.words[0] = 0x3210;
        v.words[64] = 0;
        v.words[65] = 0x001f;
        v.words[66] = 0x03e0;
        v.words[67] = 0x8000;
        let key = TextureKey {
            clut: 4,
            ..Default::default()
        };
        assert_eq!(v.sample(key, 0, 0).unwrap(), [0, 0, 0, 0]);
        assert_eq!(v.sample(key, 1, 0).unwrap(), [255, 0, 0, 255]);
        assert_eq!(v.sample(key, 2, 0).unwrap(), [0, 255, 0, 255]);
        assert_eq!(v.sample(key, 3, 0).unwrap(), [0, 0, 0, 255]);
        assert_eq!(v.sample_primitive(key, 3, 0, true).unwrap(), [0, 0, 0, 128]);
        let window = TextureKey { window: 31, ..key };
        assert_eq!(v.sample(window, 9, 0).unwrap(), [255, 0, 0, 255]);
    }
    #[test]
    fn truncated_tms_fails() {
        assert!(Vram::from_tms(b"TXSP").is_err());
    }
}
