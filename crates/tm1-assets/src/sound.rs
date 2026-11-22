//! PsyQ VAB programs and hardware ADPCM frames from the supplied disc.
use crate::{bytes, u16le, u32le};
use anyhow::{Result, ensure};

pub struct Tone {
    pub priority: u8,
    pub adsr: [u16; 2],
    pub pcm: Vec<i16>,
    pub loop_start: Option<usize>,
    repeat_prefix: Vec<i16>,
    pub center: u8,
    pub fine: u8,
    pub low: u8,
    pub high: u8,
    pub pan: u8,
    pub volume: u8,
    pub program_volume: u8,
}

impl Tone {
    /// Extend the first pass through the loop with the samples affected by the
    /// end block's predictor history. Rotate the loop point past that prefix.
    pub fn prepare_loop(&mut self) {
        if let Some(start) = &mut self.loop_start {
            *start += self.repeat_prefix.len();
            self.pcm.append(&mut self.repeat_prefix);
        }
    }
}

fn decode_block(block: &[u8], history: &mut [i32; 2], pcm: &mut Vec<i16>) -> Result<()> {
    let filter = usize::from(block[0] >> 4);
    let shift = block[0] & 15;
    ensure!(filter < 5 && shift <= 12, "invalid ADPCM predictor");
    let (c, d) = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)][filter];
    for &byte in &block[2..] {
        for nibble in [byte & 15, byte >> 4] {
            let residual = ((i32::from(nibble) << 28) >> 16) >> shift;
            // The SPU shifts each history term before adding it to the residual.
            let v =
                (residual + ((history[0] * c) >> 6) + ((history[1] * d) >> 6)).clamp(-32768, 32767);
            pcm.push(v as i16);
            *history = [v, history[0]];
        }
    }
    Ok(())
}

pub fn program(bank: &[u8], program: usize) -> Result<Vec<Tone>> {
    ensure!(
        bank.get(..4) == Some(b"pBAV") && u32le(bank, 12)? as usize == bank.len(),
        "invalid VAB"
    );
    let programs = u16le(bank, 18)? as usize;
    let samples = u16le(bank, 22)? as usize;
    ensure!(
        program < 128 && programs <= 128 && samples < 256,
        "invalid VAB table counts"
    );
    let attrs = bytes(bank, 32, 2048)?;
    let count = attrs[program * 16] as usize;
    let rank = (0..program).filter(|&i| attrs[16 * i] != 0).count();
    ensure!(
        count > 0 && count <= 16 && rank < programs,
        "missing VAB program {program}"
    );
    let sizes = 2080 + programs * 512;
    let start = sizes + 512;
    (0..count)
        .map(|i| {
            let t = bytes(bank, 2080 + 512 * rank + 32 * i, 32)?;
            let sample = u16le(t, 22)? as usize;
            ensure!(
                sample > 0 && sample <= samples,
                "invalid tone sample {sample}"
            );
            let offset = (0..sample).try_fold(start, |at, j| {
                Ok::<_, anyhow::Error>(at + 8 * u16le(bank, sizes + j * 2)? as usize)
            })?;
            let raw = bytes(bank, offset, 8 * u16le(bank, sizes + sample * 2)? as usize)?;
            ensure!(raw.len() % 16 == 0, "incomplete ADPCM block");
            let mut pcm = Vec::with_capacity(raw.len() / 16 * 28);
            let mut history = [0; 2];
            let mut loop_start = None;
            for block in raw.chunks_exact(16) {
                if block[1] & 4 != 0 {
                    loop_start = Some(pcm.len());
                }
                decode_block(block, &mut history, &mut pcm)?;
                if block[1] & 1 != 0 {
                    if block[1] & 2 == 0 {
                        loop_start = None;
                    }
                    break;
                }
            }
            ensure!(!pcm.is_empty(), "empty VAB sample");
            let mut repeat_prefix = Vec::new();
            if let Some(start) = loop_start {
                let end_history = history;
                for frame in (start..pcm.len()).step_by(28) {
                    let at = frame / 28 * 16;
                    decode_block(&raw[at..at + 16], &mut history, &mut repeat_prefix)?;
                    if history == [i32::from(pcm[frame + 27]), i32::from(pcm[frame + 26])] {
                        break;
                    }
                }
                ensure!(
                    repeat_prefix.len() < pcm.len() - start || history == end_history,
                    "unstable loop predictor in VAB sample {sample}"
                );
                while repeat_prefix
                    .last()
                    .is_some_and(|v| *v == pcm[start + repeat_prefix.len() - 1])
                {
                    repeat_prefix.pop();
                }
                // The SPU interpolator also needs three past samples at the loop boundary.
                let period = pcm.len() - start;
                for _ in 0..3 {
                    let i = repeat_prefix.len();
                    let sample = if i < period {
                        pcm[start + i]
                    } else {
                        repeat_prefix[i - period]
                    };
                    repeat_prefix.push(sample);
                }
            }
            Ok(Tone {
                priority: t[0],
                adsr: [u16le(t, 16)?, u16le(t, 18)?],
                repeat_prefix,
                loop_start,
                pcm,
                center: t[4],
                fine: t[5],
                low: t[6],
                high: t[7],
                volume: t[2],
                program_volume: attrs[program * 16 + 1],
                pan: t[3],
            })
        })
        .collect()
}
