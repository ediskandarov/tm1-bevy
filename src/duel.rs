//! Local keyboard battle with the two retail player records in one arena.
use crate::{
    campaign::{self, Campaign, StageEntity},
    render,
    smoke::Capture,
};
use anyhow::{Result, ensure};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use std::time::Duration;
use tm1_assets::{scene::Dmd, texture::Vram};
use tm1_core::drive::{
    CarView, CollisionScene, Difficulty, Input, Level, PlayerVehicle, Vehicle, VehicleCamera,
};

pub const STAGES: [Level; 6] = [
    Level::Arena,
    Level::Warehouse,
    Level::Freeway,
    Level::CityPark,
    Level::Cyburbia,
    Level::Rooftops,
];
pub const NAMES: [&str; 6] = [
    "Arena",
    "Warehouse",
    "Freeway",
    "City Park",
    "Canal",
    "Roof Top",
];

#[derive(Resource)]
pub struct Duel {
    pub cars: [PlayerVehicle; 2],
    pub cameras: [VehicleCamera; 2],
    pub scene: CollisionScene,
    remainder: Duration,
    score: [u32; 2],
    pub(crate) settings: crate::pause::Settings,
    winner: Option<usize>,
    score_time_left: Duration,
    pub(crate) pause: Option<crate::pause::Page>,
    control_slot: usize,
    abort_elapsed: Duration,
    wait_release: bool,
    layouts: [u8; 2],
}

pub fn start(
    world: &mut World,
    level: Level,
    kinds: [Vehicle; 2],
    difficulty: Difficulty,
    score: [u32; 2],
) -> Result<()> {
    let disc = &world.resource::<Campaign>().0;
    let stem = level
        .duel_database()
        .ok_or_else(|| anyhow::anyhow!("unavailable two-player stage"))?;
    let data = disc.read(&format!("UA2PLAY/{stem}.DMD"))?;
    let texture_data = disc.read(&format!("UA2PLAY/{stem}.TMS"))?;
    // FILEIO reads these two files at double speed while SCREEN leaves the score visible.
    let sectors = texture_data.len().div_ceil(2048) + data.len().div_ceil(2048);
    let score_time_left = Duration::from_secs_f64(sectors as f64 / 150.);
    let textures = Vram::from_tms(&texture_data)?;
    let dmd = Dmd::parse(&data)?;
    ensure!(dmd.version == textures.version, "DMD/TMS version mismatch");
    let mut geometry = dmd.extract_kind(1)?;
    if level == Level::Arena {
        geometry.polygons.extend(dmd.extract_kind(3)?.polygons);
    }
    let lights = !matches!(level, Level::Cyburbia | Level::Rooftops);
    let [first, second] = kinds.map(|v| dmd.vehicle_bodies_with_lights(v as u16, lights));
    let (first, second) = (first?, second?);
    let mut scene = CollisionScene::for_duel(
        &data,
        geometry.root_id,
        [first[0].root_id, second[0].root_id],
    )
    .ok_or_else(|| anyhow::anyhow!("unsupported duel collision graph"))?;
    let mut seed = 1;
    scene.start_pedestrians(level, &mut seed);
    let mut cars =
        PlayerVehicle::duel_players(&disc.read("SIPS_600.07")?, kinds, level, seed, difficulty)
            .ok_or_else(|| anyhow::anyhow!("missing JP rotation table"))?;
    let arena = campaign::arena_art(
        disc,
        level,
        kinds[0],
        &dmd,
        textures,
        geometry,
        first,
        vec![second],
        &scene,
    )?;
    let music = crate::music::track(disc, crate::music::opening_track(level, true))?;
    let sounds = crate::sfx::Sounds::load(disc, kinds[0], &kinds[1..])?;
    let settings = world
        .get_resource::<Duel>()
        .map(|d| d.settings)
        .or_else(|| {
            world
                .get_resource::<crate::contest::Contest>()
                .map(|c| c.settings)
        })
        .unwrap_or_default();
    let layouts = world
        .get_resource::<crate::contest::Contest>()
        .map(|menu| menu.duel_layouts)
        .or_else(|| world.get_resource::<Duel>().map(|duel| duel.layouts))
        .unwrap_or([1; 2]);
    for (car, layout) in cars.iter_mut().zip(layouts) {
        car.set_pad_layout(layout);
    }
    let cameras = cars
        .each_ref()
        .map(|car| VehicleCamera::new(car, CarView::Chase));
    campaign::clear_stage(world);
    world.remove_resource::<crate::contest::Contest>();
    world.insert_resource(arena);
    world.insert_resource(Duel {
        cars,
        cameras,
        scene,
        remainder: Duration::ZERO,
        score,
        winner: None,
        score_time_left,
        pause: None,
        control_slot: 0,
        abort_elapsed: Duration::ZERO,
        wait_release: false,
        layouts,
        settings,
    });
    world
        .run_system_once(render::setup)
        .map_err(|e| anyhow::anyhow!("render setup: {e:?}"))?;
    sounds.start_duel(world, settings.effects_gain());
    let track = world.resource_mut::<Assets<AudioSource>>().add(music);
    world.spawn((
        StageEntity,
        DuelMusic,
        AudioPlayer::new(track),
        PlaybackSettings::LOOP
            .with_volume(bevy::audio::Volume::Linear(0.35 * settings.music_gain())),
    ));
    let score_image = crate::ending::screen_image(world, vec![0; 320 * 240 * 4]);
    world.spawn((
        StageEntity,
        DuelHud,
        Sprite::from_image(score_image),
        Visibility::Hidden,
        Transform::from_xyz(0., 0., 4.),
    ));
    Ok(())
}
#[derive(Component)]
pub struct DuelHud;

#[derive(Component)]
pub struct DuelMusic;
#[derive(Component)]
pub struct DuelEffect;
#[derive(Component)]
pub struct PauseVoice;

pub fn pause_menu(
    mut commands: Commands,
    mut duel: ResMut<Duel>,
    keys: Res<ButtonInput<KeyCode>>,
    arena: Res<render::Arena>,
    window: Single<&Window>,
    time: Res<Time<Real>>,
    sounds: Res<crate::sfx::Sounds>,
    mut images: ResMut<Assets<Image>>,
    mut hud: Single<(&mut Sprite, &mut Visibility), With<DuelHud>>,
    mut audio: Query<
        (
            Entity,
            Has<DuelEffect>,
            Has<DuelMusic>,
            &mut PlaybackSettings,
            Option<&mut AudioSink>,
        ),
        (With<StageEntity>, Without<PauseVoice>),
    >,
) {
    use crate::pause::{Action, Page};
    use KeyCode::*;
    if duel.winner.is_some() {
        for (entity, ..) in &mut audio {
            commands.entity(entity).despawn();
        }
        return;
    }
    if !window.focused {
        return;
    }
    let before = duel.pause;
    if before == Some(Page::Aborted) {
        duel.abort_elapsed += time.delta();
        return;
    }
    let mut sound = None;
    let before_settings = duel.settings;
    if let Some(page) = before {
        let (up, down, left, right, select) = if page == Page::Controls {
            if duel.control_slot == 1 {
                (
                    false,
                    false,
                    keys.just_pressed(ArrowLeft),
                    keys.just_pressed(ArrowRight),
                    keys.just_pressed(Comma),
                )
            } else {
                (
                    false,
                    false,
                    keys.just_pressed(KeyA),
                    keys.just_pressed(KeyD),
                    keys.just_pressed(Space),
                )
            }
        } else {
            (
                keys.any_just_pressed([KeyW, ArrowUp]),
                keys.any_just_pressed([KeyS, ArrowDown]),
                keys.any_just_pressed([KeyA, ArrowLeft]),
                keys.any_just_pressed([KeyD, ArrowRight]),
                keys.any_just_pressed([Space, Comma]),
            )
        };
        let action = if keys.any_just_pressed([KeyP, NumpadEnter, Escape]) {
            Some(Action::Resume)
        } else if up {
            Some(Action::Up)
        } else if down {
            Some(Action::Down)
        } else if select || keys.just_pressed(Enter) {
            Some(Action::Select)
        } else if left {
            Some(Action::Left)
        } else if right {
            Some(Action::Right)
        } else {
            None
        };
        if let Some(action) = action {
            if page == Page::Choices(3) && matches!(action, Action::Select) {
                duel.control_slot = usize::from(keys.just_pressed(Comma));
                duel.settings.layout = duel.layouts[duel.control_slot];
            }
            duel.pause = crate::pause::navigate(page, action, &mut duel.settings);
            if page == Page::Controls {
                let slot = duel.control_slot;
                let layout = duel.settings.layout;
                duel.layouts[slot] = layout;
                duel.cars[slot].set_pad_layout(layout);
            }
            if before != duel.pause || before_settings != duel.settings {
                sound = Some(matches!(action, Action::Select | Action::Resume));
            }
        }
    } else if keys.any_just_pressed([KeyP, NumpadEnter]) {
        duel.pause = Some(Page::Choices(0));
    }
    let (sprite, visibility) = &mut *hud;
    let changed = before != duel.pause || before_settings != duel.settings;
    if let Some(page) = duel.pause {
        sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
        if changed || **visibility == Visibility::Hidden {
            let mut frame = arena
                .pause_art
                .frame(&arena.fonts, None, page, duel.settings);
            if page == Page::Controls {
                arena.fonts.centered(
                    &mut frame,
                    tm1_assets::screen::Face::Control,
                    32,
                    ["Player 1", "Player 2"][duel.control_slot],
                );
            }
            images.get_mut(&sprite.image).unwrap().data = Some(frame);
            **visibility = Visibility::Visible;
        }
    } else if before.is_some() {
        **visibility = Visibility::Hidden;
        duel.wait_release = true;
    }
    if duel.pause.is_some() || before.is_some() {
        for (entity, effect, music, mut playback, sink) in &mut audio {
            if effect {
                commands.entity(entity).despawn();
                continue;
            }
            playback.paused =
                duel.pause.is_some() && !(music && matches!(duel.pause, Some(Page::Audio(_))));
            playback.volume = bevy::audio::Volume::Linear(if music {
                0.35 * duel.settings.music_gain()
            } else {
                0.28 * duel.settings.effects_gain()
            });
            if let Some(mut sink) = sink {
                sink.set_volume(playback.volume);
                if playback.paused {
                    sink.pause()
                } else {
                    sink.play()
                }
            }
        }
    }
    if let Some(select) = sound {
        commands.spawn((
            StageEntity,
            PauseVoice,
            sounds.menu_voice(select),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(
                0.44 * duel.settings.effects_gain(),
            )),
        ));
    }
}

pub fn advance(
    mut commands: Commands,
    sounds: Res<crate::sfx::Sounds>,
    mut engines: Query<(&crate::sfx::DuelEngine, &mut AudioSink)>,
    mut duel: ResMut<Duel>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    capture: Res<Capture>,
    window: Single<&Window>,
    arena: Res<render::Arena>,
    mut images: ResMut<Assets<Image>>,
    mut hud: Single<(&mut Sprite, &mut Visibility), With<DuelHud>>,
) {
    use KeyCode::*;
    if duel.pause.is_some() || (!window.focused && !capture.demo) || capture.holding_battle_frame()
    {
        return;
    }
    if duel.winner.is_some() {
        duel.score_time_left = duel.score_time_left.saturating_sub(time.delta());
        hud.0.custom_size = Some(Vec2::new(window.width(), window.height()));
        return;
    }
    if duel.wait_release {
        if keys.any_pressed([Space, Comma, Enter, Escape, KeyP, NumpadEnter]) {
            return;
        }
        duel.wait_release = false;
    }
    let duel = &mut *duel;
    for (slot, k) in [KeyC, KeyO].into_iter().enumerate() {
        if keys.just_pressed(k) {
            duel.cameras[slot].cycle(true);
        }
    }
    let input = [
        [KeyW, KeyS, KeyA, KeyD, ShiftLeft, KeyT, Space, KeyF, KeyE],
        [
            ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ShiftRight, Slash, Comma, Period, KeyL,
        ],
    ]
    .map(|bindings| {
        let [
            accelerate,
            reverse,
            left,
            right,
            hard_turn,
            turbo,
            fire_guns,
            missile,
            cycle_weapon,
        ] = bindings.map(|k| keys.pressed(k));
        Input {
            accelerate,
            reverse,
            left,
            right,
            hard_turn,
            turbo,
            fire_guns,
            missile,
            cycle_weapon,
        }
    });
    duel.remainder += time.delta().min(Duration::from_millis(250));
    while duel.remainder >= Duration::from_millis(50) && duel.winner.is_none() {
        duel.remainder -= Duration::from_millis(50);
        for (car, camera) in duel.cars.iter_mut().zip(&duel.cameras) {
            car.set_battle_view(
                tm1_core::drive::PlayerView::Car(camera.mode),
                camera.fixed_pose.is_some(),
            );
        }
        duel.winner = duel.scene.step_duel(&mut duel.cars, input, 3);
        let mut effects = [[0; 5]; 122];
        let count = duel.scene.effects(&mut effects);
        let mut play = |id, volume| {
            commands.spawn((
                StageEntity,
                DuelEffect,
                sounds.centered(id),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(
                    volume * duel.settings.effects_gain(),
                )),
            ));
        };
        for &[x, y, z, kind, life] in &effects[..count] {
            if let Some(mut id) = crate::driving::effect_sample(kind, life, 13) {
                if kind == 10 && duel.cars[1].position() == [x, y, z] {
                    id = 12;
                }
                play(id, 0.44);
            }
        }
        for car in &duel.cars {
            for (voice, volume) in car.contact_sounds().into_iter().flatten() {
                play(18 + voice as usize, 0.44 * volume as f32 / 100.);
            }
        }
        for (camera, car) in duel.cameras.iter_mut().zip(&duel.cars) {
            camera.update(car, false);
        }
        if let Some(slot) = duel.winner {
            duel.score[slot] += 1;
        }
    }
    for (engine, sink) in &mut engines {
        sink.set_speed(crate::sfx::engine_speed(
            engine.range,
            duel.cars[engine.slot].rev(),
        ));
    }
    if let Some(winner) = duel.winner {
        let (sprite, visibility) = &mut *hud;
        if **visibility == Visibility::Hidden {
            use tm1_assets::screen::Face;
            let mut pixels = [0, 0, 0, 128].repeat(320 * 240);
            for (slot, car) in duel.cars.iter().enumerate() {
                let base = slot as i32 * 114;
                let face = if slot == winner {
                    Face::Selected
                } else {
                    Face::Choice
                };
                let fonts = &arena.fonts;
                fonts.centered_at(
                    &mut pixels,
                    face,
                    159,
                    40 + base,
                    if slot == winner { "WINNER!" } else { "LOSER" },
                );
                fonts.at(
                    &mut pixels,
                    face,
                    20,
                    79 + base,
                    &car.vehicle().name().to_uppercase(),
                );
                for (x, label, count) in [
                    (220, "WINS", duel.score[slot]),
                    (270, "LOSSES", duel.score[1 - slot]),
                ] {
                    fonts.centered_at(&mut pixels, Face::Control, x, 64 + base, label);
                    fonts.centered_at(&mut pixels, Face::Control, x, 84 + base, &count.to_string());
                }
            }
            images.get_mut(&sprite.image).unwrap().data = Some(pixels);
            **visibility = Visibility::Visible;
        }
        sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
    }
}

pub fn round_menu(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let duel = world.resource::<Duel>();
    let difficulty = duel.cars[0].difficulty();
    let aborted = duel.pause == Some(crate::pause::Page::Aborted)
        && duel.abort_elapsed >= Duration::from_secs_f64(31. / 60.);
    let result = if aborted
        || (duel.pause.is_none() && !duel.wait_release && keys.just_pressed(KeyCode::Escape))
    {
        let result = crate::contest::show(
            world,
            Vehicle::SweetTooth,
            Level::Arena,
            difficulty,
            aborted.then_some(crate::cinema::Clip::Title),
        );
        if result.is_ok() {
            world.remove_resource::<Duel>();
        }
        result
    } else if duel.winner.is_some()
        && (duel.score_time_left.is_zero() || keys.just_pressed(KeyCode::Enter))
    {
        start(
            world,
            duel.cars[0].level(),
            duel.cars.each_ref().map(PlayerVehicle::vehicle),
            difficulty,
            duel.score,
        )
    } else {
        return;
    };
    if let Err(e) = result {
        error!("Duel transition failed: {e:#}");
        world.write_message(AppExit::error());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_auto_rematch_after_rooftop_fall() {
        use super::*;
        use crate::contest::menu_world;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = menu_world();
        let kinds = [Vehicle::YellowJacket, Vehicle::Outlaw];
        start(&mut world, Level::Rooftops, kinds, Difficulty::Hard, [2, 3]).unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let initial_positions = world
            .resource::<Duel>()
            .cars
            .each_ref()
            .map(tm1_core::drive::PlayerVehicle::position);
        let hud_entity = world
            .query_filtered::<Entity, With<DuelHud>>()
            .single(&world)
            .unwrap();
        for tick in 0..4800 {
            let mut keys = ButtonInput::default();
            for key in [KeyCode::KeyW, KeyCode::Space, KeyCode::KeyF, KeyCode::KeyT] {
                keys.press(key);
            }
            if (tick / 35) % 5 == 1 {
                keys.press(KeyCode::KeyD);
            }
            if (tick / 65) % 3 == 2 {
                keys.press(KeyCode::KeyA);
            }
            world.insert_resource(keys);
            world.run_system_once(advance).unwrap();
            if *world.get::<Visibility>(hud_entity).unwrap() == Visibility::Visible {
                break;
            }
        }
        assert_eq!(
            *world.get::<Visibility>(hud_entity).unwrap(),
            Visibility::Visible,
            "driver did not fall"
        );
        assert!(world.resource::<Duel>().cars[0].health().0 <= 0);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.run_system_once(pause_menu).unwrap();
        assert_eq!(
            world
                .query_filtered::<Entity, With<DuelMusic>>()
                .iter(&world)
                .count(),
            0
        );
        round_menu(&mut world);
        assert!(world.get_entity(hud_entity).is_ok());
        assert_eq!(world.resource::<Duel>().score, [2, 4]);
        for _ in 0..400 {
            world.run_system_once(advance).unwrap();
            round_menu(&mut world);
            if world.get_entity(hud_entity).is_err() {
                break;
            }
        }
        assert!(world.get_entity(hud_entity).is_err());
        let round = world.resource::<Duel>();
        assert_eq!(round.score, [2, 4]);
        assert!(round.winner.is_none());
        for (slot, car) in round.cars.iter().enumerate() {
            assert_eq!(car.position(), initial_positions[slot]);
            assert_eq!(car.vehicle(), kinds[slot]);
            assert_eq!(car.health().0, car.health().1);
            assert_eq!(car.difficulty(), Difficulty::Hard);
        }
    }
}
