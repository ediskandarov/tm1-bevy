use bevy::prelude::*;
use std::time::Duration;
use tm1_core::drive::{CollisionScene, Input, PoliceCar, Taxi};

#[derive(Resource)]
pub struct Drive {
    pub taxi: Taxi,
    pub opponent: PoliceCar,
    pub contacts: CollisionScene,
    pub sounds: Vec<(usize, [i32; 3])>,
    remainder: Duration,
    pub paused: bool,
    pub restart_count: u32,
    was_focused: bool,
}
impl Drive {
    pub fn new(mut taxi: Taxi, contacts: CollisionScene) -> Self {
        let opponent = PoliceCar::spawn(&mut taxi);
        Self {
            opponent,
            contacts,
            taxi,
            sounds: Vec::new(),
            remainder: Duration::ZERO,
            paused: false,
            restart_count: 0,
            was_focused: false,
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
        drive.taxi.reset();
        drive.opponent = PoliceCar::spawn(&mut drive.taxi);
        drive.restart_count = drive.restart_count.wrapping_add(1);
        drive.remainder = Duration::ZERO;
        return;
    }
    if window.focused && (keys.just_pressed(KeyCode::KeyP) || just_pressed(GamepadButton::Start)) {
        drive.paused = !drive.paused;
        drive.remainder = Duration::ZERO;
        return;
    }
    let status = if drive.taxi.health().0 <= 0 {
        "Wrecked • R to restart"
    } else if drive.opponent.health().0 <= 0 {
        "Arena cleared • R to restart"
    } else if drive.paused {
        "Paused • P to resume"
    } else {
        "Arena"
    };
    let title = format!(
        "TM1 • {status}   |   Taxi {}   Police {}   |   Fire missiles {}",
        drive.taxi.health().0.max(0),
        drive.opponent.health().0.max(0),
        drive.taxi.missiles()
    );
    if window.title != title {
        window.title = title;
    }
    if drive.taxi.health().0 <= 0 || drive.opponent.health().0 <= 0 {
        drive.remainder = Duration::ZERO;
        return;
    }
    if drive.paused || (!capture.demo && (!window.focused || regained)) {
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
        let input = if capture.demo {
            Input {
                accelerate: true,
                fire_guns: true,
                cycle_weapon: state.taxi.gun_heat() == 0,
                missile: true,
                ..Default::default()
            }
        } else {
            input
        };
        state
            .contacts
            .step(&mut state.taxi, &mut state.opponent, input, 3);
        if let Some(start) = started {
            crate::smoke::sample("Simulation step", &mut timings, start);
        }
        let mut effects = [[0; 5]; 122];
        let count = state.contacts.effects(&mut effects);
        for &[x, y, z, kind, life] in &effects[..count] {
            let id = match (kind, life) {
                (0, 2) => 1,
                (1, 2) => 2,
                (2, 7) => 3,
                (4, 2) => 4,
                _ => continue,
            };
            state.sounds.push((id, [x, y, z]));
        }
        drive.remainder -= STEP;
        if drive.taxi.health().0 <= 0 || drive.opponent.health().0 <= 0 {
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
    fn presentation_clock_pause_focus_and_controller_reconnect() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
        let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
        let build = || {
            let mut app = App::new();
            let scene = CollisionScene::from_dmd(&data, root(1), root(20), root(50)).unwrap();
            app.insert_resource(Drive::new(Taxi::from_executable(&exe).unwrap(), scene))
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
                d.taxi.state().to_vec(),
                d.opponent.position(),
                d.opponent.health(),
                d.taxi.rng_state(),
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
            if app.world().resource::<Drive>().taxi.health().0 <= 0 {
                println!("Police defeated the taxi at idle step {step}");
                break;
            }
        }
        assert!(app.world().resource::<Drive>().taxi.health().0 <= 0);
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
