use bevy::prelude::*;
use std::time::Duration;
use tm1_core::drive::{
    CollisionScene, Input, Opponent, PlayerVehicle, PlayerView, RooftopPassage, Vehicle,
};

#[derive(Clone, Copy, PartialEq)]
pub enum ExclusiveVoice {
    Recharge,
    Hover,
    Pickup(i32),
    Contact(tm1_core::drive::ContactVoice),
    ScaledEffect(u8),
}

pub struct BattleSound {
    pub volume_percent: i32,
    pub exclusive: Option<ExclusiveVoice>,
    pub id: usize,
    pub position: [i32; 3],
    pub full_range: bool,
}

#[derive(Resource)]
pub struct Drive {
    pub player: PlayerVehicle,
    pub mirror: bool,
    pub windshield_holes: [[i32; 2]; 6],
    pub camera: tm1_core::drive::VehicleCamera,
    pub helicopter_unlocked: bool,
    helicopter_active: bool,
    rooftop_passage: RooftopPassage,
    helicopter: Option<tm1_core::drive::Helicopter>,
    pub deck: tm1_core::drive::ArenaDeck,
    pub opponents: Vec<Opponent>,
    pub contacts: CollisionScene,
    pub sounds: Vec<BattleSound>,
    remainder: Duration,
    pub pause: Option<crate::pause::Page>,
    pub settings: crate::pause::Settings,
    pub pause_sound: Option<bool>,
    pub abort_delay: Duration,
    options_idle: Duration,
    pub restart_count: u32,
    was_focused: bool,
    victory_fields: u16,
    boss_intro: Option<Duration>,
    boss_input_released: bool,
    minion_at_start: bool,
    pub lives: u8,
    death_fields: u16,
    lost_life: Option<Duration>,
}
impl Drive {
    pub fn helicopter_pose(&self) -> Option<[i32; 5]> {
        self.helicopter
            .as_ref()
            .filter(|_| self.helicopter_active)
            .map(|heli| heli.pose)
    }
    fn helicopter_available(&self) -> bool {
        use tm1_core::drive::Level;
        self.helicopter_unlocked
            && matches!(
                self.player.level(),
                Level::Arena | Level::Rooftops | Level::Arena5
            )
    }
    fn selected_view(&self) -> PlayerView {
        if self.helicopter_active {
            PlayerView::Helicopter
        } else {
            PlayerView::Car(self.camera.mode)
        }
    }
    fn update_rooftop_passage(&mut self, before: PlayerView) {
        if self.player.level() != tm1_core::drive::Level::Rooftops
            || !self.player.accepts_view_input()
        {
            return;
        }
        let selected = self.selected_view();
        let Some(view) = self.rooftop_passage.update(
            self.player.ground_group(),
            before,
            selected,
            self.camera.fixed_pose.is_some() && !self.helicopter_active,
        ) else {
            return;
        };
        match view {
            PlayerView::Helicopter if !self.helicopter_active => self.toggle_helicopter(),
            PlayerView::Car(mode) => {
                self.camera.mode = mode;
                if self.helicopter_active {
                    self.toggle_helicopter();
                }
            }
            _ => {}
        }
    }
    fn cycle_camera(&mut self, forward: bool) {
        use tm1_core::drive::CarView;
        if self.helicopter_active {
            self.helicopter_active = false;
            self.camera.mode = if forward {
                CarView::Cockpit
            } else {
                CarView::Far
            };
            self.camera.update(&self.player, true);
        } else if self.helicopter_available()
            && (forward && self.camera.mode == CarView::Far
                || !forward && self.camera.mode == CarView::Cockpit)
        {
            self.toggle_helicopter();
        } else {
            self.camera.cycle(forward);
        }
    }
    pub(crate) fn toggle_helicopter(&mut self) {
        if !self.helicopter_available() {
            return;
        }
        self.helicopter_active = !self.helicopter_active;
        if self.helicopter_active {
            self.mirror = false;
            let heli = self
                .helicopter
                .get_or_insert_with(|| tm1_core::drive::Helicopter::new(&mut self.player));
            heli.update(&self.player, 3, true);
        } else {
            self.camera.update(&self.player, true);
        }
    }
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
            self.boss_input_released = false;
            self.boss_intro = Some(Duration::from_secs(5));
            self.victory_fields = 0;
        }
    }
    pub fn start_with_minion(&mut self) {
        self.minion_at_start = true;
        self.boss_input_released = false;
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
        let skipped = selected && self.lost_life.unwrap() < Duration::from_millis(500);
        if elapsed >= Duration::from_millis(1500) || skipped {
            self.pause_sound = skipped.then_some(true);
            self.contacts.respawn_player(&mut self.player);
            if self.helicopter_active
                && let Some(heli) = &mut self.helicopter
            {
                heli.update(&self.player, 3, true);
            }
            self.camera.update(&self.player, true);
            self.lost_life = None;
            self.restart_count = self.restart_count.wrapping_add(1);
        } else {
            self.lost_life = Some(elapsed);
        }
    }
    pub fn new(
        mut player: PlayerVehicle,
        contacts: CollisionScene,
        opponent_kinds: &[Vehicle],
    ) -> Self {
        let opponents = Opponent::spawn_bout(&mut player, opponent_kinds);
        Self {
            windshield_holes: player.windshield_holes(),
            opponents,
            contacts,
            deck: Default::default(),
            camera: tm1_core::drive::VehicleCamera::new(&player, tm1_core::drive::CarView::Chase),
            player,
            sounds: Vec::new(),
            remainder: Duration::ZERO,
            pause: None,
            pause_sound: None,
            settings: Default::default(),
            abort_delay: Duration::ZERO,
            options_idle: Duration::ZERO,
            mirror: false,
            helicopter_unlocked: false,
            helicopter_active: false,
            rooftop_passage: Default::default(),
            helicopter: None,
            restart_count: 0,
            was_focused: false,
            victory_fields: 0,
            boss_intro: None,
            boss_input_released: false,
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
    drive.pause_sound = None;
    const STEP: Duration = Duration::from_millis(50);
    let pad = gamepads
        .iter()
        .min_by_key(|(id, _)| *id)
        .map(|(_, pad)| pad);
    let pressed = |button| pad.is_some_and(|p| p.pressed(button));
    let just_pressed = |button| pad.is_some_and(|p| p.just_pressed(button));
    let regained = window.focused && !drive.was_focused;
    drive.was_focused = window.focused;
    if drive.lives == 0 {
        return;
    }
    if window.focused
        && drive.pause != Some(crate::pause::Page::Aborted)
        && (keys.just_pressed(KeyCode::KeyR) || just_pressed(GamepadButton::Select))
    {
        drive.contacts.reset_projectiles();
        drive.pause = None;
        let state = &mut *drive;
        state.player.reset();
        state.player.set_pad_layout(state.settings.layout);
        state.camera.update(&state.player, true);
        drive.rooftop_passage = Default::default();
        drive.helicopter = None;
        drive.helicopter_active = false;
        drive.victory_fields = 0;
        drive.boss_input_released = false;
        drive.boss_intro = drive.minion_at_start.then(|| Duration::from_secs(5));
        drive.death_fields = 0;
        drive.lost_life = None;
        let kinds: Vec<_> = drive
            .opponents
            .iter()
            .map(Opponent::vehicle)
            .filter(|&kind| kind != Vehicle::Minion)
            .collect();
        drive.opponents = Opponent::spawn_bout(&mut drive.player, &kinds);
        drive.windshield_holes = drive.player.windshield_holes();
        drive.restart_count = drive.restart_count.wrapping_add(1);
        drive.remainder = Duration::ZERO;
        return;
    }
    use crate::pause::{Action, Page};
    let start = keys.just_pressed(KeyCode::KeyP) || just_pressed(GamepadButton::Start);
    if let Some(page) = drive.pause {
        if window.focused {
            if page == Page::Aborted {
                drive.abort_delay = drive.abort_delay.saturating_sub(time.delta());
            } else {
                let action = if start {
                    Some(Action::Resume)
                } else {
                    [
                        (KeyCode::ArrowUp, GamepadButton::DPadUp, Action::Up),
                        (KeyCode::ArrowDown, GamepadButton::DPadDown, Action::Down),
                        (KeyCode::ArrowLeft, GamepadButton::DPadLeft, Action::Left),
                        (KeyCode::ArrowRight, GamepadButton::DPadRight, Action::Right),
                        (KeyCode::Enter, GamepadButton::South, Action::Select),
                    ]
                    .into_iter()
                    .find(|(key, pad, _)| keys.just_pressed(*key) || just_pressed(*pad))
                    .map(|(_, _, a)| a)
                };
                let action = if action.is_none() && matches!(page, Page::Audio(_) | Page::Controls)
                {
                    drive.options_idle += time.delta();
                    (drive.options_idle >= Duration::from_secs_f64(18001. / 60.))
                        .then_some(Action::Resume)
                } else {
                    action
                };
                if let Some(action) = action {
                    drive.options_idle = Duration::ZERO;
                    drive.pause_sound = match action {
                        Action::Select | Action::Resume => Some(true),
                        Action::Up | Action::Down if !matches!(page, Page::Controls) => Some(false),
                        Action::Left | Action::Right
                            if matches!(page, Page::Audio(0 | 1) | Page::Controls) =>
                        {
                            Some(false)
                        }
                        _ => None,
                    };
                    drive.pause = crate::pause::navigate(page, action, &mut drive.settings);
                    if page == Page::Controls && drive.pause.is_none() {
                        let layout = drive.settings.layout;
                        drive.player.set_pad_layout(layout);
                    }
                    if drive.pause == Some(Page::Aborted) {
                        drive.abort_delay = Duration::from_millis(500);
                    }
                }
            }
        }
        drive.remainder = Duration::ZERO;
        return;
    }
    if window.focused && start && !drive.showing_boss_intro() && drive.lost_life.is_none() {
        drive.pause = Some(Page::Choices(0));
        drive.remainder = Duration::ZERO;
        return;
    }
    let status = if drive.player.health().0 <= 0 {
        "Wrecked • R to restart"
    } else if drive.cleared() {
        "Arena cleared • R to restart"
    } else {
        drive.player.level().name()
    };
    let target = drive
        .player
        .hud_opponent(&drive.opponents)
        .map(|ai| format!("   {} {}", ai.vehicle().name(), ai.health().0.max(0)))
        .unwrap_or_default();
    let title = format!(
        "TM1 • {status}   |   {} {}{target}   |   Fire missiles {}",
        drive.player.vehicle().name(),
        drive.player.health().0.max(0),
        drive.player.missiles()
    );
    if window.title != title {
        window.title = title;
    }
    if drive.stage_finished() {
        drive.remainder = Duration::ZERO;
        return;
    }
    if !capture.demo && (!window.focused || regained) {
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
            if !drive.boss_input_released {
                drive.boss_input_released = keys.get_pressed().next().is_none()
                    && pad.is_none_or(|p| p.get_pressed().next().is_none());
            }
            if drive.boss_input_released {
                drive.pause_sound = selected.then_some(true);
                drive.tick_boss_intro(elapsed, selected);
            }
        } else {
            drive.tick_lost_life(elapsed, selected);
        }
        drive.remainder = Duration::ZERO;
        return;
    }
    let previous_view = drive.selected_view();
    if drive.player.accepts_view_input() {
        if keys.just_pressed(KeyCode::KeyH) {
            drive.toggle_helicopter();
        }
        if keys.just_pressed(KeyCode::KeyB) && !drive.helicopter_active {
            drive.mirror = !drive.mirror;
        }
        if keys.just_pressed(KeyCode::KeyC) {
            drive.cycle_camera(true);
        } else if keys.just_pressed(KeyCode::KeyZ) {
            drive.cycle_camera(false);
        }
    }
    if previous_view != drive.selected_view() {
        drive.update_rooftop_passage(previous_view);
    }
    let input = Input {
        cycle_weapon: keys.pressed(KeyCode::KeyE),
        missile: keys.pressed(KeyCode::KeyF),
        fire_guns: keys.pressed(KeyCode::Space),
        accelerate: keys.pressed(KeyCode::KeyW),
        reverse: keys.pressed(KeyCode::KeyS),
        left: keys.pressed(KeyCode::KeyA),
        right: keys.pressed(KeyCode::KeyD),
        hard_turn: keys.pressed(KeyCode::ShiftLeft),
        turbo: keys.pressed(KeyCode::KeyT),
    };
    // Keep physical pad bits intact so uaPadInit can reinterpret them.
    let pad_bits = [
        GamepadButton::LeftTrigger,
        GamepadButton::RightTrigger,
        GamepadButton::LeftTrigger2,
        GamepadButton::RightTrigger2,
        GamepadButton::North,
        GamepadButton::East,
        GamepadButton::South,
        GamepadButton::West,
        GamepadButton::Select,
        GamepadButton::LeftThumb,
        GamepadButton::RightThumb,
        GamepadButton::Start,
        GamepadButton::DPadUp,
        GamepadButton::DPadRight,
        GamepadButton::DPadDown,
        GamepadButton::DPadLeft,
    ]
    .into_iter()
    .enumerate()
    .fold(0u16, |bits, (i, button)| {
        bits | (u16::from(pressed(button)) << i)
    });
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
        let mut pads = u32::from(
            state.player.keyboard_buttons(input) | if capture.demo { 0 } else { pad_bits },
        );
        if state.player.cheat_controls() && !capture.demo {
            // Each digit holds one original multi-button combination for a simulation step.
            for (key, mask) in [
                (KeyCode::Digit1, 0x30c9),
                (KeyCode::Digit2, 0xc036),
                (KeyCode::Digit3, 0xc035),
                (KeyCode::Digit4, 0x30ca),
                (KeyCode::Digit5, 0x3000),
                (KeyCode::Digit6, 0x30c9_0000),
                (KeyCode::Digit7, 0xc036_0000),
            ] {
                if keys.pressed(key) {
                    pads = mask;
                    break;
                }
            }
        }
        state.update_rooftop_passage(state.selected_view());
        state
            .contacts
            .step_bout_with_pads(&mut state.player, &mut state.opponents, pads, 3);
        if state.helicopter_active
            && let Some(heli) = &mut state.helicopter
        {
            heli.update(&state.player, 3, false);
        } else {
            state.camera.update(&state.player, false);
        }
        if let Some(start) = started {
            crate::smoke::sample("Simulation step", &mut timings, start);
        }
        let first_contact_sample = state.recharge_sound() + 5;
        let cars = std::iter::once((state.player.position(), state.player.contact_sounds())).chain(
            state
                .opponents
                .iter()
                .map(|ai| (ai.position(), ai.contact_sounds())),
        );
        for (position, contacts) in cars {
            for (voice, volume_percent) in contacts.into_iter().flatten() {
                state.sounds.push(BattleSound {
                    volume_percent,
                    id: first_contact_sample + voice as usize,
                    exclusive: Some(ExclusiveVoice::Contact(voice)),
                    position,
                    full_range: false,
                });
            }
        }
        let mut effects = [[0; 5]; 122];
        let count = state.contacts.effects(&mut effects);
        for &[x, y, z, kind, life] in &effects[..count] {
            let id = match (kind, life) {
                (26, 2) => state.recharge_sound(),
                (36, 2) => state.recharge_sound() + 1,
                (0 | 51, 2) => 1,
                (52, 2) => 4,
                (401..=406 | 410..=414 | 430, 2) => state.recharge_sound() + 4,
                (11, 2) => 6,
                (10, 2) => 5,
                (kind @ 18..=25, 2) => (kind - 6) as usize,
                (54, 2) => 2,
                (1, 4) => state.recharge_sound() + 2,
                (55, 32) => state.recharge_sound() + 9,
                (28, 32) => state.recharge_sound() + 2,
                (33, 2) | (2, 7) | (37, 32) | (29 | 30, 16) => 3,
                (12, 2) => 4,
                (kind @ 13..=17, 2) => (kind - 6) as usize,
                _ => continue,
            };
            state.sounds.push(BattleSound {
                volume_percent: match kind {
                    1 => 84,
                    37 => 86,
                    54 => 25,
                    _ => 99,
                },
                exclusive: match kind {
                    1 | 28 => Some(ExclusiveVoice::ScaledEffect(103)),
                    54 => Some(ExclusiveVoice::ScaledEffect(4)),
                    55 => Some(ExclusiveVoice::ScaledEffect(104)),
                    37 => Some(ExclusiveVoice::ScaledEffect(105)),
                    26 => Some(ExclusiveVoice::Recharge),
                    401..=406 | 410..=414 | 430 => Some(ExclusiveVoice::Pickup(kind)),
                    _ => None,
                },
                id,
                position: if kind == 36 {
                    state.player.position()
                } else {
                    [x, y, z]
                },
                full_range: matches!(kind, 29 | 30 | 33 | 37),
            });
        }
        for slot in 0..2 {
            if let Some([_, x, y, z, 1]) = state.contacts.merc_pose(slot) {
                state.sounds.push(BattleSound {
                    volume_percent: 99,
                    exclusive: Some(ExclusiveVoice::Hover),
                    id: state.recharge_sound() + 3,
                    position: [x, y, z],
                    full_range: false,
                });
            }
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
    fn pause_options_from_keyboard() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::pause::Page;
        use tm1_core::drive::Level;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let stage = Stage::load(
            &disc,
            Level::Warehouse,
            Vehicle::YellowJacket,
            Entry::New(Default::default()),
        )
        .unwrap();
        let mut app = App::new();
        stage.install(app.world_mut());
        app.insert_resource(Campaign(disc))
            .insert_resource(crate::smoke::Capture::new(None, false, false))
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<AudioSource>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Time<Real>>()
            .add_systems(Update, (advance, crate::campaign::advance).chain());
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        tick(&mut app, 1);
        let tap = |app: &mut App, key| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            tick(app, 50);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(key);
        };
        let frame = app.world().resource::<Drive>().player.frame();
        use KeyCode::*;
        for key in [KeyP, ArrowDown, ArrowDown, Enter, ArrowRight] {
            tap(&mut app, key);
        }
        let drive = app.world().resource::<Drive>();
        assert_eq!(drive.pause, Some(Page::Audio(0)));
        assert_eq!((drive.settings.music, drive.settings.effects), (8, 6));
        assert!(drive.settings.music_gain() > 1. && drive.settings.effects_gain() < 1.);
        assert_eq!(drive.player.frame(), frame);
        for key in [ArrowUp, Enter, KeyP, ArrowUp, Enter, ArrowRight, Enter] {
            tap(&mut app, key);
        }
        let drive = app.world().resource::<Drive>();
        assert_eq!(drive.settings.layout, 2);
        assert_eq!(drive.player.state()[0xc8 + 22], 2);
        assert!(drive.pause.is_none());
        assert_eq!(drive.player.frame(), frame);
        for key in [KeyP, ArrowDown, Enter] {
            tap(&mut app, key);
        }
        assert_eq!(app.world().resource::<Drive>().pause, Some(Page::Aborted));
        tick(&mut app, 600);
        assert!(app.world().contains_resource::<crate::contest::Contest>());
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
            crate::transition::complete(&mut world);
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
                    .query_filtered::<Entity, With<crate::sfx::RivalEngine>>()
                    .iter(&world)
                    .count(),
                count + usize::from(level == Level::Rooftops)
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
        let vehicle = Vehicle::YellowJacket;
        for level in [
            Level::Warehouse,
            Level::Freeway,
            Level::CityPark,
            Level::Cyburbia,
            Level::Rooftops,
        ] {
            let stage = crate::campaign::Stage::load(
                &disc,
                level,
                vehicle,
                crate::campaign::Entry::New(Default::default()),
            )
            .unwrap();
            let opponents = stage.drive.opponents.len();
            let mut app = App::new();
            app.insert_resource(stage.drive)
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
                    (0..6)
                        .filter_map(|i| drive.contacts.pedestrian_pose(i))
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
            assert_eq!(played.3.len(), opponents);
            assert!(!start.4.is_empty());
            assert_ne!(start.4, played.4);
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
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyB);
        tick(&mut app, 0);
        assert!(app.world().resource::<Drive>().mirror);
        {
            let mut drive = app.world_mut().resource_mut::<Drive>();
            drive.helicopter_unlocked = true;
            drive.toggle_helicopter();
            assert!(!drive.mirror);
            assert!(drive.helicopter_pose().is_some());
            drive.toggle_helicopter();
        }
        use tm1_core::drive::CarView::{Chase, Cockpit, Far};
        for (key, mode, helicopter) in [
            (KeyCode::KeyZ, Cockpit, false),
            (KeyCode::KeyZ, Cockpit, true),
            (KeyCode::KeyZ, Far, false),
            (KeyCode::KeyC, Far, true),
            (KeyCode::KeyC, Cockpit, false),
            (KeyCode::KeyC, Chase, false),
        ] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            tick(&mut app, 0);
            let drive = app.world().resource::<Drive>();
            assert_eq!(
                (drive.camera.mode, drive.helicopter_active),
                (mode, helicopter)
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(key);
        }
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
        assert_eq!(snapshot(&app), wrecked);
    }
}
