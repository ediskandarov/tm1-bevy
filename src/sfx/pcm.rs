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

#[derive(Clone)]
pub(super) struct Levels(Arc<AtomicU32>);

impl Levels {
    pub fn new(channels: [i32; 2], percent: i32) -> Self {
        let levels = Self(Arc::default());
        levels.set(channels, percent);
        levels
    }
    pub fn set(&self, channels: [i32; 2], percent: i32) {
        let [l, r] = channels.map(|v| (v * percent).clamp(0, 65535) as u32);
        self.0.store(l | (r << 16), Ordering::Relaxed);
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
    volume: f32,
    pan: [f32; 2],
}
impl LoopingTone {
    pub fn new(mut tone: tm1_assets::sound::Tone) -> anyhow::Result<Self> {
        tone.prepare_loop();
        let loop_at = tone
            .loop_start
            .ok_or_else(|| anyhow::anyhow!("missing VAB loop"))?;
        let pan = f32::from(tone.pan) / 64.;
        Ok(Self {
            pcm: tone.pcm.into(),
            loop_at,
            adsr: tone.adsr,
            volume: f32::from(tone.volume) / 127.,
            pan: [(2. - pan).min(1.), pan.min(1.)],
        })
    }
    pub fn voice(&self, pitch: u16) -> (StereoPcm, PitchRegister) {
        let register = PitchRegister(Arc::new(AtomicU16::new(pitch.min(0x4000))));
        (
            StereoPcm {
                data: PcmData::Looping(self.clone(), register.clone()),
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
    levels: Option<Levels>,
}

impl StereoPcm {
    pub(super) fn new(stereo: &[[f32; 2]]) -> Self {
        Self {
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

    pub(super) fn with_levels(&self, levels: Levels) -> Self {
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
    right: f32,
    frame_levels: u32,
    envelope: Option<super::envelope::Envelope>,
}

impl Decodable for StereoPcm {
    type Decoder = Playback;
    fn decoder(&self) -> Playback {
        Playback {
            clip: self.clone(),
            cursor: 0,
            phase: 0,
            right: 0.,
            frame_levels: 0,
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
                    let v = self.envelope.as_mut().unwrap().apply(mono) as f32 * motor.volume;
                    let channels = motor
                        .pan
                        .map(|pan| (v * pan).clamp(-32768., 32767.) / 32768.);
                    self.right = channels[1];
                    self.phase += usize::from(register.get());
                    if self.phase >= motor.pcm.len() * 4096 {
                        self.phase -= (motor.pcm.len() - motor.loop_at) * 4096;
                    }
                    channels[0]
                } else {
                    self.right
                }
            }
        };
        if let Some(levels) = &self.clip.levels {
            // Latch both channels at the start of each stereo frame.
            if self.cursor % 2 == 0 {
                self.frame_levels = levels.0.load(Ordering::Relaxed);
            }
            let gain = (self.frame_levels >> (16 * (self.cursor % 2))) as u16;
            sample *= f32::from(gain) / (33. * 99.);
        }
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
        let clip = StereoPcm::new(&[[8192., 4096.]; 3]);
        let levels = Levels::new([16, 47], 99);
        let mut a = clip.with_levels(levels.clone()).decoder();
        let mut b = clip.with_levels(Levels::new([47, 16], 50)).decoder();
        assert_eq!(a.next(), Some(0.25 * (16. / 33.)));
        levels.set([39, 21], 25);
        assert_eq!(a.next(), Some(0.125 * (47. / 33.)));
        assert_eq!(a.next(), Some(0.25 * (39. * 25. / (33. * 99.))));
        assert_eq!(a.next(), Some(0.125 * (21. * 25. / (33. * 99.))));
        assert_eq!(b.next(), Some(0.25 * (47. * 50. / (33. * 99.))));
        assert_eq!(b.next(), Some(0.125 * (16. * 50. / (33. * 99.))));
        assert_eq!(a.count(), 2);
        assert_eq!(b.count(), 4);
    }
}
