use anyhow::{Result, ensure};
use bevy::{
    audio::{AudioSinkPlayback, ChannelCount, Decodable, SampleRate, Source, Volume},
    prelude::*,
};
use std::{
    fs::File,
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tm1_assets::disc::Disc;

const SECTOR_BYTES: usize = 2352;
const CD_START: usize = 150 * SECTOR_BYTES;

#[derive(Asset, TypePath, Clone)]
pub struct CdTrack {
    pcm: Arc<[u8]>,
    normal_end: usize,
    extended: Arc<AtomicBool>,
}

impl CdTrack {
    pub fn load(disc: &Disc, number: usize) -> Result<Self> {
        let mut pcm = disc.audio_track(number)?;
        let next = disc
            .tracks
            .get(number)
            .ok_or_else(|| anyhow::anyhow!("missing next CD track"))?;
        let byte_count = pcm.len();
        ensure!(byte_count > CD_START, "stage track is too short");
        let next_bin_sector: u64 = disc.tracks[..number]
            .iter()
            .map(|t| t.size / SECTOR_BYTES as u64)
            .sum();
        // cbready discards the sector within the reported absolute second and
        // repeats only once that second is strictly beyond EndPos.
        let until_repeat = 75 - next_bin_sector % 75;
        let normal_end = byte_count + until_repeat as usize * SECTOR_BYTES;
        let next_index = next_bin_sector + next.index01;
        let extended_sectors = next.index01 + 75 - next_index % 75;
        pcm.resize(byte_count + extended_sectors as usize * SECTOR_BYTES, 0);
        File::open(&next.path)?.read_exact(&mut pcm[byte_count..])?;
        Ok(Self {
            pcm: pcm.into(),
            normal_end,
            extended: Arc::default(),
        })
    }

    pub fn extend_loop(&self, extended: bool) {
        self.extended.store(extended, Ordering::Relaxed);
    }
}

pub struct CdPlayback {
    track: CdTrack,
    cursor: usize,
}

impl Decodable for CdTrack {
    type Decoder = CdPlayback;
    fn decoder(&self) -> Self::Decoder {
        CdPlayback {
            track: self.clone(),
            cursor: CD_START,
        }
    }
}

impl Iterator for CdPlayback {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        // screenAudioOptions changes EndPos while soundResumePlayDA resumes
        // from cdInterruptLoc. Only the next loop boundary changes.
        if self.cursor == self.track.pcm.len()
            || (self.cursor == self.track.normal_end
                && !self.track.extended.load(Ordering::Relaxed))
        {
            self.cursor = CD_START;
        }
        let pcm = &self.track.pcm[self.cursor..self.cursor + 2];
        self.cursor += 2;
        Some(f32::from(i16::from_le_bytes([pcm[0], pcm[1]])) / 32768.)
    }
}

impl Source for CdPlayback {
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

#[derive(Resource)]
pub struct Music {
    opening: Option<CdTrack>,
    battle: Option<CdTrack>,
}
impl Music {
    pub fn level(disc: &Disc, level: tm1_core::drive::Level) -> Result<Self> {
        // shellSoundForThisLevel selects the stage track. soundStartPlayDA
        // starts 150 sectors after INDEX 01.
        use tm1_core::drive::Level::*;
        let opening = opening_track(level, false);
        let battle = match level {
            Warehouse | Cyburbia => Some(7),
            CityPark => Some(5),
            _ => None,
        };
        Ok(Self {
            opening: Some(CdTrack::load(disc, opening)?),
            battle: battle.map(|n| CdTrack::load(disc, n)).transpose()?,
        })
    }
}
pub fn opening_track(level: tm1_core::drive::Level, split: bool) -> usize {
    use tm1_core::drive::Level::*;
    match level {
        Arena => 3,
        Warehouse | Cyburbia if split => 7,
        Warehouse | CityPark => 4,
        Freeway | Arena5 => 2,
        Cyburbia => 6,
        Rooftops => 8,
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
pub struct ArenaMusic {
    opening: Handle<CdTrack>,
    battle: Option<Handle<CdTrack>>,
    restart: u32,
}
pub fn start(
    mut commands: Commands,
    mut music: ResMut<Music>,
    drive: Res<crate::driving::Drive>,
    mut assets: ResMut<Assets<CdTrack>>,
) {
    if let Some(source) = music.opening.take() {
        let opening = assets.add(source);
        let player = AudioPlayer(opening.clone());
        commands.spawn((
            crate::campaign::StageEntity,
            ArenaMusic {
                opening,
                battle: music.battle.take().map(|s| assets.add(s)),
                restart: drive.restart_count,
            },
            player,
            PlaybackSettings::ONCE.with_volume(Volume::Linear(drive.settings.cd_gain())),
        ));
    }
}
pub fn control(
    mut commands: Commands,
    mut music: Query<(
        Entity,
        &mut ArenaMusic,
        &AudioPlayer<CdTrack>,
        &mut PlaybackSettings,
        Option<&mut AudioSink>,
    )>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    assets: Res<Assets<CdTrack>>,
) {
    let Ok((entity, mut tracks, player, mut settings, sink)) = music.single_mut() else {
        return;
    };
    let volume = Volume::Linear(drive.settings.cd_gain());
    let pause = (drive.pause.is_some()
        && !matches!(drive.pause, Some(crate::pause::Page::Audio(_))))
        || drive.lost_life_elapsed().is_some()
        || drive.lives == 0
        || !window.focused;
    let restart = tracks.restart != drive.restart_count;
    tracks.restart = drive.restart_count;
    let audio_options = matches!(drive.pause, Some(crate::pause::Page::Audio(_)));
    let desired = if drive.contacts.battle_music_playing() {
        tracks.battle.as_ref().unwrap_or(&tracks.opening)
    } else {
        &tracks.opening
    };
    settings.paused = pause;
    settings.volume = volume;
    if restart || player.0 != *desired {
        if let Some(track) = assets.get(desired) {
            track.extend_loop(false);
        }
        // Stage entry and life restarts open a new decoder at the CD start sector.
        commands
            .entity(entity)
            .remove::<AudioSink>()
            .insert(AudioPlayer(desired.clone()));
        return;
    }
    if audio_options && let Some(track) = assets.get(desired) {
        track.extend_loop(true);
    }
    let Some(mut sink) = sink else { return };
    sink.set_volume(volume);
    if pause != sink.is_paused() {
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn stage_cd_loops_follow_retail_audio_options() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let mut sector = 0;
        for track in &disc.tracks {
            cpu.call(
                0x80156f80,
                &[
                    sector + track.index01 as u32,
                    0x801961e8 + 4 * track.number as u32,
                ],
            );
            sector += (track.size / SECTOR_BYTES as u64) as u32;
        }
        for addr in [0x80156540, 0x801632b8] {
            cpu.put(addr, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        for number in 2..=8 {
            let track = CdTrack::load(&disc, number).unwrap();
            cpu.put(0x8018bf68, &0u32.to_le_bytes());
            cpu.call_until(0x801057e0, &[number as u32], 0x80105b60);
            let (start, end) = (cpu.r[4], cpu.r[5]);
            let track_start = start - 150;
            let repeat = track_start + (track.normal_end / SECTOR_BYTES) as u32;
            cpu.put(0x8018bf68, &4u32.to_le_bytes());
            cpu.call_until(0x801057e0, &[number as u32], 0x80105a1c);
            assert_eq!(start, cpu.word(0x8018c4e4));
            let extended_end = cpu.word(0x8018c4e8);
            let extended_repeat = track_start + (track.pcm.len() / SECTOR_BYTES) as u32;
            for (limit, wrap) in [(end, repeat), (extended_end, extended_repeat)] {
                cpu.put(0x8018c4e8, &limit.to_le_bytes());
                for position in [wrap - 1, wrap] {
                    let report = 0x80010000;
                    // CdIntToPos writes minute, second and sector at report[3..6].
                    cpu.call(0x80156f80, &[position, report + 3]);
                    if position < wrap {
                        cpu.call(0x80105c78, &[1, report]);
                        assert!(cpu.word(0x8018c4ec) <= limit);
                    } else {
                        cpu.call_until(0x80105c78, &[1, report], 0x80105cec);
                        assert!(cpu.word(0x8018c4ec) > limit);
                    }
                }
            }

            let mut playback = track.decoder();
            playback.cursor = track.normal_end - 2;
            playback.next();
            playback.next();
            assert_eq!(playback.cursor, CD_START + 2);
            playback.cursor = track.normal_end - 2;
            track.extend_loop(true);
            assert_eq!(playback.cursor, track.normal_end - 2);
            playback.next();
            let mut next_bin = File::open(&disc.tracks[number].path).unwrap();
            use std::io::{Seek, SeekFrom};
            let bin_end = disc.tracks[number - 1].size as usize
                - disc.tracks[number - 1].index01 as usize * SECTOR_BYTES;
            next_bin
                .seek(SeekFrom::Start((track.normal_end - bin_end) as u64))
                .unwrap();
            let mut extension = vec![0; track.pcm.len() - track.normal_end];
            next_bin.read_exact(&mut extension).unwrap();
            for bytes in extension.chunks_exact(2) {
                assert_eq!(
                    playback.next(),
                    Some(f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.)
                );
            }
            assert_eq!(playback.cursor, track.pcm.len());
            playback.next();
            assert_eq!(playback.cursor, CD_START + 2);
        }
    }
}
