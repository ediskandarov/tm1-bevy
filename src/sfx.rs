pub mod duel;
pub mod pcm;
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
    effects: Vec<Vec<EffectTone>>,
    rival_engines: Vec<Vec<EngineSample>>,
    voices: Vec<Handle<AudioSource>>,
    player_engine: Vec<EngineSample>,
    screen_engine: Vec<(Handle<pcm::StereoPcm>, f32)>,
}

struct EffectTone {
    clip: pcm::StereoPcm,
    priority: u8,
}

pub struct EngineSample {
    priority: u8,
    pub clip: pcm::StereoPcm,
    pub range: f32,
}
impl EngineSample {
    pub fn load(bank: &[u8], car: tm1_core::drive::Vehicle) -> Result<Vec<Self>> {
        program(bank, car.sound_programs().0)?
            .into_iter()
            .map(|tone| {
                let priority = tone.priority;
                let (sample, marker) = Sample::single(tone, true);
                let marker = marker
                    .ok_or_else(|| anyhow::anyhow!("missing engine VAB loop point for {car:?}"))?;
                Ok(Self {
                    priority,
                    clip: pcm::StereoPcm::new(&sample.mix, Some(marker)),
                    range: sample.range,
                })
            })
            .collect()
    }
}

pub struct Sample {
    mix: Vec<[f32; 2]>,
    pub range: f32,
}
impl Sample {
    pub fn load(bank: &[u8], prog: usize, engine: bool) -> Result<Self> {
        Ok(Self::from_tones(program(bank, prog)?, engine))
    }
    fn from_tones(tones: Vec<tm1_assets::sound::Tone>, engine: bool) -> Self {
        let mut engine_range = 0.;
        let mut mix = Vec::<[f32; 2]>::new();
        for t in tones {
            let note = if engine {
                engine_range = f32::from(t.high) - f32::from(t.low);
                t.low
            } else {
                60
            };
            let rate = 2f32.powf((f32::from(note) - f32::from(t.center)) / 12.);
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
        }
    }
    fn single(tone: tm1_assets::sound::Tone, engine: bool) -> (Self, Option<usize>) {
        let first_loop_frame = tone.loop_start.map(|frame| {
            let note = if engine { tone.low } else { 60 };
            let rate = 2.0_f32.powf((f32::from(note) - f32::from(tone.center)) / 12.);
            (frame as f64 / f64::from(rate)).ceil() as usize
        });
        (Sample::from_tones(vec![tone], engine), first_loop_frame)
    }
    pub fn wave(&self) -> Result<AudioSource> {
        let pcm: Vec<_> = self
            .mix
            .iter()
            .flatten()
            .flat_map(|v| (v.clamp(-32768., 32767.) as i16).to_le_bytes())
            .collect();
        crate::music::stereo_wave(&pcm)
    }
}

// Program 26 is the four-tone sound at ID 0x82 in the JP sound table.
const TURBO_PROGRAM: usize = 26;

struct PreparedTone {
    wave: AudioSource,
    looping_clip: Option<pcm::StereoPcm>,
    priority: u8,
    range: f32,
}

struct Layer {
    source: Handle<AudioSource>,
    looping_clip: Option<Handle<pcm::StereoPcm>>,
    priority: u8,
    range: f32,
}

impl PreparedTone {
    fn new(tone: tm1_assets::sound::Tone, engine: bool, looping: bool) -> Result<Self> {
        let priority = tone.priority;
        let (sample, marker) = Sample::single(tone, engine);
        let first_loop_frame = marker.filter(|_| looping);
        anyhow::ensure!(
            !engine || first_loop_frame.is_some(),
            "missing VAB engine loop point"
        );
        Ok(Self {
            wave: sample.wave()?,
            looping_clip: first_loop_frame
                .map(|frame| pcm::StereoPcm::new(&sample.mix, Some(frame))),
            priority,
            range: sample.range,
        })
    }
    fn register(self, world: &mut World) -> Layer {
        Layer {
            source: world.resource_mut::<Assets<AudioSource>>().add(self.wave),
            looping_clip: self
                .looping_clip
                .map(|clip| world.resource_mut::<Assets<pcm::StereoPcm>>().add(clip)),
            priority: self.priority,
            range: self.range,
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
}

pub(crate) fn engine_speed(range: f32, rev: i32) -> f32 {
    let pitch_steps = (range as i32 * 128 * rev.clamp(0, 99)) / 99;
    let speed = 2f32.powf(pitch_steps as f32 / (128. * 12.));
    // Rodio 0.22 multiplies the reduced input and output rates in u32.
    // Common output rates share a factor of 25 Hz with these VAB rates.
    let rate = (44100. * speed / 25.).round() * 25.;
    let factor = rate / 44100.;
    if factor * 44100. < rate {
        factor.next_up()
    } else {
        factor
    }
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
        .chain([
            28,
            54,
            25,
            24,
            23,
            22,
            SweetTooth.sound_programs().1,
            57,
            58,
        ])
}

impl Sounds {
    pub fn install_vehicle_sounds(&mut self, world: &mut World) {
        world.init_resource::<Assets<AudioSource>>();
        world.init_resource::<Assets<pcm::StereoPcm>>();
        let turbo = self
            .turbo
            .drain(..)
            .map(|tone| tone.register(world))
            .collect();
        world.insert_resource(VehicleSounds { turbo });
    }

    fn register(&mut self, assets: &mut Assets<AudioSource>) {
        self.voices = std::mem::take(&mut self.sources)
            .into_iter()
            .map(|s| assets.add(s))
            .collect();
    }
    pub fn menu_voice(&self, select: bool) -> AudioPlayer {
        self.centered(self.voices.len() - 2 + usize::from(select))
    }
    pub fn centered(&self, id: usize) -> AudioPlayer {
        AudioPlayer::new(self.voices[id].clone())
    }
    pub fn ending_engine(&self, gain: f32) -> Vec<(AudioPlayer<pcm::StereoPcm>, PlaybackSettings)> {
        self.screen_engine
            .iter()
            .map(|(source, range)| {
                (
                    AudioPlayer(source.clone()),
                    PlaybackSettings::ONCE
                        .with_speed(engine_speed(*range, 30))
                        .with_volume(Volume::Linear(0.28 * gain)),
                )
            })
            .collect()
    }

    pub fn signature(&self) -> AudioPlayer {
        AudioPlayer::new(self.voices[5].clone())
    }

    pub fn lost_game_signature(&self) -> AudioPlayer {
        let program = self.voices.len() - 3;
        AudioPlayer::new(self.voices[program].clone())
    }

    pub fn load(
        disc: &Disc,
        vehicle: tm1_core::drive::Vehicle,
        opponents: &[tm1_core::drive::Vehicle],
    ) -> Result<Self> {
        let bank = disc.read("SND/UACORE.VAB")?;
        let mut sources = Vec::new();
        let mut effects = Vec::new();
        for (id, prog) in battle_programs(vehicle, opponents).enumerate() {
            let sample = Sample::load(&bank, prog, id == 0)?;
            let tones = program(&bank, prog)?
                .into_iter()
                .map(|tone| {
                    let priority = tone.priority;
                    let (sample, marker) = Sample::single(tone, false);
                    EffectTone {
                        clip: pcm::StereoPcm::new(
                            &sample.mix,
                            marker.filter(|_| id == 1 || prog == 38),
                        ),
                        priority,
                    }
                })
                .collect();
            effects.push(tones);
            sources.push(sample.wave()?);
        }
        let player_engine = EngineSample::load(&bank, vehicle)?;
        let rival_engines = opponents
            .iter()
            .map(|&car| EngineSample::load(&bank, car))
            .collect::<Result<_>>()?;
        Ok(Self {
            sources,
            effects,
            turbo: program(&bank, TURBO_PROGRAM)?
                .into_iter()
                .map(|tone| PreparedTone::new(tone, false, true))
                .collect::<Result<_>>()?,
            rival_engines,
            voices: Vec::new(),
            player_engine,
            screen_engine: Vec::new(),
        })
    }
}
#[derive(Component)]
pub struct Engine;
#[derive(Component)]
pub struct RivalEngine {
    pub(super) slot: usize,
}
#[derive(Component)]
pub struct EngineTone {
    order: Option<u64>,
    priority: u8,
    source: Handle<pcm::StereoPcm>,
    levels: pcm::Levels,
    range: f32,
}
#[derive(Component)]
pub struct OneShot {
    key: Option<crate::driving::ExclusiveVoice>,
    levels: Option<pcm::Levels>,
    order: u64,
    priority: u8,
}

#[derive(Resource, Default)]
pub struct VoiceSequence {
    serial: u64,
    last_frame: Option<u32>,
}
impl VoiceSequence {
    fn next(&mut self) -> u64 {
        self.serial += 1;
        self.serial
    }
}

struct Candidate {
    entity: Entity,
    order: u64,
    priority: u8,
    range: u32,
    finishing: bool,
    engine: bool,
}

pub fn start(
    mut commands: Commands,
    mut sounds: ResMut<Sounds>,
    drive: Res<crate::driving::Drive>,
    mut assets: ResMut<Assets<AudioSource>>,
    mut clips: ResMut<Assets<pcm::StereoPcm>>,
) {
    sounds.register(&mut assets);
    let mut sequence = VoiceSequence::default();
    let player = std::mem::take(&mut sounds.player_engine);
    let rivals = std::mem::take(&mut sounds.rival_engines);
    let engines = std::iter::once((None, player)).chain(
        rivals
            .into_iter()
            .enumerate()
            .map(|(slot, tones)| (Some(slot), tones)),
    );
    for (slot, tones) in engines {
        let levels = pcm::Levels::new(
            drive.listener.channels(
                drive.player.position(),
                drive.settings.effects_volume(),
                false,
            ),
            99,
        );
        for EngineSample {
            clip,
            range,
            priority,
        } in tones
        {
            let source = clips.add(clip.with_levels(levels.clone()));
            let mut entity = commands.spawn((
                crate::campaign::StageEntity,
                EngineTone {
                    priority,
                    order: slot.is_none().then(|| sequence.next()),
                    source: source.clone(),
                    levels: levels.clone(),
                    range,
                },
            ));
            if let Some(slot) = slot {
                entity.insert(RivalEngine { slot });
            } else {
                sounds.screen_engine.push((clips.add(clip), range));
                entity.insert((
                    Engine,
                    AudioPlayer(source),
                    PlaybackSettings::ONCE
                        .with_volume(Volume::Linear(0.28))
                        .with_speed(engine_speed(range, drive.player.rev())),
                ));
            }
        }
    }
    commands.insert_resource(sequence);
}
pub fn update(
    mut commands: Commands,
    sounds: Res<Sounds>,
    vehicle_sounds: Res<VehicleSounds>,
    mut clips: ResMut<Assets<pcm::StereoPcm>>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    mut shots: Query<(Entity, Option<&mut AudioSink>, &OneShot), Without<Engine>>,
    mut engines: Query<
        (
            Entity,
            &mut EngineTone,
            Option<&RivalEngine>,
            Has<AudioPlayer<pcm::StereoPcm>>,
            Option<&mut AudioSink>,
        ),
        Without<OneShot>,
    >,
    mut restart: Local<u32>,
    mut sequence: ResMut<VoiceSequence>,
) {
    let reset = *restart != drive.restart_count;
    let pause = drive.lives == 0 || drive.pause.is_some() || !window.focused;
    let gain = drive.settings.effects_gain();
    let volume = drive.settings.effects_volume();
    let mut ranked = Vec::new();
    let frame = drive.player.frame();
    let new_frame = sequence.last_frame.replace(frame) != Some(frame);
    for (entity, mut engine, rival, playing, sink) in &mut engines {
        let audible = if let Some(rival) = rival {
            drive.opponents.get(rival.slot).and_then(|ai| {
                let rev = ai.engine_rev()?;
                let channels = drive.listener.channels(ai.position(), volume, false);
                (channels != [0; 2]).then_some((rev, channels))
            })
        } else {
            Some((
                drive.player.rev(),
                drive
                    .listener
                    .channels(drive.player.position(), volume, false),
            ))
        };
        let Some((rev, channels)) = audible else {
            engine.order = None;
            if playing {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer<pcm::StereoPcm>, AudioSink)>();
            }
            continue;
        };
        if !playing && !new_frame && !reset {
            continue;
        }
        let position = rival
            .map(|r| drive.opponents[r.slot].position())
            .unwrap_or_else(|| drive.player.position());
        ranked.push(Candidate {
            entity,
            order: *engine.order.get_or_insert_with(|| sequence.next()),
            priority: engine.priority,
            range: drive.listener.sound_range(position),
            finishing: false,
            engine: true,
        });
        engine.levels.set(channels, 99);
        let speed = engine_speed(engine.range, rev);
        if reset && playing {
            commands.entity(entity).remove::<AudioSink>();
        }
        if let Some(sink) = sink.filter(|_| !reset) {
            sink.set_speed(speed);
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        } else if !playing || reset {
            let mut settings = PlaybackSettings::ONCE
                .with_speed(speed)
                .with_volume(Volume::Linear(0.28));
            settings.paused = pause;
            commands
                .entity(entity)
                .insert((AudioPlayer(engine.source.clone()), settings));
        }
    }
    *restart = drive.restart_count;
    for (e, sink, shot) in &mut shots {
        if reset || drive.pause.is_some() {
            commands.entity(e).despawn();
            continue;
        }
        ranked.push(Candidate {
            entity: e,
            order: shot.order,
            priority: shot.priority,
            range: 0,
            finishing: true,
            engine: false,
        });
        if let Some(sink) = sink {
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

    let mut active: Vec<_> = shots
        .iter()
        .filter_map(|(entity, _, shot)| {
            shot.key
                .filter(|_| !reset && drive.pause.is_none())
                .map(|key| (key, entity, shot.levels.clone()))
        })
        .collect();
    use crate::driving::{BattleRequest, ExclusiveVoice};
    for request in &drive.sounds {
        let sound = match request {
            BattleRequest::Play(sound) => sound,
            BattleRequest::Stop(key) => {
                active.retain(|(group, entity, _)| {
                    if group == key {
                        commands.entity(*entity).despawn();
                        ranked.retain(|v| v.entity != *entity);
                    }
                    group != key
                });
                continue;
            }
            BattleRequest::Turbo => {
                let key = ExclusiveVoice::Turbo;
                if active.iter().any(|(group, _, _)| *group == key) {
                    refresh_requested(&mut ranked, &active, key, 0);
                    continue;
                }
                for tone in &vehicle_sounds.turbo {
                    let mut voice = commands.spawn((
                        crate::campaign::StageEntity,
                        OneShot {
                            key: Some(key),
                            levels: None,
                            order: sequence.next(),
                            priority: tone.priority,
                        },
                        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.44 * gain)),
                    ));
                    tone.play(&mut voice);
                    active.push((key, voice.id(), None));
                    ranked.push(Candidate {
                        entity: voice.id(),
                        order: sequence.serial,
                        priority: tone.priority,
                        range: 0,
                        finishing: false,
                        engine: false,
                    });
                }
                continue;
            }
        };
        if let Some(key) = sound.exclusive
            && let Some((_, _, Some(control))) = active.iter().find(|(group, _, _)| *group == key)
        {
            refresh_requested(&mut ranked, &active, key, sound.range);
            control.set(sound.channels, sound.volume_percent);
            continue;
        }
        if sound.channels == [0; 2] || sound.volume_percent <= 0 {
            continue;
        }
        let control = pcm::Levels::new(sound.channels, sound.volume_percent);
        for tone in &sounds.effects[sound.id] {
            let clip = tone.clip.with_levels(control.clone());
            let order = sequence.next();
            let entity = commands
                .spawn((
                    crate::campaign::StageEntity,
                    OneShot {
                        key: sound.exclusive,
                        levels: Some(control.clone()),
                        order,
                        priority: tone.priority,
                    },
                    AudioPlayer(clips.add(clip)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.44)),
                ))
                .id();
            ranked.push(Candidate {
                entity,
                order,
                priority: tone.priority,
                range: sound.range,
                finishing: false,
                engine: false,
            });
            if let Some(key) = sound.exclusive {
                active.push((key, entity, Some(control.clone())));
            }
        }
    }
    if ranked.len() > 24 {
        // Requested tones compete first. Finishing tones fill any unused voices in list order.
        ranked.sort_unstable_by_key(|v| (v.finishing, v.order));
        let requested = ranked.partition_point(|v| !v.finishing);
        tm1_core::sound::sort_requested(&mut ranked[..requested], |v| v.range, |v| v.priority);
        for voice in &ranked[24..] {
            if voice.engine {
                commands
                    .entity(voice.entity)
                    .remove::<(AudioPlayer<pcm::StereoPcm>, AudioSink)>();
            } else {
                commands.entity(voice.entity).despawn();
            }
        }
    }
}

fn refresh_requested(
    ranked: &mut [Candidate],
    active: &[(crate::driving::ExclusiveVoice, Entity, Option<pcm::Levels>)],
    key: crate::driving::ExclusiveVoice,
    range: u32,
) {
    for (_, entity, _) in active.iter().filter(|(group, _, _)| *group == key) {
        if let Some(voice) = ranked.iter_mut().find(|v| v.entity == *entity) {
            voice.finishing = false;
            voice.range = range;
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
    fn actor_signatures_and_wrecks_select_the_disc_programs() {
        use tm1_core::drive::CarSound;
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let table = 2048 + (0x8016fefc - base) as usize;
        let program_for = |id: usize| exe[table + 6 * id + 2] as usize;
        // IDs follow the original car sound order, including Minion's separate signature.
        let cars = [
            Vehicle::SweetTooth,
            Vehicle::YellowJacket,
            Vehicle::Darkside,
            Vehicle::Hammerhead,
            Vehicle::Outlaw,
            Vehicle::CrimsonFury,
            Vehicle::Warthog,
            Vehicle::MrGrimm,
            Vehicle::PitViper,
            Vehicle::Thumper,
            Vehicle::Spectre,
            Vehicle::RoadKill,
            Vehicle::Minion,
        ];
        for (i, car) in cars.into_iter().enumerate() {
            for (slot, pair) in [
                (0, [car, Vehicle::YellowJacket]),
                (1, [Vehicle::YellowJacket, car]),
            ] {
                let bank = battle_programs(pair[0], &pair[1..]).collect::<Vec<_>>();
                let launch_ids = [
                    0x2a, 0x3c, 0x42, 0x36, 0x3f, 0x39, 0x33, 0x48, 0x27, 0x45, 0x30, 0x2d, 0x4b,
                ];
                assert_eq!(
                    bank[if slot == 0 { 5 } else { 12 }],
                    program_for(launch_ids[i])
                );
                assert_eq!(bank[6], program_for(0x0f));
                for (sound, id) in [(CarSound::Signature, 0x74 + i), (CarSound::Explosion, 0x73)] {
                    let (index, _, level) = crate::driving::car_sample(sound, car, slot, 13);
                    assert_eq!(bank[index], program_for(id), "{car:?} {sound:?}");
                    assert_eq!(level, 99);
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn hover_guns_and_car_guns_stop_independently_in_queue_order() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleRequest, BattleSound, Drive, ExclusiveVoice};
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Rooftops,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let car = ExclusiveVoice::MachineGuns(Vehicle::RoadKill);
        let hover = ExclusiveVoice::HoverGuns;
        let play = |key| {
            BattleRequest::Play(BattleSound {
                id: 1,
                exclusive: Some(key),
                volume_percent: 99,
                range: 0,
                channels: [33; 2],
            })
        };
        let voices = |world: &mut World, key| {
            let mut entities = world
                .query::<(Entity, &OneShot, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(world)
                .filter(|(_, v, _)| v.key == Some(key))
                .map(|(e, _, _)| e)
                .collect::<Vec<_>>();
            entities.sort();
            entities
        };
        world.resource_mut::<Drive>().sounds =
            vec![play(car), play(hover), BattleRequest::Stop(hover)];
        world.run_system_once(update).unwrap();
        let player = voices(&mut world, car);
        assert_eq!(player.len(), 2);
        assert!(voices(&mut world, hover).is_empty());
        world.resource_mut::<Drive>().sounds = vec![BattleRequest::Stop(hover), play(hover)];
        world.run_system_once(update).unwrap();
        let merc = voices(&mut world, hover);
        assert_eq!(merc.len(), 2);
        assert_eq!(voices(&mut world, car), player);
        world.resource_mut::<Drive>().sounds = vec![play(hover), BattleRequest::Stop(car)];
        world.run_system_once(update).unwrap();
        assert_eq!(voices(&mut world, hover), merc);
        assert!(voices(&mut world, car).is_empty());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_explosions_preempt_engines_then_yield_finishing_voices() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleSound, Drive};
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.resource_mut::<Drive>().sounds = (0..10)
            .map(|i| BattleSound {
                id: 4,
                exclusive: None,
                volume_percent: 99,
                channels: [33; 2],
                range: i * 10,
            })
            .map(crate::driving::BattleRequest::Play)
            .collect();
        world.run_system_once(update).unwrap();
        let shots = |world: &mut World| {
            let mut shots: Vec<_> = world
                .query::<&OneShot>()
                .iter(world)
                .map(|v| (v.order, v.priority))
                .collect();
            shots.sort();
            shots
        };
        let first = shots(&mut world);
        assert_eq!(first.len(), 24);
        assert!(first.iter().all(|v| v.1 == 21));
        let engines = |world: &mut World| {
            world
                .query_filtered::<Entity, (With<Engine>, With<AudioPlayer<pcm::StereoPcm>>)>()
                .iter(world)
                .count()
        };
        assert_eq!(engines(&mut world), 0);
        world.resource_mut::<Drive>().sounds.clear();
        world.run_system_once(update).unwrap();
        assert_eq!(shots(&mut world), first);
        world.insert_resource(crate::smoke::Capture::new(None, true, false));
        world.run_system_once(crate::driving::advance).unwrap();
        world.resource_mut::<Drive>().sounds.clear();
        world.run_system_once(update).unwrap();
        assert_eq!(engines(&mut world), 2);
        let finishing = shots(&mut world);
        assert!(finishing.len() <= 22);
        assert_eq!(finishing, first[..finishing.len()]);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn engine_pitch_fits_the_audio_resampler() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let ranges = Vehicle::ROSTER
            .into_iter()
            .chain([Vehicle::Minion])
            .flat_map(|car| program(&bank, car.sound_programs().0).unwrap())
            .map(|tone| tone.high - tone.low);
        for range in ranges {
            for rev in 0..=99 {
                let rate = (44100. * engine_speed(range as f32, rev)) as u32;
                assert_eq!(rate % 25, 0);
                for output in [44100, 48000, 88200, 96000, 192000] {
                    let (mut a, mut b) = (rate, output);
                    while b != 0 {
                        (a, b) = (b, a % b);
                    }
                    assert!((rate / a).checked_mul(output / a - 1).is_some());
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn road_kill_uses_both_engine_tones_in_battle_and_at_the_reward() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::Drive;
        use bevy::audio::Decodable;

        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let car = Vehicle::RoadKill;
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let tones = program(&bank, car.sound_programs().0).unwrap();
        assert_eq!(tones.len(), 2);
        let stage =
            Stage::load(disc, Level::Rooftops, car, Entry::New(Difficulty::Medium)).unwrap();
        crate::campaign::replace(&mut world, stage);
        let mut drive = world.resource_mut::<Drive>();
        let [x, y, z] = drive.player.position();
        drive.listener = tm1_core::drive::SoundListener::new(
            &drive.player,
            &drive.camera,
            Some([x + 500, y, z, 0, 0]),
        );
        let channels = drive
            .listener
            .channels([x, y, z], drive.settings.effects_volume(), false);
        assert_ne!(channels[0], channels[1]);
        world.run_system_once(update).unwrap();
        let mut battle = world
            .query_filtered::<(&EngineTone, &AudioPlayer<pcm::StereoPcm>), With<Engine>>()
            .iter(&world)
            .map(|(tone, player)| (tone.range as i32, player.0.clone()))
            .collect::<Vec<_>>();
        battle.sort_by_key(|(range, _)| *range);
        crate::ending::begin(&mut world).unwrap();
        let mut reward = world.query_filtered::<(&PlaybackSettings,&AudioPlayer<pcm::StereoPcm>), With<crate::ending::EndingEngine>>()
            .iter(&world).map(|(setting,player)| (setting.speed,player.0.clone())).collect::<Vec<_>>();
        reward.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut tones = tones;
        tones.sort_by_key(|t| t.high - t.low);
        assert_eq!(battle.len(), 2);
        assert_eq!(reward.len(), 2);
        let clips = world.resource::<Assets<pcm::StereoPcm>>();
        for ((tone, (range, driving)), (speed, ending)) in tones.into_iter().zip(battle).zip(reward)
        {
            assert_eq!(range, i32::from(tone.high - tone.low));
            assert_eq!(speed, engine_speed(range as f32, 30));
            let (dry, marker) = Sample::single(tone, true);
            let mut drive = clips.get(&driving).unwrap().decoder();
            let mut screen = clips.get(&ending).unwrap().decoder();
            for frame in dry
                .mix
                .iter()
                .chain(dry.mix[marker.unwrap()..].iter().take(1024))
            {
                for (sample, level) in frame.iter().zip(channels) {
                    let sample = sample.clamp(-32768., 32767.) / 32768.;
                    assert_eq!(screen.next(), Some(sample));
                    assert_eq!(
                        drive.next(),
                        Some(sample * ((level * 99) as f32 / (33. * 99.)))
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_loads_engine_layers_for_the_full_roster() {
        let world = crate::contest::menu_world();
        let disc = &world.resource::<crate::campaign::Campaign>().0;
        let cars = [Vehicle::CARD_ORDER.as_slice(), &[Vehicle::Minion]].concat();
        let sounds = Sounds::load(disc, Vehicle::RoadKill, &cars).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        assert_eq!(sounds.rival_engines.len(), cars.len());
        for (tones, car) in sounds.rival_engines.iter().zip(cars) {
            assert_eq!(
                tones.len(),
                program(&bank, car.sound_programs().0).unwrap().len()
            );
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn rival_engine_tones_follow_the_camera_through_their_loop() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::Drive;
        use bevy::audio::Decodable;
        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.insert_resource(crate::smoke::Capture::new(None, true, false));
        for _ in 0..110 {
            world.run_system_once(crate::driving::advance).unwrap();
        }
        let drive = world.resource::<Drive>();
        let (slot, rival) = drive
            .opponents
            .iter()
            .enumerate()
            .filter(|(_, rival)| rival.engine_rev().is_some())
            .max_by_key(|(_, rival)| {
                program(&bank, rival.vehicle().sound_programs().0)
                    .unwrap()
                    .len()
            })
            .unwrap();
        let position = rival.position();
        let mut tones = program(&bank, rival.vehicle().sound_programs().0).unwrap();
        let count = tones.len();
        let mut playbacks = Vec::new();
        let mut previous = None;
        for dx in [-500, 500] {
            let mut drive = world.resource_mut::<Drive>();
            drive.listener = tm1_core::drive::SoundListener::new(
                &drive.player,
                &drive.camera,
                Some([position[0] + dx, position[1], position[2], 0, 0]),
            );
            let channels =
                drive
                    .listener
                    .channels(position, drive.settings.effects_volume(), false);
            assert_ne!(channels[0], channels[1]);
            if let Some(old) = previous {
                assert_eq!(channels[0], old);
            }
            previous = Some(channels[1]);
            world.run_system_once(update).unwrap();
            let mut voices = world
                .query::<(Entity, &RivalEngine, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(&world)
                .filter(|(_, e, _)| e.slot == slot)
                .map(|(e, _, p)| (e, p.0.clone()))
                .collect::<Vec<_>>();
            voices.sort_by_key(|(e, _)| e.index());
            assert_eq!(voices.len(), count);
            if playbacks.is_empty() {
                for ((entity, handle), tone) in voices.iter().zip(tones.drain(..)) {
                    let (dry, marker) = Sample::single(tone, true);
                    let played = world
                        .resource::<Assets<pcm::StereoPcm>>()
                        .get(handle)
                        .unwrap()
                        .decoder();
                    playbacks.push((*entity, dry.mix, marker.unwrap(), played, 0));
                }
            }
            for ((entity, dry, marker, played, cursor), (current, _)) in
                playbacks.iter_mut().zip(voices)
            {
                assert_eq!(*entity, current);
                let mut audible = false;
                for _ in 0..dry.len() + 32 {
                    if *cursor == dry.len() {
                        *cursor = *marker;
                    }
                    for (sample, level) in dry[*cursor].into_iter().zip(channels) {
                        let expected = sample.clamp(-32768., 32767.) / 32768.
                            * ((level * 99) as f32 / (33. * 99.));
                        assert_eq!(played.next(), Some(expected));
                        audible |= expected != 0.;
                    }
                    *cursor += 1;
                }
                assert!(audible);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn repeated_impact_updates_both_channels_without_restarting() {
        use crate::{
            campaign::{Campaign, Entry, Stage},
            driving::{BattleSound, Drive, ExclusiveVoice},
        };
        use bevy::audio::Decodable;
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            Vehicle::Outlaw,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let impact = |channels| {
            crate::driving::BattleRequest::Play(BattleSound {
                range: 0,
                channels,
                exclusive: Some(ExclusiveVoice::ScaledEffect(103)),
                id: 3,
                volume_percent: 84,
            })
        };
        let mut references: Vec<_> = world.resource::<Sounds>().effects[3]
            .iter()
            .map(|tone| tone.clip.decoder())
            .collect();
        world.resource_mut::<Drive>().sounds = vec![impact([12, 55]), impact([47, 19])];
        world.run_system_once(update).unwrap();
        let voices = |world: &mut World| {
            let mut voices: Vec<_> = world
                .query::<(&OneShot, Entity, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(world)
                .map(|(voice, e, p)| (voice.order, e, p.0.clone()))
                .collect();
            voices.sort_by_key(|v| v.0);
            voices
        };
        let first = voices(&mut world);
        assert_eq!(first.len(), references.len());
        let clips = world.resource::<Assets<pcm::StereoPcm>>();
        let mut played: Vec<_> = first
            .iter()
            .map(|(_, _, h)| clips.get(h).unwrap().decoder())
            .collect();
        for channels in [[47, 19], [25, 51]] {
            world.resource_mut::<Drive>().sounds = vec![impact(channels)];
            if channels[0] == 25 {
                world.run_system_once(update).unwrap();
                assert_eq!(voices(&mut world), first);
            }
            let mut audible = false;
            for (reference, played) in references.iter_mut().zip(&mut played) {
                for _ in 0..1024 {
                    for level in channels {
                        let dry = reference.next().unwrap();
                        audible |= dry != 0.;
                        assert_eq!(
                            played.next().unwrap(),
                            dry * ((level * 84) as f32 / (33. * 99.))
                        );
                    }
                }
            }
            assert!(audible);
        }
    }

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
                    shot.key == Some(crate::driving::ExclusiveVoice::MachineGuns(car))
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
                .query::<(Entity, &OneShot, Has<AudioPlayer<pcm::StereoPcm>>)>()
                .iter(world)
                .filter(|(_, voice, _)| voice.key == Some(Turbo))
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
                    With<AudioPlayer<pcm::StereoPcm>>,
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
