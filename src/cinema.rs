//! SCREEN.c startup cards and STR movies from raw Mode 2 sectors.
mod video;
mod xa;
use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, AudioSource, Entity, Handle, Image};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    time::Duration,
};
use tm1_assets::{disc::Disc, screen};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Presents,
    Production,
    Developer,
    Logo,
    Title,
    Attract,
}
impl Clip {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Presents => Some(Self::Production),
            Self::Production => Some(Self::Developer),
            Self::Developer => Some(Self::Logo),
            Self::Logo | Self::Attract => Some(Self::Title),
            Self::Title => None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum CinemaStep {
    Playing,
    Finished,
    Skipped,
}

pub struct Cinema {
    pub clip: Clip,
    elapsed: Duration,
    first_tick: bool,
    decoder: Option<video::Decoder>,
    frame: u32,
    frames: u32,
    last_callback_frame: Option<u32>,
    pub voice: Option<Entity>,
}
impl Cinema {
    pub fn load(disc: &Disc, clip: Clip) -> Result<(Self, Vec<u8>, Option<AudioSource>)> {
        let mut movie = Self {
            clip,
            elapsed: Duration::ZERO,
            first_tick: true,
            decoder: None,
            frame: 0,
            frames: 0,
            last_callback_frame: None,
            voice: None,
        };
        let (name, limit) = match clip {
            Clip::Logo => ("MOVIES/SLOGO15S.STR", 149),
            Clip::Title => ("UAMOVIES/TMTTL15S.STR", 67),
            Clip::Attract => ("UAMOVIES/TMDMO15S.STR", 1000),
            card => {
                let path = match card {
                    Clip::Presents => "UASCREEN/SCEAPRES.TIM",
                    Clip::Production => "UASCREEN/SISAPROD.TIM",
                    Clip::Developer => "UASCREEN/DEVELOP.TIM",
                    _ => unreachable!(),
                };
                return Ok((movie, screen::background(&disc.read(path)?)?, None));
            }
        };
        let entry = disc.files.get(name).context("missing cinema in ISO")?;
        let start = u64::from(entry.sector) * 2352;
        let end = start + u64::from(entry.size.div_ceil(2048)) * 2352;
        ensure!(end <= disc.tracks[0].size, "invalid cinema extent");
        let mut track = BufReader::new(std::fs::File::open(&disc.tracks[0].path)?);
        track.seek(SeekFrom::Start(start))?;
        let audio = xa::decode(&mut track, (end - start) / 2352)?;
        let sound = crate::music::stereo_wave_at_rate(&audio, xa::RATE)?;
        // MOVIE.c's one-frame lookahead stops before displaying the limit frame.
        movie.frames = limit - 1;
        track.seek(SeekFrom::Start(start))?;
        movie.decoder = Some(video::Decoder::new(track.take(end - start)));
        let mut pixels = vec![0; screen::WIDTH * screen::HEIGHT * 4];
        movie.read_frame(&mut pixels)?;
        Ok((movie, pixels, Some(sound)))
    }
    fn read_frame(&mut self, pixels: &mut [u8]) -> Result<()> {
        self.decoder
            .as_mut()
            .context("missing STR decoder")?
            .read_frame(pixels)
    }
    pub fn advance(
        &mut self,
        dt: Duration,
        select: bool,
        images: &mut Assets<Image>,
        handle: &Handle<Image>,
    ) -> Result<CinemaStep> {
        if std::mem::take(&mut self.first_tick) {
            return Ok(CinemaStep::Playing);
        }
        self.elapsed += dt;
        let fields = self.elapsed.as_nanos() * 60 / 1_000_000_000;
        if self.decoder.is_none() {
            let finished = match self.clip {
                Clip::Presents => fields >= 121 || (fields >= 120 && select),
                _ => fields >= 140,
            };
            return Ok(if finished {
                CinemaStep::Finished
            } else {
                CinemaStep::Playing
            });
        }
        let frame = (fields / 4) as u32;
        if frame >= self.frames {
            return Ok(CinemaStep::Finished);
        }
        // moviecdPlayMovie polls after displaying a frame and reading its
        // successor. Reaching the stream limit bypasses the final callback.
        let poll =
            self.last_callback_frame.replace(frame) != Some(frame) && frame + 1 < self.frames;
        if poll && select && (self.clip != Clip::Logo || frame + 2 >= 31) {
            return Ok(CinemaStep::Skipped);
        }
        if frame > self.frame {
            let mut image = images.get_mut(handle).context("missing cinema texture")?;
            let pixels = image.data.as_mut().context("missing movie image")?;
            while self.frame < frame {
                self.read_frame(pixels)?;
                self.frame += 1;
            }
        }
        Ok(CinemaStep::Playing)
    }
}
