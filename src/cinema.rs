//! SCREEN.c startup cards and STR movies from raw Mode 2 sectors.
use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, AudioSource, Entity, Handle, Image};
use std::{
    ffi::OsString,
    io::Read,
    process::{Child, Command, Stdio},
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

struct Decoder(Child);
impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub struct Cinema {
    pub clip: Clip,
    elapsed: Duration,
    first_tick: bool,
    decoder: Option<Decoder>,
    frame: u32,
    frames: u32,
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
        let mut input = OsString::from("subfile:");
        input.push(&disc.tracks[0].path);
        // psxstr consumes complete raw sectors, including the interleaved XA audio.
        let decoder = || {
            let mut cmd = Command::new("ffmpeg");
            cmd.args([
                "-nostdin",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "psxstr",
                "-start",
            ])
            .arg(start.to_string())
            .arg("-end")
            .arg(end.to_string())
            .arg("-i")
            .arg(&input);
            cmd
        };
        let audio = decoder()
            .args([
                "-map", "0:a:0", "-ar", "44100", "-ac", "2", "-f", "s16le", "pipe:1",
            ])
            .output()
            .context("starting FFmpeg for disc cinema")?;
        ensure!(
            audio.status.success() && !audio.stdout.is_empty(),
            "XA decode failed: {}",
            String::from_utf8_lossy(&audio.stderr)
        );
        let sound = crate::music::stereo_wave(&audio.stdout)?;
        // MOVIE.c's one-frame lookahead stops before displaying the limit frame.
        movie.frames = limit - 1;
        movie.decoder = Some(Decoder(
            decoder()
                .args(["-map", "0:v:0", "-frames:v"])
                .arg(movie.frames.to_string())
                .args(["-pix_fmt", "rgba", "-f", "rawvideo", "pipe:1"])
                .stdout(Stdio::piped())
                .spawn()?,
        ));
        let mut pixels = vec![0; screen::WIDTH * screen::HEIGHT * 4];
        movie.read_frame(&mut pixels)?;
        Ok((movie, pixels, Some(sound)))
    }
    fn read_frame(&mut self, pixels: &mut [u8]) -> Result<()> {
        self.decoder
            .as_mut()
            .unwrap()
            .0
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(pixels)
            .context("incomplete disc cinema frame")
    }
    pub fn advance(
        &mut self,
        dt: Duration,
        select: bool,
        images: &mut Assets<Image>,
        handle: &Handle<Image>,
    ) -> Result<bool> {
        if std::mem::take(&mut self.first_tick) {
            return Ok(false);
        }
        self.elapsed += dt;
        let fields = self.elapsed.as_nanos() * 60 / 1_000_000_000;
        if self.decoder.is_none() {
            return Ok(match self.clip {
                Clip::Presents => fields >= 121 || (fields >= 120 && select),
                _ => fields >= 140,
            });
        }
        let frame = (fields / 4) as u32;
        if frame >= self.frames || (select && (self.clip != Clip::Logo || frame + 2 >= 31)) {
            return Ok(true);
        }
        if frame > self.frame {
            let mut image = images.get_mut(handle).context("missing cinema texture")?;
            let pixels = image.data.as_mut().context("missing movie image")?;
            while self.frame < frame {
                self.read_frame(pixels)?;
                self.frame += 1;
            }
        }
        Ok(false)
    }
}
