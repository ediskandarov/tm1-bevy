//! Tone allocation for the shared SPU during split-screen play.
use super::{
    Candidate, EffectTone, EngineSample, MenuSound, TURBO_PROGRAM, TimedVoice, VoiceSequence,
    battle_programs, pcm::Levels, pcm::StereoPcm, rank_voices,
};
use crate::{
    campaign::StageEntity,
    driving::{BattleRequest, ExclusiveVoice},
    duel::{Duel, DuelEffect},
};
use anyhow::Result;
use bevy::prelude::*;
use tm1_core::drive::Vehicle;

#[derive(Resource)]
pub struct DuelBank {
    programs: Vec<Vec<EffectTone>>,
    engines: [Vec<EngineSample>; 2],
    engine_levels: Levels,
    score_engines: Vec<Handle<StereoPcm>>,
    turbo: Vec<EffectTone>,
    pub(crate) menu: [MenuSound; 2],
}

#[derive(Component)]
pub struct DuelEngine {
    pub(crate) slot: usize,
    source: Handle<StereoPcm>,
    pitch: super::EnginePitch,
    register: super::pcm::PitchRegister,
}

#[derive(Component)]
pub struct Voice {
    timed: Option<TimedVoice>,
    order: u64,
    priority: u8,
    distance: u32,
    group: Option<(ExclusiveVoice, Levels)>,
}

#[cfg(test)]
impl Voice {
    pub(super) fn is_turbo(&self) -> bool {
        matches!(self.group, Some((ExclusiveVoice::Turbo, _)))
    }
}

impl DuelBank {
    pub fn load(bank: &[u8], cars: [Vehicle; 2], volume: u8, world: &mut World) -> Result<Self> {
        world.init_resource::<Assets<super::pcm::StereoPcm>>();
        let engines = [
            EngineSample::load(bank, cars[0])?,
            EngineSample::load(bank, cars[1])?,
        ];
        let engine_levels = Levels::new([i32::from(volume); 2], 99);
        let score_engines = engines[0]
            .iter()
            .map(|tone| {
                world
                    .resource_mut::<Assets<StereoPcm>>()
                    .add(tone.voice(30).with_levels(engine_levels.clone()))
            })
            .collect();
        let programs = battle_programs(cars[0], &cars[1..])
            .map(|prog| EffectTone::load(bank, prog))
            .collect::<Result<_>>()?;
        Ok(Self {
            programs,
            engines,
            engine_levels,
            score_engines,
            turbo: EffectTone::load(bank, TURBO_PROGRAM)?,
            menu: [MenuSound::load(bank, 57)?, MenuSound::load(bank, 58)?],
        })
    }

    pub fn install(self, world: &mut World) {
        let mut sequence = VoiceSequence::default();
        for (slot, layers) in self.engines.iter().enumerate() {
            for layer in layers {
                let (clip, register) = layer.motor.voice(layer.pitch.register(0));
                let source = world
                    .resource_mut::<Assets<StereoPcm>>()
                    .add(clip.with_levels(self.engine_levels.clone()));
                world.spawn((
                    StageEntity,
                    DuelEngine {
                        slot,
                        source: source.clone(),
                        pitch: layer.pitch,
                        register,
                    },
                    Voice {
                        timed: None,
                        order: sequence.next(),
                        priority: layer.priority,
                        distance: 0,
                        group: None,
                    },
                    AudioPlayer(source.clone()),
                    PlaybackSettings::ONCE,
                ));
            }
        }
        world.insert_resource(sequence);
        world.insert_resource(self);
    }

    pub fn set_engine_volume(&self, volume: u8) {
        self.engine_levels.set([i32::from(volume); 2], 99);
    }

    pub fn play_score_engine(&self, commands: &mut Commands) {
        for clip in &self.score_engines {
            commands.spawn((
                StageEntity,
                crate::duel::ScoreEngine,
                AudioPlayer(clip.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

pub fn mix(
    mut commands: Commands,
    duel: Res<Duel>,
    bank: Res<DuelBank>,
    mut clips: ResMut<Assets<StereoPcm>>,
    mut sequence: ResMut<VoiceSequence>,
    mut playing: Query<(
        Entity,
        &mut Voice,
        Option<&DuelEngine>,
        Has<AudioPlayer<StereoPcm>>,
    )>,
) {
    if duel.pause.is_some() || duel.finished() {
        return;
    }
    let frame = duel.cars[0].frame();
    if sequence.last_frame == Some(frame) {
        return;
    }
    sequence.last_frame = Some(frame);
    // Keep engines eligible when a higher-priority effect temporarily takes their voices.
    let mut ranked = Vec::new();
    let mut groups = Vec::new();
    let restarted = sequence.update_engines(&duel.sounds);
    for (entity, mut voice, engine, audible) in &mut playing {
        if voice.timed.is_some_and(|t| t.expired(frame)) {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(engine) = engine {
            if sequence.engines_stopped[engine.slot] {
                if audible {
                    commands
                        .entity(entity)
                        .remove::<(AudioPlayer<StereoPcm>, AudioSink)>();
                }
                continue;
            }
            if sequence.engines_deferred[engine.slot] && !audible {
                continue;
            }
            if restarted[engine.slot] {
                voice.order = sequence.next();
                if audible {
                    commands.entity(entity).remove::<AudioSink>();
                }
            }
        }
        ranked.push(Candidate {
            entity,
            order: voice.order,
            finishing: voice.timed.is_none()
                && engine.is_none_or(|e| sequence.engines_deferred[e.slot]),
            priority: voice.priority,
            range: voice.distance,
            engine: engine.is_some(),
        });
        if let Some((key, levels)) = &voice.group {
            groups.push((*key, entity, levels.clone()));
        }
    }
    let mut stopped = Vec::new();
    for request in &duel.sounds {
        let (layers, key, channels, percent, distance) = match request {
            BattleRequest::EngineRev(..)
            | BattleRequest::EngineStop(_)
            | BattleRequest::EngineDeferred(_) => continue,
            BattleRequest::Turbo => (
                &bank.turbo,
                Some(ExclusiveVoice::Turbo),
                [i32::from(duel.settings.effects_volume()); 2],
                99,
                0,
            ),
            BattleRequest::Play(sound) => (
                &bank.programs[sound.id],
                sound.exclusive,
                sound.channels,
                sound.volume_percent,
                sound.range,
            ),
            BattleRequest::Stop(key) => {
                groups.retain(|(group, entity, _)| {
                    if group != key {
                        return true;
                    }
                    ranked.retain(|entry| entry.entity != *entity);
                    stopped.push(*entity);
                    false
                });
                continue;
            }
        };
        if let Some(key) = key
            && let Some((_, _, levels)) = groups.iter().find(|(k, _, _)| *k == key)
        {
            levels.set(channels, percent);
            for (_, entity, _) in groups.iter().filter(|(k, _, _)| *k == key) {
                if let Some(entry) = ranked.iter_mut().find(|entry| entry.entity == *entity) {
                    entry.finishing = false;
                    entry.range = distance;
                }
                commands
                    .entity(*entity)
                    .entry::<Voice>()
                    .and_modify(move |mut voice| voice.distance = distance);
            }
            continue;
        }
        let levels = Levels::new(channels, percent);
        for tone in layers
            .iter()
            .take(super::TONE_POOL_SIZE.saturating_sub(ranked.len()))
        {
            let order = sequence.next();
            let entity = commands
                .spawn((
                    StageEntity,
                    DuelEffect,
                    Voice {
                        timed: TimedVoice::from_request(request),
                        order,
                        priority: tone.priority,
                        distance,
                        group: key.map(|key| (key, levels.clone())),
                    },
                    AudioPlayer(clips.add(tone.clip.with_levels(levels.clone()))),
                    PlaybackSettings::DESPAWN,
                ))
                .id();
            ranked.push(Candidate {
                order,
                entity,
                priority: tone.priority,
                range: distance,
                finishing: false,
                engine: false,
            });
            if let Some(key) = key {
                groups.push((key, entity, levels.clone()));
            }
        }
    }
    super::stop_effects(&mut commands, stopped, ranked.is_empty());
    if ranked.len() > 24 {
        rank_voices(&mut ranked);
    }
    for (rank, candidate) in ranked.into_iter().enumerate() {
        let entity = candidate.entity;
        if candidate.engine {
            let Ok((_, _, Some(engine), audible)) = playing.get(entity) else {
                continue;
            };
            if let Some(mix) = sequence.engine_mix[engine.slot] {
                engine.register.set(engine.pitch.register(mix.rev));
            } else if !sequence.engines_deferred[engine.slot] {
                engine
                    .register
                    .set(engine.pitch.register(duel.cars[engine.slot].rev()));
            }
            if rank < 24 {
                if !audible {
                    commands
                        .entity(entity)
                        .insert(AudioPlayer(engine.source.clone()));
                }
            } else if audible {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer<StereoPcm>, AudioSink)>();
            }
        } else if rank >= 24 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driving::BattleSound;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;
    use tm1_core::drive::{ContactVoice, Difficulty, Level};

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn bottom_view_taser_sparks_enter_the_next_sound_queue() {
        use tm1_core::drive::{BattleEvent, SoundListener};
        let mut world = crate::contest::menu_world();
        crate::duel::start(
            &mut world,
            Level::Warehouse,
            [Vehicle::Outlaw, Vehicle::Spectre],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyF);
        world.insert_resource(keys);
        for tick in 0..2 {
            let duel = world.resource::<Duel>();
            let views =
                [0, 1].map(|slot| SoundListener::new(&duel.cars[slot], &duel.cameras[slot], None));
            world.run_system_once(crate::duel::advance).unwrap();
            let duel = world.resource::<Duel>();
            let before_primary = duel.scene.events_before_primary_view();
            let mut expected = Vec::new();
            let mut spark_count = [0; 2];
            for (i, event) in duel.scene.battle_events(&views[0]).enumerate() {
                if let BattleEvent::Effect { kind: 1, position } = event {
                    let view = usize::from(i < before_primary);
                    spark_count[view] += 1;
                    expected.push((
                        views[view].sound_range(position),
                        views[view].channels(position, duel.settings.effects_volume(), false),
                    ));
                }
            }
            assert_eq!(spark_count, if tick == 0 { [0, 0] } else { [2, 2] });
            let requests: Vec<_> = duel
                .sounds
                .iter()
                .filter_map(|r| match r {
                    BattleRequest::Play(s)
                        if s.exclusive == Some(ExclusiveVoice::ScaledEffect(103)) =>
                    {
                        Some((s.range, s.channels))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(requests, expected);
            world.insert_resource(ButtonInput::<KeyCode>::default());
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_engine_levels_follow_the_keyboard_mix() {
        use KeyCode::*;
        use bevy::audio::Decodable;
        let mut world = crate::contest::menu_world();
        let settings = &mut world.resource_mut::<crate::contest::Contest>().settings;
        crate::pause::navigate(
            crate::pause::Page::Audio(1),
            crate::pause::Action::Right,
            settings,
        );
        let initial = settings.effects_volume();
        assert_ne!(initial, 33);
        crate::duel::start(
            &mut world,
            Level::Warehouse,
            [Vehicle::RoadKill, Vehicle::Outlaw],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let check = |world: &mut World, volume| {
            let mut engines = world.query::<(&DuelEngine, &PlaybackSettings)>();
            let mut clips: Vec<_> = engines
                .iter(world)
                .map(|(engine, settings)| {
                    assert_eq!(settings.volume, bevy::audio::Volume::Linear(1.));
                    engine.source.clone()
                })
                .collect();
            assert!(clips.len() >= 3); // both RoadKill tones plus Outlaw
            clips.extend(world.resource::<DuelBank>().score_engines.iter().cloned());
            for handle in clips {
                let clip = world.resource::<Assets<StereoPcm>>().get(&handle).unwrap();
                let gain = clip.volume.gain(volume, 99);
                let mut nonzero = false;
                for (sample, dry) in clip.decoder().zip(clip.unscaled_decoder()).take(4096) {
                    nonzero |= dry != 0.;
                    assert_eq!(sample, dry * gain);
                }
                assert!(nonzero);
            }
        };
        check(&mut world, initial);
        for key in [KeyP, KeyS, KeyS, Space, KeyD] {
            let mut input = ButtonInput::default();
            input.press(key);
            world.insert_resource(input);
            world.run_system_once(crate::duel::pause_menu).unwrap();
            world.insert_resource(ButtonInput::<KeyCode>::default());
            world.run_system_once(crate::duel::pause_menu).unwrap();
        }
        let updated = world.resource::<Duel>().settings.effects_volume();
        assert_ne!(updated, initial);
        check(&mut world, updated);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_fire_uses_both_retail_listeners() {
        use bevy::audio::Decodable;
        use tm1_core::drive::{BattleEvent, SoundListener};
        let mut world = crate::contest::menu_world();
        crate::duel::start(
            &mut world,
            Level::Warehouse,
            [Vehicle::Spectre, Vehicle::Outlaw],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let mut checked_missile = false;
        let mut flight_positions = std::collections::HashSet::new();
        let mut checked_guns = false;
        for tick in 0..130 {
            let duel = world.resource::<Duel>();
            let views =
                [0, 1].map(|slot| SoundListener::new(&duel.cars[slot], &duel.cameras[slot], None));
            let mut keys = ButtonInput::default();
            keys.press(KeyCode::Comma);
            if tick == 0 {
                keys.press(KeyCode::KeyF);
            }
            world.insert_resource(keys);
            world.run_system_once(crate::duel::advance).unwrap();
            world.run_system_once(mix).unwrap();
            let duel = world.resource::<Duel>();
            let volume = duel.settings.effects_volume();
            let boundary = duel.scene.events_before_primary_view();
            for (index, event) in duel.scene.battle_events(&views[0]).enumerate() {
                let (id, pos) = match event {
                    BattleEvent::Guns {
                        slot: 1,
                        firing: true,
                        position,
                    } if !checked_guns => (1, position),
                    BattleEvent::MissileSound {
                        weapon,
                        launch: false,
                        position,
                    } => {
                        flight_positions.insert(position);
                        (crate::driving::missile_sample(weapon), position)
                    }
                    BattleEvent::Effect { kind: 28, position } if index < boundary => {
                        (15, position)
                    }
                    BattleEvent::Effect {
                        kind: 55 | 56,
                        position,
                    } if index < boundary => (22, position),
                    _ => continue,
                };
                let listener = usize::from(index < boundary);
                let expected = views[listener].channels(pos, volume, false);
                let range = views[listener].sound_range(pos);
                let other = views[1 - listener].sound_range(pos);
                assert_ne!(other, range);
                assert!(duel.sounds.iter().any(|request|matches!(request,BattleRequest::Play(s) if s.id==id && s.range==range && s.channels==expected)),"tick {tick} sample {id}");
                if id == 1 {
                    checked_guns = true;
                } else if matches!(event, BattleEvent::Effect { .. }) {
                    checked_missile = true;
                }
            }
            if tick == 0 {
                let mut query = world.query::<(&Voice, &AudioPlayer<StereoPcm>)>();
                let shots: Vec<_> = query
                    .iter(&world)
                    .filter(|(v, _)| {
                        matches!(
                            &v.group,
                            Some((ExclusiveVoice::MachineGuns(Vehicle::Outlaw), _))
                        )
                    })
                    .map(|(_, p)| p.0.clone())
                    .collect();
                let duel = world.resource::<Duel>();
                let gain = duel
                    .sounds
                    .iter()
                    .find_map(|r| match r {
                        BattleRequest::Play(s)
                            if s.exclusive
                                == Some(ExclusiveVoice::MachineGuns(Vehicle::Outlaw)) =>
                        {
                            Some(s.channels)
                        }
                        _ => None,
                    })
                    .unwrap();
                assert_ne!(gain[0], gain[1]);
                let bank = world.resource::<DuelBank>();
                assert_eq!(shots.len(), bank.programs[1].len());
                for (layer, handle) in bank.programs[1].iter().zip(shots) {
                    let clip = world.resource::<Assets<StereoPcm>>().get(&handle).unwrap();
                    let mut nonzero = false;
                    for (i, (dry, wet)) in layer
                        .clip
                        .unscaled_decoder()
                        .zip(clip.decoder())
                        .take(4096)
                        .enumerate()
                    {
                        nonzero |= dry != 0.;
                        assert_eq!(wet, dry * layer.clip.volume.gain(gain[i % 2] as u8, 99));
                    }
                    assert!(nonzero);
                }
            }
            if checked_missile && checked_guns {
                break;
            }
        }
        assert!(checked_guns && checked_missile);
        assert!(flight_positions.len() > 1);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn machine_guns_stop_independently_after_cooldown() {
        use KeyCode::*;
        let mut world = crate::contest::menu_world();
        let cars = [Vehicle::YellowJacket, Vehicle::Outlaw];
        crate::duel::start(&mut world, Level::Arena, cars, Difficulty::Medium, [0; 2]).unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let step = |world: &mut World, keys: &[KeyCode]| {
            let mut input = ButtonInput::default();
            for &key in keys {
                input.press(key);
            }
            world.insert_resource(input);
            world.run_system_once(crate::duel::advance).unwrap();
            world.run_system_once(mix).unwrap();
            world.resource::<Duel>().scene.duel_gun_requests()
        };
        let voices = |world: &mut World, car| {
            let mut entities = world
                .query::<(Entity, &Voice)>()
                .iter(world)
                .filter(
                    |(_, v)| matches!(&v.group,Some((ExclusiveVoice::MachineGuns(v),_)) if *v==car),
                )
                .map(|(entity, _)| entity)
                .collect::<Vec<_>>();
            entities.sort();
            entities
        };
        assert_eq!(step(&mut world, &[Space, Comma]), [Some(true); 2]);
        let first = voices(&mut world, cars[0]);
        let second = voices(&mut world, cars[1]);
        assert_eq!((first.len(), second.len()), (2, 2));
        for _ in 0..3 {
            assert_eq!(step(&mut world, &[Comma]), [None; 2]);
            assert_eq!(voices(&mut world, cars[0]), first);
        }
        assert_eq!(step(&mut world, &[Comma]), [Some(false), Some(true)]);
        assert!(voices(&mut world, cars[0]).is_empty());
        assert_eq!(voices(&mut world, cars[1]), second);
        let mut stopped_hot = false;
        for _ in 0..200 {
            let requests = step(&mut world, &[Comma]);
            if world.resource::<Duel>().cars[1].gun_overheated() && requests[1] == Some(false) {
                stopped_hot = true;
                break;
            }
        }
        assert!(stopped_hot);
        assert!(voices(&mut world, cars[1]).is_empty());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_saturation_steals_individual_vab_tones() {
        let mut world = crate::contest::menu_world();
        crate::duel::start(
            &mut world,
            Level::Arena,
            [Vehicle::Darkside, Vehicle::YellowJacket],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        for (key, expected, other) in [(KeyCode::KeyF, 5, 12), (KeyCode::Period, 12, 5)] {
            let mut keys = ButtonInput::default();
            keys.press(key);
            world.insert_resource(keys);
            let mut fired = false;
            for _ in 0..24 {
                world.run_system_once(crate::duel::advance).unwrap();
                let events = &world.resource::<Duel>().sounds;
                assert!(
                    !events
                        .iter()
                        .any(|r| matches!(r, BattleRequest::Play(s) if s.id == other))
                );
                let car = if expected == 5 {
                    Vehicle::Darkside
                } else {
                    Vehicle::YellowJacket
                };
                if let Some(index) = events
                    .iter()
                    .position(|r| matches!(r, BattleRequest::Play(s) if s.id == expected))
                {
                    assert!(index > 0);
                    assert!(
                        matches!(events[index-1], BattleRequest::Stop(ExclusiveVoice::SpecialLaunch(v)) if v==car)
                    );
                    assert!(
                        matches!(events[index], BattleRequest::Play(ref s) if s.exclusive==Some(ExclusiveVoice::SpecialLaunch(car)))
                    );
                    fired = true;
                }
            }
            assert!(fired, "{key:?} should use sample {expected}");
        }
        let request = |id, exclusive, volume_percent, channels, range| {
            BattleRequest::Play(BattleSound {
                frame: 0,
                id,
                exclusive,
                volume_percent,
                channels,
                range,
            })
        };
        // Program 32 contains nine crash tones. Two requests for this sound ID share one group.
        let crash = Some(ExclusiveVoice::Contact(ContactVoice::HeavyCrash));
        world.resource_mut::<Duel>().sounds = vec![
            request(20, crash, 50, [47, 25], 800),
            request(20, crash, 99, [25, 47], 400),
        ];
        world.run_system_once(mix).unwrap();
        assert_eq!(
            world
                .query_filtered::<Entity, With<DuelEffect>>()
                .iter(&world)
                .count(),
            9
        );
        use bevy::audio::Decodable;
        let mut voices =
            world.query_filtered::<(&Voice, &AudioPlayer<StereoPcm>), With<DuelEffect>>();
        for ((voice, source), tone) in voices
            .iter(&world)
            .zip(&world.resource::<DuelBank>().programs[20])
        {
            assert_eq!(voice.distance, 400);
            let clip = world
                .resource::<Assets<StereoPcm>>()
                .get(&source.0)
                .unwrap();
            for (i, (dry, wet)) in tone
                .clip
                .unscaled_decoder()
                .zip(clip.decoder())
                .take(2048)
                .enumerate()
            {
                assert_eq!(wet, dry * tone.clip.volume.gain([25, 47][i % 2], 99));
            }
        }
        // Program 19 alternates three priority-21 tones with one priority-81 tone.
        world.run_system_once(crate::duel::advance).unwrap();
        let disc = &world.resource::<crate::campaign::Campaign>().0;
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let mut engines: Vec<_> = world
            .query_filtered::<&Voice, With<DuelEngine>>()
            .iter(&world)
            .map(|v| (v.order, v.priority, v.distance))
            .collect();
        engines.sort_by_key(|v| v.0);
        let mut expected = Vec::new();
        let mut allocate = |order, priority, range: u32| {
            let record = cpu.call(0x80107280, &[]);
            if record != 0 {
                cpu.put(record + 6, &[priority]);
                cpu.put(record + 0x74, &range.to_le_bytes());
                if let Some(order) = order {
                    cpu.put(
                        0x80007000 + 4 * expected.len() as u32,
                        &record.to_le_bytes(),
                    );
                    expected.push((record, order));
                }
            }
        };
        for (order, priority, range) in engines {
            allocate(Some(order), priority, range);
        }
        // The previous crash still owns nine tone records during admission.
        for _ in 0..9 {
            allocate(None, 0, 0);
        }
        let mut serial = world.resource::<VoiceSequence>().serial;
        let mut requests = Vec::new();
        for i in 0..10 {
            let range = (i % 3) * 128;
            // In-range requests compete for hardware voices even at zero gain.
            let channels = if i % 2 == 0 { [0; 2] } else { [33; 2] };
            let percent = if i % 2 == 0 { 99 } else { 0 };
            requests.push(request(4, None, percent, channels, range));
            for tone in &world.resource::<DuelBank>().programs[4] {
                serial += 1;
                allocate(Some(serial), tone.priority, range);
            }
        }
        cpu.call(0x80104f98, &[0x80007000, 0, expected.len() as u32 - 1]);
        let mut expected: Vec<_> = (0..24)
            .map(|i| {
                let record = cpu.word(0x80007000 + 4 * i);
                expected.iter().find(|(ptr, _)| *ptr == record).unwrap().1
            })
            .collect();
        expected.sort_unstable();
        world.resource_mut::<Duel>().sounds = requests;
        world.run_system_once(mix).unwrap();
        let mut survivors: Vec<_> = world
            .query_filtered::<&Voice, With<AudioPlayer<StereoPcm>>>()
            .iter(&world)
            .map(|v| v.order)
            .collect();
        survivors.sort_unstable();
        assert_eq!(survivors, expected);
        let effects: Vec<_> = world
            .query_filtered::<Entity, With<DuelEffect>>()
            .iter(&world)
            .collect();
        for entity in effects {
            world.despawn(entity);
        }
        world.run_system_once(crate::duel::advance).unwrap();
        world.resource_mut::<Duel>().sounds.clear();
        world.run_system_once(mix).unwrap();
        let mut engines =
            world.query_filtered::<(&DuelEngine, Has<AudioPlayer<StereoPcm>>), With<Voice>>();
        let slots: Vec<_> = engines
            .iter(&world)
            .map(|(engine, audible)| {
                assert!(audible);
                engine.slot
            })
            .collect();
        assert!(slots.contains(&0) && slots.contains(&1));
    }
}
