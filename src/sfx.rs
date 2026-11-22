pub mod duel;
pub mod looping;
use anyhow::Result;
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
};
use tm1_assets::{disc::Disc, sound::program};

#[derive(Resource)]
pub struct Sounds {
    sources: Vec<AudioSource>,
    turbo: Vec<PreparedTone>,
    guns: Vec<Vec<PreparedTone>>,
    rival_engines: Vec<(AudioSource, f32, std::time::Duration)>,
    voices: Vec<Handle<AudioSource>>,
    engine_range: f32,
    engine_loop: std::time::Duration,
}

pub struct Sample {
    mix: Vec<[f32; 2]>,
    pub range: f32,
    loop_start: std::time::Duration,
}
impl Sample {
    pub fn load(bank: &[u8], prog: usize, engine: bool) -> Result<Self> {
        Ok(Self::from_tones(program(bank, prog)?, engine))
    }
    fn from_tones(tones: Vec<tm1_assets::sound::Tone>, engine: bool) -> Self {
        let mut engine_range = 0.;
        let mut engine_loop = std::time::Duration::ZERO;
        let mut mix = Vec::<[f32; 2]>::new();
        for t in tones {
            let note = if engine {
                engine_range = f32::from(t.high) - f32::from(t.low);
                t.low
            } else {
                60
            };
            let rate = 2f32.powf((f32::from(note) - f32::from(t.center)) / 12.);
            if engine {
                engine_loop = std::time::Duration::from_secs_f64(
                    t.loop_start.unwrap_or(0) as f64 / (44100. * f64::from(rate)),
                );
            }
            let n = (t.pcm.len() as f32 / rate).ceil() as usize;
            mix.resize(mix.len().max(n), [0.; 2]);
            let pan = f32::from(t.pan) / 64.;
            let gain = f32::from(t.volume) / 127.;
            for (i, out) in mix[..n].iter_mut().enumerate() {
                let at = (i as f32 * rate) as usize;
                let v = t.pcm[at.min(t.pcm.len() - 1)] as f32 * gain;
                out[0] += v * (2. - pan).min(1.);
                out[1] += v * pan.min(1.);
            }
        }
        Self {
            mix,
            range: engine_range,
            loop_start: engine_loop,
        }
    }
    pub fn wave(&self, pan: f32) -> Result<AudioSource> {
        let pcm: Vec<_> = self
            .mix
            .iter()
            .flat_map(|[l, r]| [l * (1. - pan.max(0.)), r * (1. + pan.min(0.))])
            .flat_map(|v| (v.clamp(-32768., 32767.) as i16).to_le_bytes())
            .collect();
        crate::music::stereo_wave(&pcm)
    }
}

// Program 26 is the four-tone sound at ID 0x82 in the JP sound table.
const TURBO_PROGRAM: usize = 26;

struct PreparedTone {
    wave: AudioSource,
    looping_clip: Option<looping::LoopingPcm>,
    priority: u8,
    range: f32,
    loop_start: std::time::Duration,
}

struct Layer {
    source: Handle<AudioSource>,
    looping_clip: Option<Handle<looping::LoopingPcm>>,
    priority: u8,
    range: f32,
    loop_start: std::time::Duration,
}

impl PreparedTone {
    fn new(tone: tm1_assets::sound::Tone, engine: bool, looping: bool) -> Result<Self> {
        Self::panned(tone, engine, looping, 0.)
    }
    fn panned(
        tone: tm1_assets::sound::Tone,
        engine: bool,
        looping: bool,
        pan: f32,
    ) -> Result<Self> {
        let priority = tone.priority;
        let first_loop_frame = tone.loop_start.filter(|_| looping).map(|frame| {
            let note = if engine { tone.low } else { 60 };
            let rate = 2.0_f32.powf((f32::from(note) - f32::from(tone.center)) / 12.);
            (frame as f64 / f64::from(rate)).ceil() as usize
        });
        let mut sample = Sample::from_tones(vec![tone], engine);
        if pan != 0. {
            for [l, r] in &mut sample.mix {
                *l *= 1. - pan.max(0.);
                *r *= 1. + pan.min(0.);
            }
        }
        Ok(Self {
            wave: sample.wave(0.)?,
            looping_clip: first_loop_frame
                .map(|frame| looping::LoopingPcm::new(&sample.mix, frame)),
            priority,
            range: sample.range,
            loop_start: sample.loop_start,
        })
    }
    fn register(self, world: &mut World) -> Layer {
        Layer {
            source: world.resource_mut::<Assets<AudioSource>>().add(self.wave),
            looping_clip: self.looping_clip.map(|clip| {
                world
                    .resource_mut::<Assets<looping::LoopingPcm>>()
                    .add(clip)
            }),
            priority: self.priority,
            range: self.range,
            loop_start: self.loop_start,
        }
    }
}
impl Layer {
    fn play(&self, entity: &mut bevy::ecs::system::EntityCommands) {
        if let Some(clip) = &self.looping_clip {
            entity.insert(AudioPlayer(clip.clone()));
        } else {
            entity.insert(AudioPlayer::new(self.source.clone()));
        }
    }
}
#[derive(Resource)]
pub struct VehicleSounds {
    turbo: Vec<Layer>,
    guns: Vec<Vec<Layer>>,
}

pub(crate) fn engine_speed(range: f32, rev: i32) -> f32 {
    let pitch_steps = (range as i32 * 128 * rev.clamp(0, 99)) / 99;
    2f32.powf(pitch_steps as f32 / (128. * 12.))
}

fn battle_programs(
    vehicle: tm1_core::drive::Vehicle,
    opponents: &[tm1_core::drive::Vehicle],
) -> impl Iterator<Item = usize> + '_ {
    use tm1_core::drive::Vehicle::SweetTooth;
    let (engine, special) = vehicle.sound_programs();
    [engine, 12, 13, 29, 19, special, 21, 16, 15, 18, 17, 21]
        .into_iter()
        .chain(opponents.iter().map(|&car| car.sound_programs().1))
        .chain([52, 53, 27, 38, 36])
        .chain(tm1_core::drive::ContactVoice::PROGRAMS)
        .chain([28, SweetTooth.sound_programs().1, 57, 58])
}

impl Sounds {
    pub fn install_vehicle_sounds(&mut self, world: &mut World) {
        world.init_resource::<Assets<AudioSource>>();
        world.init_resource::<Assets<looping::LoopingPcm>>();
        let turbo = self
            .turbo
            .drain(..)
            .map(|tone| tone.register(world))
            .collect();
        let guns = self
            .guns
            .drain(..)
            .map(|tones| tones.into_iter().map(|tone| tone.register(world)).collect())
            .collect();
        world.insert_resource(VehicleSounds { turbo, guns });
    }

    fn register(&mut self, assets: &mut Assets<AudioSource>) {
        self.voices = std::mem::take(&mut self.sources)
            .into_iter()
            .map(|s| assets.add(s))
            .collect();
    }
    pub fn menu_voice(&self, select: bool) -> AudioPlayer {
        self.centered(self.voices.len() / 5 - 2 + usize::from(select))
    }
    pub fn centered(&self, id: usize) -> AudioPlayer {
        AudioPlayer::new(self.voices[id * 5 + 2].clone())
    }
    pub fn ending_engine(&self, gain: f32) -> (AudioPlayer, PlaybackSettings) {
        (
            AudioPlayer::new(self.voices[2].clone()),
            PlaybackSettings::LOOP
                .with_speed(engine_speed(self.engine_range, 30))
                .with_volume(Volume::Linear(0.28 * gain)),
        )
    }

    pub fn signature(&self) -> AudioPlayer {
        AudioPlayer::new(self.voices[5 * 5 + 2].clone())
    }

    pub fn lost_game_signature(&self) -> AudioPlayer {
        let program = self.voices.len() / 5 - 3;
        AudioPlayer::new(self.voices[program * 5 + 2].clone())
    }

    pub fn load(
        disc: &Disc,
        vehicle: tm1_core::drive::Vehicle,
        opponents: &[tm1_core::drive::Vehicle],
    ) -> Result<Self> {
        let bank = disc.read("SND/UACORE.VAB")?;
        let mut sources = Vec::new();
        let mut engine_range = 0.;
        let mut engine_loop = std::time::Duration::ZERO;
        for (id, prog) in battle_programs(vehicle, opponents).enumerate() {
            let sample = Sample::load(&bank, prog, id == 0)?;
            if id == 0 {
                engine_range = sample.range;
                engine_loop = sample.loop_start;
            }
            // Keep VAB stereo and quantize the listener-relative direction to five pan positions.
            for pan in [-1_f32, -0.5, 0., 0.5, 1.] {
                sources.push(sample.wave(pan)?);
            }
        }
        let mut rival_engines = Vec::with_capacity(opponents.len());
        for car in opponents {
            let sample = Sample::load(&bank, car.sound_programs().0, true)?;
            rival_engines.push((sample.wave(0.)?, sample.range, sample.loop_start));
        }
        Ok(Self {
            sources,
            guns: [-1., -0.5, 0., 0.5, 1.]
                .into_iter()
                .map(|pan| {
                    program(&bank, 12)?
                        .into_iter()
                        .map(|tone| PreparedTone::panned(tone, false, true, pan))
                        .collect::<Result<_>>()
                })
                .collect::<Result<_>>()?,
            turbo: program(&bank, TURBO_PROGRAM)?
                .into_iter()
                .map(|tone| PreparedTone::new(tone, false, true))
                .collect::<Result<_>>()?,
            rival_engines,
            voices: Vec::new(),
            engine_range,
            engine_loop,
        })
    }
}
#[derive(Component)]
pub struct Engine;
#[derive(Component)]
pub struct RivalEngine {
    slot: usize,
    source: Handle<AudioSource>,
    range: f32,
    loop_start: std::time::Duration,
}
#[derive(Component)]
pub struct OneShot(Option<crate::driving::ExclusiveVoice>);
pub fn start(
    mut commands: Commands,
    mut sounds: ResMut<Sounds>,
    drive: Res<crate::driving::Drive>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    sounds.register(&mut assets);
    commands.spawn((crate::campaign::StageEntity, SpatialListener::new(1.0)));
    for (slot, (source, range, loop_start)) in sounds.rival_engines.drain(..).enumerate() {
        commands.spawn((
            crate::campaign::StageEntity,
            Transform::default(),
            RivalEngine {
                slot,
                source: assets.add(source),
                range,
                loop_start,
            },
        ));
    }
    commands.spawn((
        crate::campaign::StageEntity,
        Engine,
        AudioPlayer::new(sounds.voices[2].clone()),
        PlaybackSettings::LOOP
            .with_start_position(sounds.engine_loop)
            .with_volume(Volume::Linear(0.28 * drive.settings.effects_gain())),
    ));
}
pub fn update(
    mut commands: Commands,
    sounds: Res<Sounds>,
    vehicle_sounds: Res<VehicleSounds>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    mut engine: Query<&mut AudioSink, With<Engine>>,
    mut shots: Query<(Entity, Option<&mut AudioSink>, &OneShot), Without<Engine>>,
    mut rivals: Query<(
        Entity,
        &RivalEngine,
        &mut Transform,
        Has<AudioPlayer>,
        Option<&mut SpatialAudioSink>,
    )>,
    mut restart: Local<u32>,
) {
    let reset = *restart != drive.restart_count;
    let pause = drive.lives == 0 || drive.pause.is_some() || !window.focused;
    let gain = drive.settings.effects_gain();
    if let Ok(mut sink) = engine.single_mut() {
        sink.set_volume(Volume::Linear(0.28 * gain));
        if reset {
            let _ = sink.try_seek(sounds.engine_loop);
        }
        sink.set_speed(engine_speed(sounds.engine_range, drive.player.rev()));
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
    let heading = drive.player.heading() as f32 * std::f32::consts::TAU / 4096.;
    let listener = drive.player.position();
    let right = Vec3::new(heading.cos(), -heading.sin(), 0.);
    let mix_position = |point: [i32; 3]| {
        let offset = Vec3::from_array(std::array::from_fn(|i| (point[i] - listener[i]) as f32));
        let distance = offset.length();
        let pan = (offset.dot(right) / distance.max(100.)).clamp(-1., 1.);
        (pan, (1. - distance / 6000.).clamp(0., 1.))
    };
    let occupied = shots.iter().count()
        + rivals
            .iter()
            .filter(|(_, _, _, playing, _)| *playing)
            .count();
    let mut free_engine_slots = 22usize.saturating_sub(occupied);
    let mut active_engines = 0;
    for (entity, engine, mut transform, playing, sink) in &mut rivals {
        let audible = drive.opponents.get(engine.slot).and_then(|ai| {
            let rev = ai.engine_rev()?;
            let (pan, attenuation) = mix_position(ai.position());
            (attenuation > 0.).then_some((rev, 0.28 * gain * attenuation, pan))
        });
        let Some((rev, volume, pan)) = audible else {
            if playing {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer, SpatialAudioSink)>();
            }
            continue;
        };
        if !playing {
            if free_engine_slots == 0 {
                continue;
            }
            free_engine_slots -= 1;
        }
        active_engines += 1;
        transform.translation = Vec3::X * pan;
        let speed = engine_speed(engine.range, rev);
        if let Some(mut sink) = sink {
            sink.set_speed(speed);
            sink.set_volume(Volume::Linear(volume));
            if reset {
                let _ = sink.try_seek(std::time::Duration::ZERO);
            }
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        } else if !playing {
            let mut settings = PlaybackSettings::LOOP
                .with_spatial(true)
                .with_start_position(engine.loop_start)
                .with_speed(speed)
                .with_volume(Volume::Linear(volume));
            settings.paused = pause;
            commands
                .entity(entity)
                .insert((AudioPlayer::new(engine.source.clone()), settings));
        }
    }
    *restart = drive.restart_count;
    for (e, sink, _) in &mut shots {
        if reset || drive.pause.is_some() {
            commands.entity(e).despawn();
        } else if let Some(sink) = sink {
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
    if let Some(select) = drive.pause_sound {
        commands.spawn((
            crate::campaign::StageEntity,
            sounds.menu_voice(select),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.44 * gain)),
        ));
    }

    if drive.vehicle_sounds.is_empty() && drive.sounds.is_empty() {
        return;
    }
    let mut remaining = 22usize.saturating_sub(shots.iter().count() + active_engines);
    let mut active: Vec<_> = shots
        .iter()
        .filter_map(|(entity, _, shot)| shot.0.filter(|_| !reset).map(|key| (key, entity)))
        .collect();
    use crate::driving::{ExclusiveVoice, VehicleSound};
    for request in &drive.vehicle_sounds {
        let (key, playing, layers, attenuation) = match *request {
            VehicleSound::Turbo(play) => (ExclusiveVoice::Turbo, play, &vehicle_sounds.turbo, 1.),
            VehicleSound::Guns {
                car,
                position,
                firing,
            } => {
                let (pan, attenuation) = mix_position(position);
                let bin = ((pan * 2.).round() as i32 + 2) as usize;
                (
                    ExclusiveVoice::MachineGuns(car),
                    firing,
                    &vehicle_sounds.guns[bin],
                    attenuation,
                )
            }
        };
        if playing {
            if attenuation <= 0. {
                continue;
            }
            if active.iter().any(|&(group, _)| group == key) {
                continue;
            }
            for tone in layers.iter().take(remaining) {
                let mut voice = commands.spawn((
                    crate::campaign::StageEntity,
                    OneShot(Some(key)),
                    PlaybackSettings::DESPAWN
                        .with_volume(Volume::Linear(0.44 * attenuation * gain)),
                ));
                tone.play(&mut voice);
                active.push((key, voice.id()));
                remaining -= 1;
            }
        } else {
            active.retain(|&(group, entity)| {
                if group == key {
                    commands.entity(entity).despawn();
                    remaining += 1;
                }
                group != key
            });
        }
    }
    for sound in &drive.sounds {
        let id = sound.id;
        let (pan, attenuation) = mix_position(sound.position);
        let bin = (pan * 2.).round() as i32 + 2;
        let attenuation = if sound.full_range { 1. } else { attenuation };
        let volume = attenuation * 0.44 * sound.volume_percent.max(0) as f32 / 99.;
        let level = Volume::Linear(volume * gain);
        let playback = || {
            (
                AudioPlayer::new(sounds.voices[id * 5 + bin as usize].clone()),
                PlaybackSettings::DESPAWN.with_volume(level),
            )
        };
        if let Some(key) = sound.exclusive
            && let Some(&(_, entity)) = active.iter().find(|(active_key, _)| *active_key == key)
        {
            if matches!(
                key,
                crate::driving::ExclusiveVoice::Contact(_)
                    | crate::driving::ExclusiveVoice::ScaledEffect(_)
            ) {
                if let Ok((_, Some(mut sink), _)) = shots.get_mut(entity) {
                    sink.set_volume(level);
                } else {
                    commands.entity(entity).insert(playback());
                }
            }
            continue;
        }
        if volume > 0. && remaining > 0 {
            remaining -= 1;
            let entity = commands
                .spawn((
                    crate::campaign::StageEntity,
                    OneShot(sound.exclusive),
                    playback(),
                ))
                .id();
            if let Some(key) = sound.exclusive {
                active.push((key, entity));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use tm1_core::drive::{Difficulty, Level, Vehicle};

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_machine_guns_release_after_cooldown_and_overheat() {
        use crate::campaign::{Campaign, Entry, Stage};
        let mut world = crate::contest::menu_world();
        let car = Vehicle::RoadKill;
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            car,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(50));
        let step = |world: &mut World, held| {
            let mut keys = ButtonInput::default();
            if held {
                keys.press(KeyCode::Space);
            }
            world.insert_resource(keys);
            world.run_system_once(crate::driving::advance).unwrap();
            world.run_system_once(update).unwrap();
            let mut voices = world
                .query::<(Entity, &OneShot)>()
                .iter(world)
                .filter(|(_, shot)| {
                    shot.0 == Some(crate::driving::ExclusiveVoice::MachineGuns(car))
                })
                .map(|(entity, _)| entity)
                .collect::<Vec<_>>();
            voices.sort();
            voices
        };
        step(&mut world, false);
        let first = step(&mut world, true);
        assert_eq!(first.len(), 2);
        for _ in 0..3 {
            assert_eq!(step(&mut world, false), first);
        }
        assert!(step(&mut world, false).is_empty());
        let second = step(&mut world, true);
        assert_eq!(second.len(), 2);
        for _ in 0..10 {
            assert_eq!(step(&mut world, true), second);
        }
        for _ in 0..200 {
            if step(&mut world, true).is_empty()
                && world
                    .resource::<crate::driving::Drive>()
                    .player
                    .gun_overheated()
            {
                return;
            }
        }
        panic!("expected the machine gun voices to stop during overheat");
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn turbo_tones_follow_fuel_and_duel_slot_order() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::ExclusiveVoice::Turbo;
        use KeyCode::*;
        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let record = 2048 + (0x8016fefc - base) as usize + 0x82 * 6;
        assert_eq!(&exe[record..record + 4], &[0, 0, TURBO_PROGRAM as u8, 0]);
        let cars = [Vehicle::YellowJacket, Vehicle::Outlaw];
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            cars[0],
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(50));
        let solo = |world: &mut World, held| {
            let mut keys = ButtonInput::default();
            if held {
                keys.press(KeyT);
            }
            world.insert_resource(keys);
            world.run_system_once(crate::driving::advance).unwrap();
            world.run_system_once(update).unwrap();
            let mut voices = world
                .query::<(Entity, &OneShot, Has<AudioPlayer<looping::LoopingPcm>>)>()
                .iter(world)
                .filter(|(_, voice, _)| voice.0 == Some(Turbo))
                .map(|(e, _, loops)| (e, loops))
                .collect::<Vec<_>>();
            voices.sort();
            voices
        };
        solo(&mut world, false); // Acquire window focus before the first driving step.
        let voices = solo(&mut world, true);
        assert_eq!(voices.len(), 4);
        assert_eq!(voices.iter().filter(|v| v.1).count(), 3);
        assert_eq!(solo(&mut world, true), voices);
        assert!(solo(&mut world, false).is_empty());
        assert_eq!(solo(&mut world, true).len(), 4);
        for _ in 0..130 {
            solo(&mut world, true);
        }
        assert!(solo(&mut world, true).is_empty());

        crate::duel::start(
            &mut world,
            Level::Warehouse,
            cars,
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        let duel = |world: &mut World, keys: &[KeyCode]| {
            let mut input = ButtonInput::default();
            for key in keys {
                input.press(*key);
            }
            world.insert_resource(input);
            world.run_system_once(crate::duel::advance).unwrap();
            world.run_system_once(duel::mix).unwrap();
            world
                .query_filtered::<Entity, (
                    With<crate::duel::DuelEffect>,
                    With<AudioPlayer<looping::LoopingPcm>>,
                )>()
                .iter(world)
                .collect::<Vec<_>>()
        };
        // Both requests address sound 0x82; the second slot can stop the first slot.
        assert!(duel(&mut world, &[KeyT]).is_empty());
        let voices = duel(&mut world, &[KeyT, Slash]);
        assert_eq!(voices.len(), 3);
        assert_eq!(duel(&mut world, &[KeyT, Slash]), voices);
        assert_eq!(duel(&mut world, &[Slash]).len(), 3);
        assert!(duel(&mut world, &[]).is_empty());
    }
}
