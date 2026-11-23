// SPDX-License-Identifier: GPL-2.0-or-later
//! STR video sectors: the 32-byte chunk header and frame reassembly.
//!
//! Every video sector's 2048 user bytes are a 32-byte header plus 2016
//! bytes of one frame's bitstream:
//!
//! | Bytes  | Field |
//! |--------|-------|
//! | 0..2   | `0x0160` |
//! | 2..4   | `0x8001` (MDEC video chunk) |
//! | 4..6   | chunk index within the frame |
//! | 6..8   | chunks in the frame |
//! | 8..12  | frame number (1-based) |
//! | 12..16 | frame bitstream size in bytes |
//! | 16..18 | width |
//! | 18..20 | height |
//! | 20..28 | copy of the BS frame header |
//!
//! Interleaved XA audio sectors never reach this layer: with the drive in
//! XA mode they go straight to the SPU and raise no data IRQ.

/// Bytes of chunk header at the start of each video sector.
pub const CHUNK_HEADER_BYTES: usize = 32;
/// Bitstream bytes carried per video sector.
pub const CHUNK_PAYLOAD_BYTES: usize = 2016;
/// Most chunks per frame [`FrameAssembler`] tracks (a 32-bit mask). At
/// 2016 bytes each that is 64 KB, far above any frame a 2x stream carries.
pub const MAX_CHUNKS: u16 = 32;

/// Parsed chunk header of one video sector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk {
    /// Index of this chunk within its frame.
    pub index: u16,
    /// Chunks in the frame.
    pub count: u16,
    /// Frame number.
    pub frame: u32,
    /// Frame bitstream size in bytes.
    pub size: u32,
    /// Frame width in pixels.
    pub width: u16,
    /// Frame height in pixels.
    pub height: u16,
}

impl Chunk {
    /// Parse a sector's user data; `None` if it is not an STR video chunk.
    pub fn parse(sector: &[u8]) -> Option<Chunk> {
        if sector.len() < CHUNK_HEADER_BYTES {
            return None;
        }
        let h = |i: usize| u16::from_le_bytes([sector[i], sector[i + 1]]);
        let w = |i: usize| u32::from_le_bytes(sector[i..i + 4].try_into().unwrap());
        if h(0) != 0x0160 || h(2) != 0x8001 {
            return None;
        }
        let chunk = Chunk {
            index: h(4),
            count: h(6),
            frame: w(8),
            size: w(12),
            width: h(16),
            height: h(18),
        };
        if chunk.count == 0 || chunk.count > MAX_CHUNKS || chunk.index >= chunk.count {
            return None;
        }
        Some(chunk)
    }
}

/// A frame whose every chunk has arrived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Frame number from the chunk headers.
    pub number: u32,
    /// Bitstream bytes.
    pub size: u32,
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
}

/// Collects one frame's chunks into a caller-owned buffer.
///
/// Chunks may arrive in any order. A chunk from a new frame while the
/// current one is incomplete abandons it (a lost sector) and counts it in
/// [`dropped`](Self::dropped).
#[derive(Default)]
pub struct FrameAssembler {
    frame: u32,
    mask: u32,
    count: u16,
    /// Frames abandoned with chunks missing.
    pub dropped: u32,
}

impl FrameAssembler {
    /// Empty assembler.
    pub const fn new() -> Self {
        FrameAssembler {
            frame: 0,
            mask: 0,
            count: 0,
            dropped: 0,
        }
    }

    /// Add one sector (2048 user bytes). `buf` is the frame being filled
    /// and must hold `count * 2016` bytes. Returns the finished frame when
    /// this chunk completes it; the caller then hands `buf` to the decoder
    /// and passes a different buffer from now on.
    pub fn add(&mut self, sector: &[u8], buf: &mut [u8]) -> Option<Frame> {
        let chunk = Chunk::parse(sector)?;
        if chunk.frame != self.frame || chunk.count != self.count {
            if self.mask != 0 {
                self.dropped += 1;
            }
            self.frame = chunk.frame;
            self.count = chunk.count;
            self.mask = 0;
        }
        let at = chunk.index as usize * CHUNK_PAYLOAD_BYTES;
        let end = at + CHUNK_PAYLOAD_BYTES;
        if end > buf.len() || sector.len() < CHUNK_HEADER_BYTES + CHUNK_PAYLOAD_BYTES {
            return None;
        }
        buf[at..end]
            .copy_from_slice(&sector[CHUNK_HEADER_BYTES..CHUNK_HEADER_BYTES + CHUNK_PAYLOAD_BYTES]);
        self.mask |= 1 << chunk.index;
        let full = if chunk.count == 32 {
            u32::MAX
        } else {
            (1u32 << chunk.count) - 1
        };
        if self.mask != full {
            return None;
        }
        self.mask = 0;
        self.frame = 0;
        Some(Frame {
            number: chunk.frame,
            size: chunk.size,
            width: chunk.width,
            height: chunk.height,
        })
    }
}
