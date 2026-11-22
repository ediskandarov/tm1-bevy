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
    let textures = Vram::from_tms(&disc.read(&format!("UA2PLAY/{stem}.TMS"))?)?;
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
    let music = crate::music::Music::level(disc, level)?
        .opening
        .ok_or_else(|| anyhow::anyhow!("missing level soundtrack"))?;
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
    if (!window.focused && !capture.demo) || capture.holding_battle_frame() {
        return;
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
            let mut pixels = [0, 0, 0, 175].repeat(320 * 240);
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
                    fonts.centered_at(&mut pixels, Face::Text, x, 64 + base, label);
                    fonts.centered_at(&mut pixels, Face::Text, x, 84 + base, &count.to_string());
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
    let result = if keys.just_pressed(KeyCode::Escape) {
        let result =
            crate::contest::show(world, Vehicle::SweetTooth, Level::Arena, difficulty, None);
        if result.is_ok() {
            world.remove_resource::<Duel>();
        }
        result
    } else if duel.winner.is_some() && keys.just_pressed(KeyCode::Enter) {
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
