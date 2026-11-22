//! Tone allocation for the shared SPU during split-screen play.
use super::{
    Candidate, EffectTone, Layer, PreparedTone, Sample, TURBO_PROGRAM, VoiceSequence,
    battle_programs, engine_speed, pcm::Levels, pcm::StereoPcm, rank_voices,
};
use crate::{
    campaign::StageEntity,
    driving::{BattleRequest, ExclusiveVoice},
    duel::{Duel, DuelEffect},
};
use anyhow::Result;
use bevy::{audio::Volume, prelude::*};
use tm1_assets::sound::program;
use tm1_core::drive::Vehicle;

#[derive(Resource)]
pub struct DuelBank {
    programs: Vec<Vec<EffectTone>>,
    engines: [Vec<Layer>; 2],
    turbo: Vec<EffectTone>,
    menu: [Handle<AudioSource>; 2],
}

#[derive(Component)]
pub struct DuelEngine {
    slot: usize,
    source: Handle<StereoPcm>,
    pitch: super::EnginePitch,
}

#[derive(Component)]
pub struct Voice {
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
    pub fn load(bank: &[u8], cars: [Vehicle; 2], world: &mut World) -> Result<Self> {
        world.init_resource::<Assets<super::pcm::StereoPcm>>();
        let mut engines = |car: Vehicle| -> Result<Vec<Layer>> {
            program(bank, car.sound_programs().0)?
                .into_iter()
                .map(|tone| Ok(PreparedTone::new(tone, true, true)?.register(world)))
                .collect()
        };
        let engines = [engines(cars[0])?, engines(cars[1])?];
        let programs = battle_programs(cars[0], &cars[1..])
            .enumerate()
            .map(|(i, id)| EffectTone::load(bank, id, i == 1 || id == 38))
            .collect::<Result<_>>()?;
        let mut menu = |id| -> Result<Handle<AudioSource>> {
            Ok(world
                .resource_mut::<Assets<AudioSource>>()
                .add(Sample::load(bank, id, false)?.wave()?))
        };
        Ok(Self {
            programs,
            engines,
            turbo: EffectTone::load(bank, TURBO_PROGRAM, true)?,
            menu: [menu(57)?, menu(58)?],
        })
    }

    pub fn install(self, world: &mut World, gain: f32) {
        let mut sequence = VoiceSequence::default();
        for (slot, layers) in self.engines.iter().enumerate() {
            for layer in layers {
                let source = layer
                    .looping_clip
                    .as_ref()
                    .expect("engine loop validated during load");
                world.spawn((
                    StageEntity,
                    DuelEngine {
                        slot,
                        source: source.clone(),
                        pitch: layer.pitch,
                    },
                    Voice {
                        order: sequence.next(),
                        priority: layer.priority,
                        distance: 0,
                        group: None,
                    },
                    AudioPlayer(source.clone()),
                    PlaybackSettings::ONCE.with_volume(Volume::Linear(0.28 * gain)),
                ));
            }
        }
        world.insert_resource(sequence);
        world.insert_resource(self);
    }

    pub fn play_score_engine(&self, commands: &mut Commands, gain: f32) {
        for layer in &self.engines[0] {
            let mut voice = commands.spawn((
                StageEntity,
                crate::duel::ScoreEngine,
                PlaybackSettings::DESPAWN
                    .with_speed(engine_speed(layer.pitch, 30))
                    .with_volume(Volume::Linear(0.28 * gain)),
            ));
            layer.play(&mut voice);
        }
    }

    pub fn menu_voice(&self, select: bool) -> AudioPlayer {
        AudioPlayer::new(self.menu[usize::from(select)].clone())
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
        &Voice,
        Option<&DuelEngine>,
        Has<AudioPlayer<StereoPcm>>,
        &mut PlaybackSettings,
        Option<&mut AudioSink>,
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
    for (entity, voice, engine, _, _, _) in &playing {
        ranked.push(Candidate {
            entity,
            order: voice.order,
            finishing: engine.is_none(),
            priority: voice.priority,
            range: voice.distance,
            engine: engine.is_some(),
        });
        if let Some((key, levels)) = &voice.group {
            groups.push((*key, entity, levels.clone()));
        }
    }
    for request in &duel.sounds {
        let (layers, key, channels, percent, distance) = match request {
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
                    commands.entity(*entity).despawn();
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
        if channels == [0; 2] || percent <= 0 {
            continue;
        }
        let levels = Levels::new(channels, percent);
        for tone in layers {
            let order = sequence.next();
            let entity = commands
                .spawn((
                    StageEntity,
                    DuelEffect,
                    Voice {
                        order,
                        priority: tone.priority,
                        distance,
                        group: key.map(|key| (key, levels.clone())),
                    },
                    AudioPlayer(clips.add(tone.clip.with_levels(levels.clone()))),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.44)),
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
    if ranked.len() > 24 {
        rank_voices(&mut ranked);
    }
    for (rank, candidate) in ranked.into_iter().enumerate() {
        let entity = candidate.entity;
        if candidate.engine {
            let Ok((_, _, Some(engine), audible, mut settings, sink)) = playing.get_mut(entity)
            else {
                continue;
            };
            settings.speed = engine_speed(engine.pitch, duel.cars[engine.slot].rev());
            if rank < 24 {
                if !audible {
                    commands
                        .entity(entity)
                        .insert(AudioPlayer(engine.source.clone()));
                } else if let Some(sink) = sink {
                    sink.set_speed(settings.speed);
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
            let boundary = duel.scene.duel_events_before_primary_view();
            for (index, event) in duel.scene.battle_events(&views[0]).enumerate() {
                let (id, pos) = match event {
                    BattleEvent::Guns {
                        slot: 1,
                        firing: true,
                        position,
                    } if !checked_guns => (1, position),
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
                } else {
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
                        .decoder()
                        .zip(clip.decoder())
                        .take(4096)
                        .enumerate()
                    {
                        nonzero |= dry != 0.;
                        assert_eq!(wet, dry * (gain[i % 2] as f32 / 33.));
                    }
                    assert!(nonzero);
                }
            }
            if checked_missile && checked_guns {
                break;
            }
        }
        assert!(checked_guns && checked_missile);
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
                .decoder()
                .zip(clip.decoder())
                .take(2048)
                .enumerate()
            {
                assert_eq!(wet, dry * ([25., 47.][i % 2] / 33.));
            }
        }
        // Program 19 alternates three priority-21 tones with one priority-81 tone.
        world.run_system_once(crate::duel::advance).unwrap();
        let mut expected: Vec<_> = world
            .query_filtered::<&Voice, With<DuelEngine>>()
            .iter(&world)
            .map(|v| (v.order, v.priority, v.distance))
            .collect();
        expected.sort_by_key(|v| v.0);
        let mut serial = world.resource::<VoiceSequence>().serial;
        let mut requests = Vec::new();
        for i in 0..10 {
            let range = (i % 3) * 128;
            requests.push(request(4, None, 99, [33; 2], range));
            for tone in &world.resource::<DuelBank>().programs[4] {
                serial += 1;
                expected.push((serial, tone.priority, range));
            }
        }
        tm1_core::sound::sort_requested(&mut expected, |v| v.2, |v| v.1);
        let mut expected: Vec<_> = expected[..24].iter().map(|v| v.0).collect();
        expected.sort_unstable();
        world.resource_mut::<Duel>().sounds = requests;
        world.run_system_once(mix).unwrap();
        let mut survivors: Vec<_> = world
            .query_filtered::<&Voice, With<AudioPlayer<StereoPcm>>>()
            .iter(&world)
            .map(|v| {
                assert_eq!(v.priority, 21);
                v.order
            })
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
