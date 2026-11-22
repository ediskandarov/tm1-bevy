pub mod disc;
pub mod scene;
pub mod texture;

use anyhow::{Result, bail};

pub(crate) fn bytes(data: &[u8], offset: usize, len: usize) -> Result<&[u8]> {
    match offset
        .checked_add(len)
        .and_then(|end| data.get(offset..end))
    {
        Some(b) => Ok(b),
        None => bail!(
            "range {offset:#x}+{len:#x} exceeds buffer of {} bytes",
            data.len()
        ),
    }
}
pub(crate) fn u16le(data: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(bytes(data, offset, 2)?.try_into()?))
}
pub(crate) fn u32le(data: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(bytes(data, offset, 4)?.try_into()?))
}
pub mod sound;
