//! Shared VAB samples with per-voice stereo levels and optional ADPCM loop points.
use bevy::{
    audio::{ChannelCount, Decodable, SampleRate, Source},
    prelude::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
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

#[derive(Asset, TypePath, Clone)]
pub struct StereoPcm {
    samples: Arc<[f32]>,
    loop_sample: Option<usize>,
    levels: Option<Levels>,
}

impl StereoPcm {
    pub(super) fn new(stereo: &[[f32; 2]], first_loop_frame: Option<usize>) -> Self {
        assert!(first_loop_frame.is_none_or(|frame| frame < stereo.len()));
        Self {
            samples: stereo
                .iter()
                .flatten()
                .map(|s| s.clamp(-32768., 32767.) / 32768.)
                .collect(),
            loop_sample: first_loop_frame.map(|frame| 2 * frame),
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
    frame_levels: u32,
}

impl Decodable for StereoPcm {
    type Decoder = Playback;
    fn decoder(&self) -> Playback {
        Playback {
            clip: self.clone(),
            cursor: 0,
            frame_levels: 0,
        }
    }
}
impl Iterator for Playback {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == self.clip.samples.len() {
            self.cursor = self.clip.loop_sample?;
        }
        let mut sample = self.clip.samples[self.cursor];
        if let Some(levels) = &self.clip.levels {
            // Latch both channels at the start of each stereo frame.
            if self.cursor % 2 == 0 {
                self.frame_levels = levels.0.load(Ordering::Relaxed);
            }
            let gain = (self.frame_levels >> (16 * (self.cursor % 2))) as u16;
            sample *= f32::from(gain) / (33. * 99.);
        }
        self.cursor += 1;
        Some(sample)
    }
}
impl Source for Playback {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn total_duration(&self) -> Option<Duration> {
        self.clip
            .loop_sample
            .is_none()
            .then(|| Duration::from_secs_f64(self.clip.samples.len() as f64 / 88200.))
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
        let clip = StereoPcm::new(&[[8192., 4096.]; 3], None);
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

    #[test]
    fn attack_is_played_once_and_loop_is_stereo_aligned() {
        let clip = StereoPcm::new(&[[0., 1.], [2., 3.], [4., 5.]], Some(1));
        let samples: Vec<_> = clip
            .decoder()
            .take(12)
            .map(|v| (v * 32768.) as i16)
            .collect();
        assert_eq!(samples, [0, 1, 2, 3, 4, 5, 2, 3, 4, 5, 2, 3]);
    }
}
