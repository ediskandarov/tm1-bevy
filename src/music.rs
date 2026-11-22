use anyhow::{Result, ensure};
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
};
use std::time::Duration;
use tm1_assets::disc::Disc;

#[derive(Resource)]
pub struct Music {
    opening: Option<AudioSource>,
    battle: Option<AudioSource>,
}
impl Music {
    pub fn level(disc: &Disc, level: tm1_core::drive::Level) -> Result<Self> {
        // shellSoundForThisLevel selects the stage track. soundStartPlayDA
        // starts 150 sectors after INDEX 01 and ends before the next pregap.
        use tm1_core::drive::Level::*;
        let (opening, battle) = match level {
            Rooftops => (8, None),
            Cyburbia => (6, Some(7)),
            Freeway | Arena5 => (2, None),
            Arena => (3, None),
            Warehouse => (4, Some(7)),
            CityPark => (4, Some(5)),
        };
        Ok(Self {
            opening: Some(track(disc, opening)?),
            battle: battle.map(|n| track(disc, n)).transpose()?,
        })
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
pub struct ArenaMusic {
    opening: Handle<AudioSource>,
    battle: Option<Handle<AudioSource>>,
    restart: u32,
}
pub fn start(
    mut commands: Commands,
    mut music: ResMut<Music>,
    drive: Res<crate::driving::Drive>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    if let Some(source) = music.opening.take() {
        let opening = assets.add(source);
        commands.spawn((
            crate::campaign::StageEntity,
            ArenaMusic {
                opening: opening.clone(),
                battle: music.battle.take().map(|s| assets.add(s)),
                restart: drive.restart_count,
            },
            AudioPlayer::new(opening),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.35 * drive.settings.music_gain())),
        ));
    }
}
pub fn control(
    mut commands: Commands,
    mut music: Query<(
        Entity,
        &mut ArenaMusic,
        &AudioPlayer,
        Option<&mut AudioSink>,
    )>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
) {
    let Ok((entity, mut tracks, player, sink)) = music.single_mut() else {
        return;
    };
    let volume = Volume::Linear(0.35 * drive.settings.music_gain());
    let pause = (drive.pause.is_some()
        && !matches!(drive.pause, Some(crate::pause::Page::Audio(_))))
        || drive.lives == 0
        || !window.focused;
    let restart = tracks.restart != drive.restart_count;
    tracks.restart = drive.restart_count;
    let desired = if drive.contacts.battle_music_playing() {
        tracks.battle.as_ref().unwrap_or(&tracks.opening)
    } else {
        &tracks.opening
    };
    if player.0 != *desired {
        let settings = PlaybackSettings::LOOP.with_volume(volume);
        commands.entity(entity).remove::<AudioSink>().insert((
            AudioPlayer::new(desired.clone()),
            if pause { settings.paused() } else { settings },
        ));
        return;
    }
    let Some(mut sink) = sink else { return };
    if restart && let Err(err) = sink.try_seek(Duration::ZERO) {
        warn!("Music restart failed: {err}");
    }
    sink.set_volume(volume);
    if pause != sink.is_paused() {
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
}
