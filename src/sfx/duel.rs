//! Tone allocation for the shared SPU during split-screen play.
use super::{Layer, PreparedTone, TURBO_PROGRAM, battle_programs, engine_speed, pcm::StereoPcm};
use crate::{
    campaign::StageEntity,
    driving::ExclusiveVoice,
    duel::{Duel, DuelEffect},
};
use anyhow::Result;
use bevy::{audio::Volume, prelude::*};
use tm1_assets::sound::program;
use tm1_core::drive::Vehicle;

#[derive(Resource)]
pub struct DuelBank {
    programs: Vec<Vec<Layer>>,
    second_engine: Vec<Layer>,
    turbo: Vec<Layer>,
}

#[derive(Component)]
pub struct DuelEngine {
    slot: usize,
    source: Handle<StereoPcm>,
    range: f32,
}

#[derive(Component)]
pub struct Voice {
    priority: u8,
    distance: i64,
    key: Option<ExclusiveVoice>,
}

pub enum Request {
    Play {
        id: usize,
        key: Option<ExclusiveVoice>,
        volume: f32,
        position: [i32; 3],
    },
    Stop(ExclusiveVoice),
    Turbo,
}

impl DuelBank {
    pub fn load(bank: &[u8], cars: [Vehicle; 2], world: &mut World) -> Result<Self> {
        world.init_resource::<Assets<super::pcm::StereoPcm>>();
        let mut layers = |id, engine, looping| -> Result<Vec<Layer>> {
            program(bank, id)?
                .into_iter()
                .map(|tone| Ok(PreparedTone::new(tone, engine, looping)?.register(world)))
                .collect()
        };
        let programs = battle_programs(cars[0], &cars[1..])
            .enumerate()
            .map(|(i, id)| layers(id, i == 0, i <= 1))
            .collect::<Result<_>>()?;
        Ok(Self {
            programs,
            second_engine: layers(cars[1].sound_programs().0, true, true)?,
            turbo: layers(TURBO_PROGRAM, false, true)?,
        })
    }

    pub fn install(self, world: &mut World, gain: f32) {
        for (slot, layers) in [&self.programs[0], &self.second_engine]
            .into_iter()
            .enumerate()
        {
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
                        range: layer.range,
                    },
                    Voice {
                        priority: layer.priority,
                        distance: 0,
                        key: None,
                    },
                    AudioPlayer(source.clone()),
                    PlaybackSettings::ONCE.with_volume(Volume::Linear(0.28 * gain)),
                ));
            }
        }
        world.insert_resource(self);
    }

    pub fn play_score_engine(&self, commands: &mut Commands, gain: f32) {
        for layer in &self.programs[0] {
            let mut voice = commands.spawn((
                StageEntity,
                crate::duel::ScoreEngine,
                PlaybackSettings::DESPAWN
                    .with_speed(engine_speed(layer.range, 30))
                    .with_volume(Volume::Linear(0.28 * gain)),
            ));
            layer.play(&mut voice);
        }
    }

    pub fn menu_voice(&self, select: bool) -> AudioPlayer {
        AudioPlayer::new(
            self.programs[self.programs.len() - 2 + usize::from(select)][0]
                .source
                .clone(),
        )
    }
}

pub fn mix(
    mut commands: Commands,
    duel: Res<Duel>,
    bank: Res<DuelBank>,
    mut last_frame: Local<Option<u32>>,
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
    if *last_frame == Some(frame) {
        return;
    }
    *last_frame = Some(frame);
    // Keep engines eligible when a higher-priority effect temporarily takes their voices.
    let mut ranked = Vec::new();
    let mut groups = Vec::new();
    for (entity, voice, engine, _, _, _) in &playing {
        ranked.push((
            engine.is_none(),
            voice.priority,
            voice.distance,
            entity,
            engine.is_some(),
        ));
        if let Some(key) = voice.key {
            groups.push((key, entity));
        }
    }
    let gain = duel.settings.effects_gain();
    let listener = duel.cars[0].position();
    for request in &duel.sounds {
        let (layers, key, volume, position) = match *request {
            Request::Turbo => (&bank.turbo, Some(ExclusiveVoice::Turbo), 1., listener),
            Request::Play {
                id,
                key,
                volume,
                position,
            } => (&bank.programs[id], key, volume, position),
            Request::Stop(key) => {
                groups.retain(|&(group, entity)| {
                    if group != key {
                        return true;
                    }
                    ranked.retain(|entry| entry.3 != entity);
                    commands.entity(entity).despawn();
                    false
                });
                continue;
            }
        };
        let volume = Volume::Linear(0.44 * gain * volume);
        if let Some(key) = key
            && groups.iter().any(|(k, _)| *k == key)
        {
            for &(_, entity) in groups.iter().filter(|(k, _)| *k == key) {
                if let Some(entry) = ranked.iter_mut().find(|entry| entry.3 == entity) {
                    entry.0 = false;
                }
            }
            if matches!(
                key,
                ExclusiveVoice::Contact(_) | ExclusiveVoice::ScaledEffect(_)
            ) {
                for &(_, entity) in groups.iter().filter(|(k, _)| *k == key) {
                    if let Ok((_, _, _, _, mut settings, sink)) = playing.get_mut(entity) {
                        settings.volume = volume;
                        if let Some(mut sink) = sink {
                            sink.set_volume(volume);
                        }
                    } else {
                        commands
                            .entity(entity)
                            .insert(PlaybackSettings::DESPAWN.with_volume(volume));
                    }
                }
            }
            continue;
        }
        let distance = position
            .into_iter()
            .zip(listener)
            .map(|(p, l)| (i64::from(p) - i64::from(l)).pow(2))
            .sum();
        for layer in layers {
            let mut voice = commands.spawn((
                StageEntity,
                DuelEffect,
                Voice {
                    priority: layer.priority,
                    distance,
                    key,
                },
                PlaybackSettings::DESPAWN.with_volume(volume),
            ));
            layer.play(&mut voice);
            let entity = voice.id();
            ranked.push((false, layer.priority, distance, entity, false));
            if let Some(key) = key {
                groups.push((key, entity));
            }
        }
    }
    // SOUND.c fills unused slots with tones finishing from prior updates.
    ranked.sort_by_key(|&(finishing, priority, distance, _, _)| (finishing, priority, distance));
    for (rank, (_, _, _, entity, engine)) in ranked.into_iter().enumerate() {
        if engine {
            let Ok((_, _, Some(engine), audible, mut settings, sink)) = playing.get_mut(entity)
            else {
                continue;
            };
            settings.speed = engine_speed(engine.range, duel.cars[engine.slot].rev());
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
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;
    use tm1_core::drive::{ContactVoice, Difficulty, Level};

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
                .filter(|(_, v)| v.key == Some(ExclusiveVoice::MachineGuns(car)))
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
                        .any(|r| matches!(r, Request::Play{id,..} if *id == other))
                );
                fired |= events
                    .iter()
                    .any(|r| matches!(r, Request::Play{id,..} if *id == expected));
            }
            assert!(fired, "{key:?} should use sample {expected}");
        }
        let request = |id, key, volume| Request::Play {
            id,
            key,
            volume,
            position: [0; 3],
        };
        // Program 32 contains nine crash tones. Two requests for this sound ID share one group.
        let crash = Some(ExclusiveVoice::Contact(ContactVoice::HeavyCrash));
        world.resource_mut::<Duel>().sounds = vec![request(20, crash, 0.5), request(20, crash, 1.)];
        world.run_system_once(mix).unwrap();
        assert_eq!(
            world
                .query_filtered::<Entity, With<DuelEffect>>()
                .iter(&world)
                .count(),
            9
        );
        for settings in world
            .query_filtered::<&PlaybackSettings, With<DuelEffect>>()
            .iter(&world)
        {
            assert_eq!(
                settings.volume.to_linear(),
                0.44 * world.resource::<Duel>().settings.effects_gain()
            );
        }
        // Program 19 alternates three priority-21 tones with one priority-81 tone.
        world.resource_mut::<Duel>().sounds = (0..10).map(|_| request(4, None, 1.)).collect();
        world.run_system_once(mix).unwrap();
        let priorities: Vec<_> = world
            .query_filtered::<&Voice, Or<(With<AudioPlayer>, With<AudioPlayer<StereoPcm>>)>>()
            .iter(&world)
            .map(|v| v.priority)
            .collect();
        assert_eq!(priorities, vec![21; 24]);
        let effects: Vec<_> = world
            .query_filtered::<Entity, With<DuelEffect>>()
            .iter(&world)
            .collect();
        for entity in effects {
            world.despawn(entity);
        }
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
