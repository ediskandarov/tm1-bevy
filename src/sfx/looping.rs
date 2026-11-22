//! Play the attack once, then repeat the stereo PCM at the VAB loop marker.
use bevy::{
    audio::{ChannelCount, Decodable, SampleRate, Source},
    prelude::*,
};
use std::{sync::Arc, time::Duration};

#[derive(Asset, TypePath, Clone)]
pub struct LoopingPcm {
    samples: Arc<[f32]>,
    loop_sample: usize,
}

impl LoopingPcm {
    pub(super) fn new(stereo: &[[f32; 2]], first_loop_frame: usize) -> Self {
        assert!(first_loop_frame < stereo.len());
        Self {
            samples: stereo
                .iter()
                .flatten()
                .map(|s| s.clamp(-32768., 32767.) / 32768.)
                .collect(),
            loop_sample: 2 * first_loop_frame,
        }
    }
}

pub struct Playback {
    clip: LoopingPcm,
    cursor: usize,
}

impl Decodable for LoopingPcm {
    type Decoder = Playback;
    fn decoder(&self) -> Playback {
        Playback {
            clip: self.clone(),
            cursor: 0,
        }
    }
}
impl Iterator for Playback {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.clip.samples[self.cursor];
        self.cursor += 1;
        if self.cursor == self.clip.samples.len() {
            self.cursor = self.clip.loop_sample;
        }
        Some(sample)
    }
}
impl Source for Playback {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn total_duration(&self) -> Option<Duration> {
        None
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
    fn attack_is_played_once_and_loop_is_stereo_aligned() {
        let clip = LoopingPcm::new(&[[0., 1.], [2., 3.], [4., 5.]], 1);
        let samples: Vec<_> = clip
            .decoder()
            .take(12)
            .map(|v| (v * 32768.) as i16)
            .collect();
        assert_eq!(samples, [0, 1, 2, 3, 4, 5, 2, 3, 4, 5, 2, 3]);
    }
}
