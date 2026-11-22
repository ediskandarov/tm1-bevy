use bevy::prelude::*;
use std::time::Duration;
use tm1_core::drive::{CollisionScene, Input, Opponent, PlayerVehicle, Vehicle};

#[derive(Resource)]
pub struct Drive {
    pub player: PlayerVehicle,
    pub deck: tm1_core::drive::ArenaDeck,
    pub opponents: Vec<Opponent>,
    pub contacts: CollisionScene,
    pub sounds: Vec<(usize, [i32; 3])>,
    remainder: Duration,
    pub paused: bool,
    pub restart_count: u32,
    was_focused: bool,
    victory_fields: u16,
    boss_intro: Option<Duration>,
    minion_at_start: bool,
    pub lives: u8,
    death_fields: u16,
    lost_life: Option<Duration>,
}
impl Drive {
    pub fn cleared(&self) -> bool {
        !self.awaiting_minion() && self.opponents.iter().all(|ai| ai.health().0 <= 0)
    }
    fn awaiting_minion(&self) -> bool {
        self.player.level() == tm1_core::drive::Level::Rooftops && self.opponents.len() == 3
    }
    pub fn recharge_sound(&self) -> usize {
        12 + self.opponents.len() + usize::from(self.awaiting_minion())
    }
    pub fn stage_finished(&self) -> bool {
        self.victory_fields > 180 && self.cleared()
    }
    fn advance_victory_delay(&mut self, fields: u16) {
        self.victory_fields += fields;
        if self.victory_fields > 180 && self.awaiting_minion() {
            self.boss_intro = Some(Duration::from_secs(5));
            self.victory_fields = 0;
        }
    }
    pub fn start_with_minion(&mut self) {
        self.minion_at_start = true;
        self.boss_intro = Some(Duration::from_secs(5));
    }
    pub fn showing_boss_intro(&self) -> bool {
        self.boss_intro.is_some()
    }
    fn tick_boss_intro(&mut self, elapsed: Duration, selected: bool) {
        let Some(time) = self.boss_intro else { return };
        let left = time.saturating_sub(elapsed);
        if selected || left.is_zero() {
            self.boss_intro = None;
            self.opponents
                .push(Opponent::spawn_minion(&mut self.player));
        } else {
            self.boss_intro = Some(left);
        }
    }
    pub fn lost_life_elapsed(&self) -> Option<Duration> {
        self.lost_life
    }
    fn advance_death(&mut self, fields: u16) {
        self.death_fields += fields;
        if self.death_fields > 180 {
            self.lives -= 1;
            self.death_fields = 0;
            self.victory_fields = 0;
            if self.lives > 0 {
                self.lost_life = Some(Duration::ZERO);
            }
        }
    }
    fn tick_lost_life(&mut self, delta: Duration, selected: bool) {
        let elapsed = self.lost_life.unwrap() + delta;
        if elapsed >= Duration::from_millis(1500)
            || (selected && elapsed < Duration::from_millis(500))
        {
            self.contacts.respawn_player(&mut self.player);
            self.lost_life = None;
            self.restart_count = self.restart_count.wrapping_add(1);
        } else {
            self.lost_life = Some(elapsed);
        }
    }
    pub fn nearest_opponent(&self) -> &Opponent {
        let origin = self.player.position();
        self.opponents
            .iter()
            .min_by_key(|ai| {
                let d = ai
                    .position()
                    .into_iter()
                    .zip(origin)
                    .map(|(a, b)| (i64::from(a) - i64::from(b)).pow(2))
                    .sum::<i64>();
                (ai.health().0 <= 0, d)
            })
            .unwrap()
    }
    pub fn new(
        mut player: PlayerVehicle,
        contacts: CollisionScene,
        opponent_kinds: &[Vehicle],
    ) -> Self {
        let opponents = Opponent::spawn_bout(&mut player, opponent_kinds);
        Self {
            opponents,
            contacts,
            deck: Default::default(),
            player,
            sounds: Vec::new(),
            remainder: Duration::ZERO,
            paused: false,
            restart_count: 0,
            was_focused: false,
            victory_fields: 0,
            boss_intro: None,
            minion_at_start: false,
            lives: 3,
            death_fields: 0,
            lost_life: None,
        }
    }
}
pub fn advance(
    mut drive: ResMut<Drive>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut window: Single<&mut Window>,
    mut capture: ResMut<crate::smoke::Capture>,
    mut timings: Local<Vec<f64>>,
) {
    drive.sounds.clear();
    const STEP: Duration = Duration::from_millis(50);
    let pad = gamepads
        .iter()
        .min_by_key(|(id, _)| *id)
        .map(|(_, pad)| pad);
    let pressed = |button| pad.is_some_and(|p| p.pressed(button));
    let just_pressed = |button| pad.is_some_and(|p| p.just_pressed(button));
    let regained = window.focused && !drive.was_focused;
    drive.was_focused = window.focused;
    if window.focused && (keys.just_pressed(KeyCode::KeyR) || just_pressed(GamepadButton::Select)) {
        drive.contacts.reset_projectiles();
        drive.paused = false;
        drive.player.reset();
        drive.victory_fields = 0;
        drive.boss_intro = drive.minion_at_start.then(|| Duration::from_secs(5));
        drive.death_fields = 0;
        drive.lost_life = None;
        if drive.lives == 0 {
            drive.lives = 3;
        }
        let kinds: Vec<_> = drive
            .opponents
            .iter()
            .map(Opponent::vehicle)
            .filter(|&kind| kind != Vehicle::Minion)
            .collect();
        drive.opponents = Opponent::spawn_bout(&mut drive.player, &kinds);
        drive.restart_count = drive.restart_count.wrapping_add(1);
        drive.remainder = Duration::ZERO;
        return;
    }
    if window.focused && (keys.just_pressed(KeyCode::KeyP) || just_pressed(GamepadButton::Start)) {
        drive.paused = !drive.paused;
        drive.remainder = Duration::ZERO;
        return;
    }
    let status = if drive.player.health().0 <= 0 {
        "Wrecked • R to restart"
    } else if drive.cleared() {
        "Arena cleared • R to restart"
    } else if drive.paused {
        "Paused • P to resume"
    } else {
        drive.player.level().name()
    };
    let title = format!(
        "TM1 • {status}   |   {} {}   {} {}   |   Fire missiles {}",
        drive.player.vehicle().name(),
        drive.player.health().0.max(0),
        drive.nearest_opponent().vehicle().name(),
        drive.nearest_opponent().health().0.max(0),
        drive.player.missiles()
    );
    if window.title != title {
        window.title = title;
    }
    if drive.lives == 0 || drive.stage_finished() {
        drive.remainder = Duration::ZERO;
        return;
    }
    if drive.paused || (!capture.demo && (!window.focused || regained)) {
        drive.remainder = Duration::ZERO;
        return;
    }
    if drive.showing_boss_intro() || drive.lost_life.is_some() {
        let elapsed = if capture.demo && !capture.realtime {
            STEP
        } else {
            time.delta()
        };
        let selected = keys.just_pressed(KeyCode::Enter) || just_pressed(GamepadButton::South);
        if drive.showing_boss_intro() {
            drive.tick_boss_intro(elapsed, selected);
        } else {
            drive.tick_lost_life(elapsed, selected);
        }
        drive.remainder = Duration::ZERO;
        return;
    }
    let input = Input {
        cycle_weapon: keys.pressed(KeyCode::KeyE) || pressed(GamepadButton::LeftTrigger2),
        missile: keys.pressed(KeyCode::KeyF) || pressed(GamepadButton::LeftTrigger),
        fire_guns: keys.pressed(KeyCode::Space) || pressed(GamepadButton::RightTrigger),
        accelerate: keys.pressed(KeyCode::KeyW) || pressed(GamepadButton::West),
        reverse: keys.pressed(KeyCode::KeyS) || pressed(GamepadButton::East),
        left: keys.pressed(KeyCode::KeyA) || pressed(GamepadButton::DPadLeft),
        right: keys.pressed(KeyCode::KeyD) || pressed(GamepadButton::DPadRight),
        hard_turn: keys.pressed(KeyCode::ShiftLeft) || pressed(GamepadButton::South),
        turbo: keys.pressed(KeyCode::KeyT) || pressed(GamepadButton::North),
    };
    if capture.realtime {
        capture.elapsed += time.delta();
    }
    drive.remainder += if capture.demo && !capture.realtime {
        STEP
    } else {
        time.delta()
    };
    while drive.remainder >= STEP {
        if capture.realtime {
            capture.steps += 1;
        }
        let started = capture.demo.then(std::time::Instant::now);
        let state = &mut *drive;
        let input = if capture.special_demo {
            Input {
                missile: true,
                ..Default::default()
            }
        } else if capture.demo {
            Input {
                accelerate: true,
                fire_guns: true,
                cycle_weapon: state.player.gun_heat() == 0,
                missile: true,
                ..Default::default()
            }
        } else {
            input
        };
        state
            .contacts
            .step_bout(&mut state.player, &mut state.opponents, input, 3);
        if let Some(start) = started {
            crate::smoke::sample("Simulation step", &mut timings, start);
        }
        let mut effects = [[0; 5]; 122];
        let count = state.contacts.effects(&mut effects);
        for &[x, y, z, kind, life] in &effects[..count] {
            let id = match (kind, life) {
                (26, 2) => state.recharge_sound(),
                (0, 2) => 1,
                (11, 2) => 6,
                (10, 2) => 5,
                (kind @ 18..=25, 2) => (kind - 6) as usize,
                (1, 2) => 2,
                (2, 7) | (28, 32) | (29 | 30, 16) => 3,
                (12, 2) => 4,
                (kind @ 13..=17, 2) => (kind - 6) as usize,
                _ => continue,
            };
            state.sounds.push((id, [x, y, z]));
        }
        if state.player.health().0 > 0 && state.opponents.iter().all(|car| car.health().0 <= 0) {
            state.advance_victory_delay(3);
        } else if state.player.health().0 <= 0 {
            state.advance_death(3);
        }
        drive.remainder -= STEP;
        if drive.lives == 0
            || drive.stage_finished()
            || drive.showing_boss_intro()
            || drive.lost_life.is_some()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tick(app: &mut App, milliseconds: u64) {
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(milliseconds));
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_replaces_stage_entities_and_audio_in_one_world() {
        use crate::campaign::{Campaign, Stage, StageEntity};
        use bevy::ecs::system::RunSystemOnce;
        use tm1_core::drive::Level;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let stage = Stage::load(
            &disc,
            Level::Arena,
            Vehicle::YellowJacket,
            crate::campaign::Entry::New(Default::default()),
        )
        .unwrap();
        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<crate::additive::Additive>>();
        world.init_resource::<Assets<AudioSource>>();
        world.insert_resource(Campaign(disc));
        stage.install(&mut world);
        world.run_system_once(crate::render::setup).unwrap();
        world.run_system_once(crate::music::start).unwrap();
        world.run_system_once(crate::sfx::start).unwrap();
        for (level, count) in [
            (Level::Warehouse, 3),
            (Level::Freeway, 5),
            (Level::CityPark, 6),
            (Level::Cyburbia, 8),
            (Level::Rooftops, 3),
        ] {
            let old = world
                .query_filtered::<Entity, With<StageEntity>>()
                .iter(&world)
                .collect::<Vec<_>>();
            let mut drive = world.resource_mut::<Drive>();
            // Inject the shell victory event at the end of its return delay.
            drive.lives = 2;
            drive.opponents.clear();
            drive.advance_victory_delay(181);
            crate::campaign::advance(&mut world);
            let drive = world.resource::<Drive>();
            assert_eq!(drive.player.level(), level);
            assert_eq!(drive.lives, 2);
            assert_eq!(drive.opponents.len(), count);
            assert!(old.iter().all(|&entity| world.get_entity(entity).is_err()));
            assert_eq!(
                world
                    .query_filtered::<Entity, With<crate::music::ArenaMusic>>()
                    .iter(&world)
                    .count(),
                1
            );
            assert_eq!(
                world
                    .query_filtered::<Entity, With<crate::sfx::Engine>>()
                    .iter(&world)
                    .count(),
                1
            );
            assert_eq!(
                world
                    .query_filtered::<Entity, With<crate::render::HudLabel>>()
                    .iter(&world)
                    .count(),
                1
            );
        }
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn group_restarts_and_replays() {
        use tm1_core::drive::Level;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let vehicle = Vehicle::YellowJacket;
        for level in [
            Level::Warehouse,
            Level::Freeway,
            Level::CityPark,
            Level::Cyburbia,
            Level::Rooftops,
        ] {
            let kinds =
                level.opponents(vehicle, &mut tm1_core::drive::ArenaDeck::default(), &mut 1);
            let data = disc
                .read(&format!("UADMD/{}.DMD", level.database(vehicle)))
                .unwrap();
            let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
            let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;

            let mut models = kinds.clone();
            if level == Level::Rooftops {
                models.push(Vehicle::Minion);
            }
            let scene = CollisionScene::for_opponents(
                &data,
                root(1),
                root(vehicle as u16),
                &models.iter().map(|&v| root(v as u16)).collect::<Vec<_>>(),
            )
            .unwrap();
            let player = PlayerVehicle::on_level(&exe, vehicle, level).unwrap();
            let mut app = App::new();
            app.insert_resource(Drive::new(player, scene, &kinds))
                .insert_resource(crate::smoke::Capture::new(None, false, false))
                .init_resource::<ButtonInput<KeyCode>>()
                .init_resource::<Time<Real>>()
                .add_systems(Update, advance);
            app.world_mut().spawn(Window {
                focused: true,
                ..default()
            });
            tick(&mut app, 1);
            let snapshot = |app: &App| {
                let drive = app.world().resource::<Drive>();
                (
                    drive.player.frame(),
                    drive.player.rng_state(),
                    drive.player.health(),
                    drive
                        .opponents
                        .iter()
                        .map(|ai| (ai.vehicle(), ai.position(), ai.health()))
                        .collect::<Vec<_>>(),
                )
            };
            {
                let mut drive = app.world_mut().resource_mut::<Drive>();
                let rivals: Vec<_> = drive
                    .opponents
                    .iter()
                    .map(|r| (r.health(), r.position()))
                    .collect();
                drive.advance_death(180);
                assert_eq!(drive.lives, 3);
                drive.advance_death(3);
                assert_eq!(drive.lives, 2);
                drive.tick_lost_life(Duration::from_millis(1499), false);
                assert!(drive.lost_life.is_some());
                drive.tick_lost_life(Duration::from_millis(1), false);
                assert!(drive.lost_life.is_none());
                assert_eq!(
                    rivals,
                    drive
                        .opponents
                        .iter()
                        .map(|r| (r.health(), r.position()))
                        .collect::<Vec<_>>()
                );
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            tick(&mut app, 50);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::KeyR);
            let start = snapshot(&app);
            let run = |app: &mut App| {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::KeyW);
                for _ in 0..180 {
                    tick(app, 50);
                }
            };
            run(&mut app);
            let played = snapshot(&app);
            assert_eq!(played.0, 180);
            assert_eq!(played.3.len(), kinds.len());
            for (before, after) in start.3.iter().zip(&played.3) {
                assert_ne!(before.1, after.1);
            }
            if level == Level::Rooftops {
                let mut drive = app.world_mut().resource_mut::<Drive>();
                let sound = drive.recharge_sound();
                for _ in 0..60 {
                    drive.advance_victory_delay(3);
                }
                assert_eq!(drive.opponents.len(), 3);
                drive.advance_victory_delay(3);
                assert!(drive.showing_boss_intro());
                drive.tick_boss_intro(Duration::from_millis(4999), false);
                assert_eq!(drive.opponents.len(), 3);
                drive.tick_boss_intro(Duration::from_millis(1), false);
                assert_eq!(drive.opponents[3].vehicle(), Vehicle::Minion);
                assert_eq!(drive.opponents[3].health(), (250, 250));
                assert_eq!(sound, drive.recharge_sound());
                tick(&mut app, 50);
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            tick(&mut app, 50);
            assert_eq!(start, snapshot(&app));
            run(&mut app);
            assert_eq!(played, snapshot(&app));
        }
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn presentation_clock_pause_focus_and_controller_reconnect() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
        let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
        let build = || {
            let mut app = App::new();
            let scene = CollisionScene::from_dmd(&data, root(1), root(20), root(50)).unwrap();
            app.insert_resource(Drive::new(
                PlayerVehicle::from_executable(&exe).unwrap(),
                scene,
                &[Vehicle::Outlaw],
            ))
            .insert_resource(crate::smoke::Capture::new(None, false, false))
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Time<Real>>()
            .add_systems(Update, advance);
            let window = app
                .world_mut()
                .spawn(Window {
                    focused: true,
                    ..default()
                })
                .id();
            tick(&mut app, 1);
            (app, window)
        };
        let snapshot = |app: &App| {
            let d = app.world().resource::<Drive>();
            (
                d.player.state().to_vec(),
                d.opponents[0].position(),
                d.opponents[0].health(),
                d.player.rng_state(),
                d.contacts.hidden_pickups(),
            )
        };
        let (mut app, window) = build();
        let start = snapshot(&app);
        let mut pad = Gamepad::default();
        pad.digital_mut().press(GamepadButton::West);
        pad.digital_mut().press(GamepadButton::RightTrigger);
        let id = app.world_mut().spawn(pad).id();
        for _ in 0..200 {
            tick(&mut app, 10);
        }
        let driven = snapshot(&app);
        assert_ne!(driven, start);
        let (mut key_app, _) = build();
        for key in [KeyCode::KeyW, KeyCode::Space] {
            key_app
                .world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
        }
        for _ in 0..40 {
            tick(&mut key_app, 50);
        }
        assert_eq!(driven, snapshot(&key_app));
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        tick(&mut app, 15000);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        tick(&mut app, 15000);
        assert_eq!(driven, snapshot(&app));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyP);
        tick(&mut app, 50);
        tick(&mut app, 20000);
        assert_eq!(driven, snapshot(&app));
        app.world_mut().despawn(id);
        let mut replacement = Gamepad::default();
        replacement.digital_mut().press(GamepadButton::Select);
        let id = app.world_mut().spawn(replacement).id();
        tick(&mut app, 50);
        assert_eq!(start, snapshot(&app));
        let mut pad = app.world_mut().get_mut::<Gamepad>(id).unwrap();
        pad.digital_mut().reset_all();
        pad.digital_mut().press(GamepadButton::West);
        pad.digital_mut().press(GamepadButton::RightTrigger);
        for _ in 0..80 {
            tick(&mut app, 25);
        }
        assert_eq!(driven, snapshot(&app));
        app.world_mut().despawn(id);
        for step in 1..=24000 {
            tick(&mut app, 50);
            if app.world().resource::<Drive>().player.health().0 <= 0 {
                println!("Police defeated the taxi at idle step {step}");
                break;
            }
        }
        assert!(app.world().resource::<Drive>().player.health().0 <= 0);
        for _ in 0..90 {
            tick(&mut app, 50);
        }
        {
            let drive = app.world().resource::<Drive>();
            assert_eq!(drive.lives, 2);
            assert_eq!(drive.player.health().0, drive.player.health().1);
            assert_eq!(drive.player.missiles(), 2);
        }
        for _ in 0..48000 {
            tick(&mut app, 50);
            if app.world().resource::<Drive>().lives == 0 {
                break;
            }
        }
        assert_eq!(app.world().resource::<Drive>().lives, 0);
        let wrecked = snapshot(&app);
        tick(&mut app, 5000);
        assert_eq!(snapshot(&app), wrecked);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        tick(&mut app, 50);
        assert_eq!(snapshot(&app), start);
    }
}
