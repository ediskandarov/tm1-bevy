use anyhow::{Result, ensure};
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
};
use std::time::Duration;
use tm1_assets::disc::Disc;

#[derive(Resource)]
pub struct Music(Option<AudioSource>);
impl Music {
    pub fn level(disc: &Disc, level: tm1_core::drive::Level) -> Result<Self> {
        // shellSoundForThisLevel selects the stage track. soundStartPlayDA
        // starts 150 sectors after INDEX 01 and ends before the next pregap.
        let track = disc.audio_track(match level {
            tm1_core::drive::Level::Freeway => 2,
            tm1_core::drive::Level::Arena => 3,
            tm1_core::drive::Level::Warehouse | tm1_core::drive::Level::CityPark => 4,
        })?;
        let pcm = &track[150 * 2352..];
        Ok(Self(Some(stereo_wave(pcm)?)))
    }
}
pub fn stereo_wave(pcm: &[u8]) -> Result<AudioSource> {
    ensure!(pcm.len() % 4 == 0, "incomplete stereo CD frame");
    let size = u32::try_from(pcm.len())?;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend(b"RIFF");
    wav.extend((36 + size).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(44100u32.to_le_bytes());
    wav.extend(176400u32.to_le_bytes());
    wav.extend(4u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(size.to_le_bytes());
    wav.extend(pcm);
    Ok(AudioSource { bytes: wav.into() })
}
#[derive(Component)]
pub struct ArenaMusic;
pub fn start(
    mut commands: Commands,
    mut music: ResMut<Music>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    if let Some(source) = music.0.take() {
        commands.spawn((
            ArenaMusic,
            AudioPlayer::new(assets.add(source)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.35)),
        ));
    }
}
pub fn control(
    sink: Query<&AudioSink, With<ArenaMusic>>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    mut last_restart: Local<u32>,
) {
    let Ok(sink) = sink.single() else { return };
    if *last_restart != drive.restart_count {
        if let Err(err) = sink.try_seek(Duration::ZERO) {
            warn!("Music restart failed: {err}");
        }
        *last_restart = drive.restart_count;
    }
    let pause = drive.paused || !window.focused;
    if pause != sink.is_paused() {
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
}
