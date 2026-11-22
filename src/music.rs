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
        let track = match level {
            tm1_core::drive::Level::Rooftops => 8,
            tm1_core::drive::Level::Cyburbia => 6,
            tm1_core::drive::Level::Freeway | tm1_core::drive::Level::Arena5 => 2,
            tm1_core::drive::Level::Arena => 3,
            tm1_core::drive::Level::Warehouse | tm1_core::drive::Level::CityPark => 4,
        };
        Ok(Self(Some(self::track(disc, track)?)))
    }
}
pub fn track(disc: &Disc, number: usize) -> Result<AudioSource> {
    let track = disc.audio_track(number)?;
    stereo_wave(&track[150 * 2352..])
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
    drive: Res<crate::driving::Drive>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    if let Some(source) = music.0.take() {
        commands.spawn((
            crate::campaign::StageEntity,
            ArenaMusic,
            AudioPlayer::new(assets.add(source)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.35 * drive.settings.music_gain())),
        ));
    }
}
pub fn control(
    mut sink: Query<&mut AudioSink, With<ArenaMusic>>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    mut last_restart: Local<u32>,
) {
    let Ok(mut sink) = sink.single_mut() else {
        return;
    };
    if *last_restart != drive.restart_count {
        if let Err(err) = sink.try_seek(Duration::ZERO) {
            warn!("Music restart failed: {err}");
        }
        *last_restart = drive.restart_count;
    }
    sink.set_volume(Volume::Linear(0.35 * drive.settings.music_gain()));
    let pause = (drive.pause.is_some()
        && !matches!(drive.pause, Some(crate::pause::Page::Audio(_))))
        || drive.lives == 0
        || !window.focused;
    if pause != sink.is_paused() {
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
}
