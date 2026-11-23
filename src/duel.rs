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
    CollisionScene, Difficulty, Input, Level, PlayerVehicle, Vehicle, VehicleCamera,
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
    pub(crate) primary_cars: [crate::render::CarPose; 2],
    pub(crate) primary_view: tm1_core::drive::SoundListener,
    pub(crate) primary_hidden: bool,
    pub cars: [PlayerVehicle; 2],
    pub cameras: [VehicleCamera; 2],
    pub scene: CollisionScene,
    remainder: Duration,
    focus: DuelFocus,
    score: [i16; 2],
    pub(crate) settings: crate::pause::Settings,
    winner: Option<usize>,
    score_time_left: Duration,
    pub(crate) pause: Option<crate::pause::Page>,
    control_slot: usize,
    controls_view: crate::pause::ControlsView,
    abort_wait: crate::pause::AbortWait,
    options_idle: Duration,
    pause_input: crate::pause::PauseInput,
    just_resumed: bool,
    pub(crate) layouts: [u8; 2],
    pub(crate) sounds: Vec<crate::driving::BattleRequest>,
}

impl Duel {
    pub(crate) fn finished(&self) -> bool {
        self.winner.is_some()
    }
}

enum DuelFocus {
    Playing,
    Background(Vec<Entity>),
    Returning,
}

pub fn configure(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            update_focus,
            (
                pause_menu,
                advance,
                crate::sfx::duel::mix,
                render::duel_view,
                round_menu,
            )
                .chain()
                .run_if(accepts_input),
        )
            .chain()
            .run_if(resource_exists::<Duel>),
    );
}

fn update_focus(
    mut duel: ResMut<Duel>,
    window: Single<&Window>,
    capture: Res<Capture>,
    mut voices: Query<(Entity, &mut PlaybackSettings, Option<&AudioSink>), With<StageEntity>>,
) {
    if !window.focused && !capture.demo {
        if !matches!(duel.focus, DuelFocus::Background(_)) {
            let mut resume = Vec::new();
            for (entity, mut settings, sink) in &mut voices {
                if !settings.paused {
                    settings.paused = true;
                    if let Some(sink) = sink {
                        sink.pause();
                    }
                    resume.push(entity);
                }
            }
            duel.focus = DuelFocus::Background(resume);
            duel.remainder = Duration::ZERO;
        }
        return;
    }
    match &duel.focus {
        DuelFocus::Background(resume) => {
            for entity in resume {
                if let Ok((_, mut settings, sink)) = voices.get_mut(*entity) {
                    settings.paused = false;
                    if let Some(sink) = sink {
                        sink.play();
                    }
                }
            }
            // The first foreground delta can include the entire background interval.
            duel.focus = DuelFocus::Returning;
        }
        DuelFocus::Returning => duel.focus = DuelFocus::Playing,
        DuelFocus::Playing => {}
    }
}

fn accepts_input(duel: Option<Res<Duel>>) -> bool {
    duel.is_some_and(|duel| matches!(duel.focus, DuelFocus::Playing))
}

pub fn start(
    world: &mut World,
    level: Level,
    kinds: [Vehicle; 2],
    difficulty: Difficulty,
    score: [i16; 2],
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
    let geometry = dmd.battle_geometry(level == Level::Arena)?;
    let lights = !matches!(level, Level::Cyburbia | Level::Rooftops);
    let [first, second] = kinds.map(|v| dmd.vehicle_bodies_with_lights(v as u16, lights));
    let (first, second) = (first?, second?);
    let mut scene = CollisionScene::for_duel(
        &data,
        geometry.root_id,
        [first[0].root_id, second[0].root_id],
    )
    .ok_or_else(|| anyhow::anyhow!("unsupported duel collision graph"))?;
    let pedestrian_seed = world
        .get_resource::<Duel>()
        .map(|previous| previous.cars[0].rng_state())
        .unwrap_or_else(|| crate::campaign::round_ticks(world));
    let mut seed = pedestrian_seed;
    scene.start_pedestrians(level, &mut seed);
    let mut cars = if let Some(previous) = world.get_resource::<Duel>().filter(|previous| {
        previous.finished()
            && previous.cars[0].level() == level
            && previous.cars[0].difficulty() == difficulty
            && previous.cars.each_ref().map(PlayerVehicle::vehicle) == kinds
    }) {
        PlayerVehicle::duel_rematch(&previous.cars, seed)
    } else {
        PlayerVehicle::duel_players(&disc.read("SIPS_600.07")?, kinds, level, seed, difficulty)
            .ok_or_else(|| anyhow::anyhow!("missing JP rotation table"))?
    };
    let setup_seed = crate::campaign::round_ticks(world);
    PlayerVehicle::initialize_duel_level(&mut cars, setup_seed);
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
    let music = crate::music::CdTrack::load(disc, crate::music::opening_track(level, true))?;
    let bank = disc.read("SND/UACORE.VAB")?;
    let settings = world
        .get_resource::<Duel>()
        .map(|d| d.settings)
        .or_else(|| {
            world
                .get_resource::<crate::contest::Contest>()
                .map(|c| c.settings)
        })
        .unwrap_or_default();
    let sounds = crate::sfx::duel::DuelBank::load(&bank, kinds, settings.effects_volume(), world)?;
    let layouts = world
        .get_resource::<crate::contest::Contest>()
        .map(crate::contest::Contest::duel_layouts)
        .or_else(|| world.get_resource::<Duel>().map(|duel| duel.layouts))
        .unwrap_or([1; 2]);
    for (car, layout) in cars.iter_mut().zip(layouts) {
        car.set_pad_layout(layout);
    }
    let cameras = cars.each_ref().map(VehicleCamera::duel);
    crate::contest::remember_battle_cars(world, &kinds);
    campaign::clear_stage(world);
    world.remove_resource::<crate::contest::Contest>();
    world.insert_resource(arena);
    world.insert_resource(campaign::stage_clear_color(level));
    // InitLevel resets the shared BIOS generator once both cars are ready.
    let battle_seed = crate::campaign::round_ticks(world);
    info!("Duel seeds: pedestrians={pedestrian_seed}, setup={setup_seed}, battle={battle_seed}");
    for car in &mut cars {
        car.set_round_seed(battle_seed);
    }
    // shellInitLevel initializes one shared set of windshield holes in both modes.
    cars[0].windshield_holes();
    let opening_seed = cars[0].rng_state();
    cars[1].set_rng_state(opening_seed);
    world.insert_resource(Duel {
        primary_cars: cars.each_ref().map(crate::render::CarPose::player),
        primary_view: tm1_core::drive::SoundListener::new(&cars[0], &cameras[0], None),
        primary_hidden: !cameras[0].show_body,
        cars,
        cameras,
        scene,
        remainder: Duration::ZERO,
        focus: DuelFocus::Playing,
        score,
        winner: None,
        score_time_left,
        pause: None,
        control_slot: 0,
        controls_view: Default::default(),
        abort_wait: Default::default(),
        options_idle: Duration::ZERO,
        pause_input: crate::pause::PauseInput::Ready,
        just_resumed: false,
        layouts,
        sounds: Vec::new(),
        settings,
    });
    world
        .run_system_once(render::setup)
        .map_err(|e| anyhow::anyhow!("render setup: {e:?}"))?;
    sounds.install(world);
    let track = world
        .get_resource_or_init::<Assets<crate::music::CdTrack>>()
        .add(music);
    world.spawn((
        StageEntity,
        DuelMusic,
        AudioPlayer(track),
        PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(settings.cd_gain())),
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
#[derive(Component)]
pub struct ScoreEngine;

pub fn pause_menu(
    mut commands: Commands,
    mut duel: ResMut<Duel>,
    keys: Res<ButtonInput<KeyCode>>,
    arena: Res<render::Arena>,
    window: Single<&Window>,
    time: Res<Time<Real>>,
    sounds: Res<crate::sfx::duel::DuelBank>,
    tracks: Res<Assets<crate::music::CdTrack>>,
    mut images: ResMut<Assets<Image>>,
    mut hud: Single<(&mut Sprite, &mut Visibility), With<DuelHud>>,
    mut audio: Query<
        (
            Entity,
            Has<DuelEffect>,
            Option<&AudioPlayer<crate::music::CdTrack>>,
            Has<AudioPlayer<crate::sfx::pcm::StereoPcm>>,
            &mut PlaybackSettings,
            Option<&mut AudioSink>,
        ),
        (With<StageEntity>, Without<PauseVoice>, Without<ScoreEngine>),
    >,
) {
    use crate::pause::{Action, Page, PauseInput};
    use KeyCode::*;
    if duel.winner.is_some() {
        for (entity, ..) in &mut audio {
            commands.entity(entity).despawn();
        }
        return;
    }
    let before = duel.pause;
    let held = crate::pause::duel_pad_held(&keys, 0)
        || crate::pause::duel_pad_held(&keys, 1)
        || keys.any_pressed([Enter, Escape]);
    if duel.pause_input != PauseInput::Ready {
        if held {
            return;
        }
        let pending = duel.pause_input;
        duel.pause_input = PauseInput::Ready;
        if let PauseInput::ChangeOnRelease(next) = pending {
            duel.pause = next;
        } else {
            return;
        }
    }
    if before == Some(Page::Aborted) {
        duel.abort_wait.advance(
            time.delta(),
            keys.any_pressed([Space, Comma, Enter, KeyP, NumpadEnter]),
        );
        return;
    }
    let mut sound = None;
    let before_settings = duel.settings;
    let mut controls_changed = false;
    if let Some(page) = duel.pause.filter(|_| duel.pause == before) {
        let (action, idle_reset) = if page == Page::Controls {
            let slot = duel.control_slot;
            let input = duel.controls_view.advance(time.delta(), &keys, Some(slot));
            controls_changed = input.redraw;
            let resume = if slot == 0 { KeyP } else { NumpadEnter };
            (
                if keys.just_pressed(resume) {
                    Some(Action::Resume)
                } else {
                    input.action
                },
                input.reset_idle,
            )
        } else {
            let action = if keys.any_just_pressed([KeyP, NumpadEnter, Escape]) {
                Some(Action::Resume)
            } else if keys.any_just_pressed([KeyW, ArrowUp]) {
                Some(Action::Up)
            } else if keys.any_just_pressed([KeyS, ArrowDown]) {
                Some(Action::Down)
            } else if matches!(page, Page::Audio(_)) && keys.any_pressed([KeyA, ArrowLeft]) {
                Some(Action::Left)
            } else if matches!(page, Page::Audio(_)) && keys.any_pressed([KeyD, ArrowRight]) {
                Some(Action::Right)
            } else if keys.any_pressed([Space, Comma, Enter]) {
                Some(Action::Select)
            } else {
                None
            };
            (action, action.is_some())
        };
        if page != Page::Controls && held {
            duel.pause_input = PauseInput::WaitRelease;
        }
        let expired =
            crate::pause::options_timed_out(page, &mut duel.options_idle, time.delta(), idle_reset);
        if let Some(action) = action.or(expired.then_some(Action::Resume)) {
            if page == Page::Choices(3) && matches!(action, Action::Select) {
                duel.control_slot = usize::from(crate::pause::duel_pad_held(&keys, 1));
                duel.settings.layout = duel.layouts[duel.control_slot];
            }
            duel.pause = crate::pause::navigate(page, action, &mut duel.settings);
            if page == Page::Controls {
                let slot = duel.control_slot;
                let layout = duel.settings.layout;
                duel.layouts[slot] = layout;
                duel.cars[slot].set_pad_layout(layout);
            }
            let next = duel.pause;
            duel.pause = duel.pause_input.defer_transition(page, next, held);
            if !expired {
                sound = crate::pause::navigation_sound(
                    page,
                    action,
                    before_settings.layout != duel.settings.layout,
                );
            }
        }
    } else if keys.any_just_pressed([KeyP, NumpadEnter]) {
        duel.pause = Some(Page::Choices(0));
        duel.pause_input = PauseInput::WaitRelease;
    }
    let (sprite, visibility) = &mut *hud;
    if duel.pause == Some(Page::Controls) && before != duel.pause {
        duel.controls_view = Default::default();
        duel.pause_input = PauseInput::Ready;
    }
    if before_settings.effects_volume() != duel.settings.effects_volume() {
        sounds.set_engine_volume(duel.settings.effects_volume());
    }
    let changed = before != duel.pause || before_settings != duel.settings || controls_changed;
    if let Some(page) = duel.pause {
        sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
        if changed || **visibility == Visibility::Hidden {
            let frame = if page == Page::Controls {
                duel.controls_view.frame(
                    &arena.pause_art,
                    &arena.fonts,
                    duel.settings.layout,
                    Some(duel.control_slot),
                )
            } else {
                arena
                    .pause_art
                    .frame(&arena.fonts, None, page, duel.settings)
            };
            images.get_mut(&sprite.image).unwrap().data = Some(frame);
            **visibility = Visibility::Visible;
        }
    } else if before.is_some() {
        **visibility = Visibility::Hidden;
        duel.just_resumed = true;
    }
    if duel.pause.is_some() || before.is_some() {
        for (entity, effect, music, engine_playing, mut playback, sink) in &mut audio {
            if effect {
                commands.entity(entity).despawn();
                continue;
            }
            if engine_playing && duel.pause.is_some() {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer<crate::sfx::pcm::StereoPcm>, AudioSink)>();
            }
            playback.paused = duel.pause.is_some()
                && !(music.is_some() && matches!(duel.pause, Some(Page::Audio(_))));
            playback.volume = bevy::audio::Volume::Linear(if music.is_some() {
                duel.settings.cd_gain()
            } else {
                0.28
            });
            if matches!(duel.pause, Some(Page::Audio(_)))
                && let Some(track) = music.and_then(|p| tracks.get(&p.0))
            {
                track.extend_loop(true);
            }
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
        let player = sounds.menu_voice(select);
        let gain = duel.settings.effects_gain();
        commands.queue(move |world: &mut World| {
            let entity = crate::sfx::play_menu_button(world, select, player, gain);
            world.entity_mut(entity).insert((StageEntity, PauseVoice));
        });
    }
}

pub fn advance(
    mut commands: Commands,
    bank: Res<crate::sfx::duel::DuelBank>,
    mut duel: ResMut<Duel>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    capture: Res<Capture>,
    window: Single<&Window>,
    mut arena: ResMut<render::Arena>,
    mut images: ResMut<Assets<Image>>,
    mut hud: Single<(&mut Sprite, &mut Visibility), With<DuelHud>>,
) {
    use KeyCode::*;
    duel.sounds.clear();
    if duel.pause.is_some() || capture.holding_battle_frame() {
        return;
    }
    if duel.winner.is_some() {
        duel.score_time_left = duel.score_time_left.saturating_sub(time.delta());
        hud.0.custom_size = Some(Vec2::new(window.width(), window.height()));
        return;
    }
    if std::mem::take(&mut duel.just_resumed) {
        return;
    }
    let duel = &mut *duel;
    let input = [
        [
            KeyW, KeyS, KeyA, KeyD, ShiftLeft, KeyT, Space, KeyF, KeyE, KeyQ,
        ],
        [
            ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ShiftRight, Slash, Comma, Period, KeyL, KeyK,
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
            previous_weapon,
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
            previous_weapon,
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
        let listener = tm1_core::drive::SoundListener::new(&duel.cars[0], &duel.cameras[0], None);
        duel.primary_cars = duel.cars.each_ref().map(crate::render::CarPose::player);
        duel.primary_view = listener;
        duel.primary_hidden = !duel.cameras[0].show_body;
        let previous_view =
            tm1_core::drive::SoundListener::new(&duel.cars[1], &duel.cameras[1], None);
        duel.winner = duel.scene.step_duel(&mut duel.cars, input, 3);
        crate::driving::battle_audio(
            duel.cars[0].frame(),
            &duel.scene,
            &listener,
            duel.settings.effects_volume(),
            13,
            &previous_view,
            true,
            |slot| duel.cars[slot].vehicle(),
            |request| duel.sounds.push(request),
        );
        for ((camera, car), updated) in duel
            .cameras
            .iter_mut()
            .zip(&duel.cars)
            .zip(duel.scene.duel_updated_players())
        {
            camera.after_step(car, updated);
        }
        if arena.has_animations {
            arena.advance_view_animation(
                &duel.cars[0],
                &duel.scene,
                false,
                &duel.primary_cars,
                duel.primary_hidden.then_some(0),
                true,
                duel.primary_view,
            );
            let poses = duel.cars.each_ref().map(render::CarPose::player);
            let view = tm1_core::drive::SoundListener::new(&duel.cars[1], &duel.cameras[1], None);
            arena.advance_view_animation(
                &duel.cars[1],
                &duel.scene,
                true,
                &poses,
                (!duel.cameras[1].show_body).then_some(1),
                true,
                view,
            );
        }
        if let Some(slot) = duel.winner {
            duel.score[slot] = duel.score[slot].wrapping_add(1);
            bank.play_score_engine(&mut commands);
        }
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
                    fonts.car_name(car.vehicle().card_index()),
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
    let aborted = duel.pause == Some(crate::pause::Page::Aborted) && duel.abort_wait.finished;
    let result = if aborted
        || (duel.pause.is_none() && !duel.just_resumed && keys.just_pressed(KeyCode::Escape))
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
            world.remove_resource::<crate::sfx::duel::DuelBank>();
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
    fn keyboard_weapon_selection_matches_both_retail_directions() {
        use super::*;
        use KeyCode::*;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = crate::contest::menu_world();
        let exe = world
            .resource::<crate::campaign::Campaign>()
            .0
            .read("SIPS_600.07")
            .unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        start(
            &mut world,
            Level::Arena,
            [Vehicle::Outlaw, Vehicle::Darkside],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        for layout in 0..4 {
            for (slot, car) in world.resource_mut::<Duel>().cars.iter_mut().enumerate() {
                car.set_pad_layout((layout + slot as u8) % 4);
            }
            let mut changed = [false; 2];
            for mask in [0, 1, 1, 0, 1, 0, 3, 2, 0, 2, 0, 2, 0, 2, 0, 1] {
                let mut keys = ButtonInput::default();
                let mut expected = [0; 2];
                for (slot, car) in world.resource::<Duel>().cars.iter().enumerate() {
                    let bits = if slot == 0 {
                        mask
                    } else {
                        (mask & 1) << 1 | (mask & 2) >> 1
                    };
                    let [forward, backward] = [bits & 1 != 0, bits & 2 != 0];
                    let bindings = [[KeyE, KeyQ], [KeyL, KeyK]][slot];
                    if forward {
                        keys.press(bindings[0]);
                    }
                    if backward {
                        keys.press(bindings[1]);
                    }
                    let at = 0x8019ee40 + 0x4a4 * slot as u32;
                    cpu.put(at, car.state());
                    cpu.put(at + 0xc8 + 11, &[u8::from(forward), u8::from(backward)]);
                    cpu.call(0x801315e8, &[at]);
                    expected[slot] = cpu.word(at + 0x460);
                    changed[slot] |=
                        backward && !forward && expected[slot] != car.dashboard_weapon().0 as u32;
                }
                world.insert_resource(keys);
                world.run_system_once(advance).unwrap();
                for (slot, car) in world.resource::<Duel>().cars.iter().enumerate() {
                    assert_eq!(
                        car.dashboard_weapon().0 as u32,
                        expected[slot],
                        "layout {layout} slot {slot} mask {mask}"
                    );
                }
            }
            assert_eq!(changed, [true; 2], "backward selection on layout {layout}");
        }
    }

    #[test]
    fn round_clock_wraps_and_capture_is_repeatable() {
        use super::*;
        let mut world = World::new();
        world.init_resource::<Time<Real>>();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(1_100_250));
        assert_eq!(
            crate::campaign::round_ticks(&world),
            4_658_018_400u64 as u32
        );
        world.insert_resource(Capture::new(None, true, false));
        assert_eq!(crate::campaign::round_ticks(&world), 1);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_focus_preserves_simulation_and_voices() {
        use super::*;
        use KeyCode::*;
        let mut world = crate::contest::menu_world();
        start(
            &mut world,
            Level::Warehouse,
            [Vehicle::Outlaw, Vehicle::RoadKill],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(Capture::new(None, false, false));
        let mut schedule = Schedule::default();
        configure(&mut schedule);
        let mut tick = |world: &mut World, focused, millis, pressed: &[KeyCode]| {
            world
                .query::<&mut Window>()
                .single_mut(world)
                .unwrap()
                .focused = focused;
            let mut keys = ButtonInput::default();
            for &key in pressed {
                keys.press(key);
            }
            world.insert_resource(keys);
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(millis));
            schedule.run(world);
        };
        let clocks = |world: &World| {
            world
                .resource::<Duel>()
                .cars
                .each_ref()
                .map(PlayerVehicle::frame)
        };
        let voices = |world: &mut World| {
            world
                .query_filtered::<(Entity, &PlaybackSettings), With<StageEntity>>()
                .iter(world)
                .map(|(e, s)| (e, s.paused))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        tick(&mut world, true, 50, &[KeyW, ArrowUp, Space, Comma]);
        assert!(
            world
                .query_filtered::<Entity, With<DuelEffect>>()
                .iter(&world)
                .count()
                >= 4
        );
        let frame = clocks(&world);
        let playing = voices(&mut world);
        assert!(playing.values().all(|paused| !paused));
        tick(&mut world, false, 50, &[Escape]);
        tick(&mut world, false, 20_000, &[Enter, KeyP]);
        assert_eq!(frame, clocks(&world));
        let suspended = voices(&mut world);
        assert!(suspended.keys().eq(playing.keys()));
        assert!(suspended.values().all(|paused| *paused));
        tick(&mut world, true, 20_000, &[Escape]);
        assert_eq!(frame, clocks(&world));
        assert_eq!(voices(&mut world), playing);
        tick(&mut world, true, 50, &[]);
        assert_eq!(clocks(&world), frame.map(|f| f + 1));
        for key in [KeyP, ArrowDown, ArrowDown] {
            tick(&mut world, true, 50, &[key]);
            tick(&mut world, true, 0, &[]);
        }
        tick(&mut world, true, 50, &[Enter, Slash]);
        tick(&mut world, true, 500, &[Slash]);
        assert_eq!(
            world.resource::<Duel>().pause,
            Some(crate::pause::Page::Choices(2))
        );
        let track = world
            .query_filtered::<&PlaybackSettings, With<AudioPlayer<crate::music::CdTrack>>>()
            .single(&world)
            .unwrap();
        assert!(track.paused);
        tick(&mut world, true, 0, &[]);
        assert_eq!(
            world.resource::<Duel>().pause,
            Some(crate::pause::Page::Audio(0))
        );
        let menu = voices(&mut world);
        assert!(menu.values().any(|paused| *paused) && menu.values().any(|paused| !paused));
        tick(&mut world, false, 400_000, &[KeyP]);
        assert!(voices(&mut world).values().all(|paused| *paused));
        tick(&mut world, true, 400_000, &[Enter]);
        assert_eq!(voices(&mut world), menu);
        assert_eq!(
            world.resource::<Duel>().pause,
            Some(crate::pause::Page::Audio(0))
        );
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_auto_rematch_after_rooftop_fall() {
        use super::*;
        use crate::contest::menu_world;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = menu_world();
        let kinds = [Vehicle::YellowJacket, Vehicle::Outlaw];
        world.insert_resource(ClearColor(Color::WHITE));
        start(&mut world, Level::Rooftops, kinds, Difficulty::Hard, [2, 3]).unwrap();
        assert_eq!(world.resource::<ClearColor>().0, Color::srgb_u8(0, 16, 0));
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
        let mut stopped = false;
        let mut restarted = false;
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
            let before = world
                .resource::<Duel>()
                .cars
                .each_ref()
                .map(crate::render::CarPose::player);
            world.run_system_once(advance).unwrap();
            assert_eq!(world.resource::<Duel>().primary_cars, before);
            world.run_system_once(crate::sfx::duel::mix).unwrap();
            let requests_stop = world
                .resource::<Duel>()
                .sounds
                .iter()
                .any(|r| matches!(r, crate::driving::BattleRequest::EngineStop(0)));
            let audible = world.query_filtered::<&crate::sfx::duel::DuelEngine, With<AudioPlayer<crate::sfx::pcm::StereoPcm>>>().iter(&world).any(|engine| engine.slot == 0);
            if requests_stop {
                assert!(!audible, "wreck kept an active engine");
                stopped = true;
            } else if stopped && audible {
                restarted = true;
            }
            if *world.get::<Visibility>(hud_entity).unwrap() == Visibility::Visible {
                break;
            }
        }
        assert!(stopped && restarted);
        assert_eq!(
            *world.get::<Visibility>(hud_entity).unwrap(),
            Visibility::Visible,
            "driver did not fall"
        );
        assert!(world.resource::<Duel>().cars[0].health().0 <= 0);
        let tire_positions = |car: &PlayerVehicle| {
            std::array::from_fn::<[u8; 8], 4, _>(|i| {
                let start = 0x270 + i * 0x44 + 0x38;
                car.state()[start..start + 8].try_into().unwrap()
            })
        };
        let last_tires = world.resource::<Duel>().cars.each_ref().map(tire_positions);
        assert_ne!(last_tires[0], [[0; 8]; 4]);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.run_system_once(pause_menu).unwrap();
        assert_eq!(
            world
                .query_filtered::<Entity, With<DuelMusic>>()
                .iter(&world)
                .count(),
            0
        );
        let score_voices: Vec<_> = world
            .query_filtered::<Entity, With<ScoreEngine>>()
            .iter(&world)
            .collect();
        assert!(!score_voices.is_empty());
        world.run_system_once(pause_menu).unwrap();
        assert_eq!(
            world
                .query_filtered::<Entity, With<ScoreEngine>>()
                .iter(&world)
                .collect::<Vec<_>>(),
            score_voices
        );
        let previous_seed = world.resource::<Duel>().cars[0].rng_state();
        let remaining = world.resource::<Duel>().score_time_left;
        let mut schedule = Schedule::default();
        configure(&mut schedule);
        world
            .query::<&mut Window>()
            .single_mut(&mut world)
            .unwrap()
            .focused = false;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(60));
        schedule.run(&mut world);
        assert!(
            score_voices
                .iter()
                .all(|&e| world.get::<PlaybackSettings>(e).unwrap().paused)
        );
        world
            .query::<&mut Window>()
            .single_mut(&mut world)
            .unwrap()
            .focused = true;
        schedule.run(&mut world);
        assert!(
            score_voices
                .iter()
                .all(|&e| !world.get::<PlaybackSettings>(e).unwrap().paused)
        );
        assert_eq!(world.resource::<Duel>().score_time_left, remaining);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        let last_instant = Duration::from_nanos(1);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(remaining - last_instant);
        world.run_system_once(advance).unwrap();
        round_menu(&mut world);
        assert!(world.get_entity(hud_entity).is_ok());
        assert_eq!(world.resource::<Duel>().score_time_left, last_instant);
        assert_eq!(world.resource::<Duel>().score, [2, 4]);
        world.resource_mut::<Time<Real>>().advance_by(last_instant);
        world.run_system_once(advance).unwrap();
        round_menu(&mut world);
        assert!(world.get_entity(hud_entity).is_err());
        assert!(
            score_voices
                .iter()
                .all(|&entity| world.get_entity(entity).is_err())
        );
        let round = world.resource::<Duel>();
        assert_eq!(round.score, [2, 4]);
        assert_eq!(round.cars.each_ref().map(tire_positions), last_tires);
        let mut expected_seed = crate::campaign::round_ticks(&world);
        for _ in 0..12 {
            tm1_core::drive::bios_rand(&mut expected_seed);
        }
        assert_ne!(expected_seed, previous_seed);
        assert_eq!(
            round.cars.each_ref().map(PlayerVehicle::rng_state),
            [expected_seed; 2]
        );
        assert!(round.winner.is_none());
        for (slot, car) in round.cars.iter().enumerate() {
            assert_eq!(car.position(), initial_positions[slot]);
            assert_eq!(car.vehicle(), kinds[slot]);
            assert_eq!(car.health().0, car.health().1);
            assert_eq!(car.difficulty(), Difficulty::Hard);
        }
    }
}
