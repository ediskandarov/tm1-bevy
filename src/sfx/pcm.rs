//! Shared VAB samples with per-voice stereo levels and optional ADPCM loop points.
use bevy::{
    audio::{ChannelCount, Decodable, SampleRate, Source},
    prelude::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU16, AtomicU32, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Copy)]
pub(super) struct VabVolume {
    pub program: u8,
    pub tone: u8,
}

impl VabVolume {
    pub fn from_tone(tone: &tm1_assets::sound::Tone) -> Self {
        Self {
            program: tone.program_volume,
            tone: tone.volume,
        }
    }
    // SOUND.c soundProcessIds truncates after each scale. soundPlaySound caps the SDK value.
    pub fn gain(self, channel: u8, percent: u8) -> f32 {
        f32::from(tm1_core::sound::voice_volume(
            self.program,
            self.tone,
            channel,
            percent,
        )) / 33.
    }
}

#[derive(Clone)]
pub(crate) struct Levels(Arc<AtomicU32>);

impl Levels {
    pub fn new(channels: [i32; 2], percent: i32) -> Self {
        let levels = Self(Arc::default());
        levels.set(channels, percent);
        levels
    }
    pub fn set(&self, channels: [i32; 2], percent: i32) {
        let [l, r] = channels.map(|v| v.clamp(0, 255) as u32);
        self.0.store(
            l | (r << 8) | ((percent.clamp(0, 255) as u32) << 16),
            Ordering::Relaxed,
        );
    }
}

/// The game thread writes the SPU pitch register. The audio thread samples it
/// once per stereo frame, without moving the voice's oscillator phase.
#[derive(Clone)]
pub(super) struct PitchRegister(Arc<AtomicU16>);
impl PitchRegister {
    pub fn set(&self, value: u16) {
        self.0.store(value.min(0x4000), Ordering::Relaxed);
    }
    pub fn get(&self) -> u16 {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub(super) struct LoopingTone {
    pcm: Arc<[i16]>,
    loop_at: usize,
    adsr: [u16; 2],
    volume: VabVolume,
}
impl LoopingTone {
    pub fn new(mut tone: tm1_assets::sound::Tone) -> anyhow::Result<Self> {
        tone.prepare_loop();
        let loop_at = tone
            .loop_start
            .ok_or_else(|| anyhow::anyhow!("missing VAB loop"))?;
        let volume = VabVolume::from_tone(&tone);
        Ok(Self {
            pcm: tone.pcm.into(),
            loop_at,
            adsr: tone.adsr,
            volume,
        })
    }
    pub fn voice(&self, pitch: u16) -> (StereoPcm, PitchRegister) {
        let register = PitchRegister(Arc::new(AtomicU16::new(pitch.min(0x4000))));
        (
            StereoPcm {
                data: PcmData::Looping(self.clone(), register.clone()),
                volume: self.volume,
                levels: None,
            },
            register,
        )
    }
}
#[derive(Clone)]
enum PcmData {
    Stereo { samples: Arc<[f32]> },
    Looping(LoopingTone, PitchRegister),
}
#[derive(Asset, TypePath, Clone)]
pub struct StereoPcm {
    data: PcmData,
    pub(super) volume: VabVolume,
    levels: Option<Levels>,
}

impl StereoPcm {
    pub(crate) fn one_shot(tone: &tm1_assets::sound::Tone) -> Self {
        let mut raw = Vec::new();
        super::Sample::mix_tone(&mut raw, tone, false, 1.);
        Self::new(&raw, VabVolume::from_tone(tone))
    }

    pub(super) fn new(stereo: &[[f32; 2]], volume: VabVolume) -> Self {
        Self {
            volume,
            data: PcmData::Stereo {
                samples: stereo
                    .iter()
                    .flatten()
                    .map(|s| s.clamp(-32768., 32767.) / 32768.)
                    .collect(),
            },
            levels: None,
        }
    }

    #[cfg(test)]
    pub(super) fn unscaled_decoder(&self) -> Playback {
        let mut clip = self.clone();
        clip.levels = None;
        clip.volume = VabVolume {
            program: 99,
            tone: 99,
        };
        clip.decoder()
    }

    pub(crate) fn with_levels(&self, levels: Levels) -> Self {
        Self {
            levels: Some(levels),
            ..self.clone()
        }
    }
}

pub struct Playback {
    clip: StereoPcm,
    cursor: usize,
    phase: usize,
    mono: f32,
    frame_gain: [f32; 2],
    last_levels: u32,
    envelope: Option<super::envelope::Envelope>,
}

impl Decodable for StereoPcm {
    type Decoder = Playback;
    fn decoder(&self) -> Playback {
        Playback {
            clip: self.clone(),
            cursor: 0,
            phase: 0,
            mono: 0.,
            frame_gain: [0.; 2],
            last_levels: u32::MAX,
            envelope: match &self.data {
                PcmData::Looping(motor, _) => Some(super::envelope::Envelope::new(motor.adsr)),
                PcmData::Stereo { .. } => None,
            },
        }
    }
}
impl Iterator for Playback {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let mut sample = match &self.clip.data {
            PcmData::Stereo { samples } => *samples.get(self.cursor)?,
            PcmData::Looping(motor, register) => {
                if self.cursor & 1 == 0 {
                    let mono = super::interpolation::interpolate(&motor.pcm, self.phase);
                    let v = self.envelope.as_mut().unwrap().apply(mono) as f32;
                    self.mono = v.clamp(-32768., 32767.) / 32768.;
                    self.phase += usize::from(register.get());
                    if self.phase >= motor.pcm.len() * 4096 {
                        self.phase -= (motor.pcm.len() - motor.loop_at) * 4096;
                    }
                }
                self.mono
            }
        };
        if self.cursor % 2 == 0 {
            let levels = self
                .clip
                .levels
                .as_ref()
                .map_or(33 | (33 << 8) | (99 << 16), |l| l.0.load(Ordering::Relaxed));
            if levels != self.last_levels {
                let [left, right, percent, _] = levels.to_le_bytes();
                self.frame_gain =
                    [left, right].map(|channel| self.clip.volume.gain(channel, percent));
                self.last_levels = levels;
            }
        }
        sample *= self.frame_gain[self.cursor % 2];
        self.cursor = self.cursor.wrapping_add(1);
        Some(sample)
    }
}
impl Source for Playback {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn total_duration(&self) -> Option<Duration> {
        match &self.clip.data {
            PcmData::Stereo { samples } => {
                Some(Duration::from_secs_f64(samples.len() as f64 / 88200.))
            }
            _ => None,
        }
    }
    fn channels(&self) -> ChannelCount {
        ChannelCount::new(2).unwrap()
    }
    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(44100).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_voices_latch_channel_updates_at_frame_boundaries() {
        let clip = StereoPcm::new(
            &[[8192., 4096.]; 3],
            VabVolume {
                program: 99,
                tone: 99,
            },
        );
        let levels = Levels::new([16, 47], 99);
        let mut a = clip.with_levels(levels.clone()).decoder();
        let mut b = clip.with_levels(Levels::new([47, 16], 50)).decoder();
        assert_eq!(a.next(), Some(0.25 * (16. / 33.)));
        levels.set([39, 21], 25);
        assert_eq!(a.next(), Some(0.125 * (47. / 33.)));
        assert_eq!(a.next(), Some(0.25 * (9. / 33.)));
        assert_eq!(a.next(), Some(0.125 * (5. / 33.)));
        assert_eq!(b.next(), Some(0.25 * (23. / 33.)));
        assert_eq!(b.next(), Some(0.125 * (8. / 33.)));
        levels.set([50, 33], 109);
        assert_eq!(a.next(), Some(0.25 * (55. / 33.)));
        assert_eq!(a.next(), Some(0.125 * (36. / 33.)));
        assert_eq!(a.next(), None);
        assert_eq!(b.count(), 4);
    }
}
