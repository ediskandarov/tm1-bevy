use crate::render::CarPose;
use bevy::prelude::*;
use std::time::Duration;
use tm1_core::drive::{
    CollisionScene, HelicopterRoute, Input, Opponent, PlayerVehicle, PlayerView, RooftopPassage,
    Vehicle,
};

#[derive(Clone, Copy, PartialEq)]
pub enum ExclusiveVoice {
    MachineGuns(Vehicle),
    Turbo,
    Recharge,
    Scream,
    Hover,
    HoverGuns,
    WeaponLaunch(u8),
    SpecialLaunch(Vehicle),
    CarExplosion,
    Signature(Vehicle),
    Pickup(i32),
    Contact(tm1_core::drive::ContactVoice),
    ScaledEffect(u8),
}

#[derive(Clone, Copy, Default)]
pub struct EngineMix {
    pub rev: i32,
    pub channels: [i32; 2],
    pub range: u32,
}

pub enum BattleRequest {
    EngineRev(usize, EngineMix),
    EngineStop(usize),
    EngineDeferred(usize),
    Play(BattleSound),
    Stop(ExclusiveVoice),
    Turbo,
}

pub struct BattleSound {
    pub frame: u32,
    pub volume_percent: i32,
    pub exclusive: Option<ExclusiveVoice>,
    pub id: usize,
    pub channels: [i32; 2],
    pub range: u32,
}

pub(crate) fn effect_volume(kind: i32) -> i32 {
    match kind {
        1 => 84,
        37 => 86,
        29 => 95,
        54 => 25,
        _ => 99,
    }
}
pub(crate) fn effect_voice(kind: i32) -> Option<ExclusiveVoice> {
    match kind {
        1 | 28 => Some(ExclusiveVoice::ScaledEffect(103)),
        54 => Some(ExclusiveVoice::ScaledEffect(4)),
        55 | 56 => Some(ExclusiveVoice::ScaledEffect(104)),
        29 | 30 | 33 | 37 => Some(ExclusiveVoice::ScaledEffect(105)),
        11 => Some(ExclusiveVoice::WeaponLaunch(7)),
        26 => Some(ExclusiveVoice::Recharge),
        36 => Some(ExclusiveVoice::Scream),
        401..=406 | 410..=414 | 430 => Some(ExclusiveVoice::Pickup(kind)),
        _ => None,
    }
}
pub(crate) fn missile_sample(weapon: u8) -> usize {
    [4, 7, 8, 9, 10, 11][usize::from(weapon)]
}

pub(crate) fn hover_sample(
    sound: tm1_core::drive::HoverSound,
    motor: usize,
) -> (ExclusiveVoice, Option<usize>) {
    use tm1_core::drive::HoverSound;
    match sound {
        HoverSound::Guns(firing) => (ExclusiveVoice::HoverGuns, firing.then_some(1)),
        HoverSound::Launch => (ExclusiveVoice::WeaponLaunch(0), Some(missile_sample(0))),
        HoverSound::Motor => (ExclusiveVoice::Hover, Some(motor)),
    }
}

pub(crate) fn car_sample(
    sound: tm1_core::drive::CarSound,
    car: Vehicle,
    slot: usize,
    recharge: usize,
) -> (usize, ExclusiveVoice, i32) {
    use tm1_core::drive::CarSound;
    match sound {
        CarSound::Contact(voice, level) => (
            recharge + 5 + voice as usize,
            ExclusiveVoice::Contact(voice),
            level,
        ),
        CarSound::Explosion => (3, ExclusiveVoice::CarExplosion, 99),
        CarSound::Signature => {
            let id = if car == Vehicle::Minion {
                recharge + 10
            } else if slot == 0 {
                5
            } else {
                11 + slot
            };
            (id, ExclusiveVoice::Signature(car), 99)
        }
    }
}

pub(crate) fn effect_sample(kind: i32, recharge: usize) -> Option<usize> {
    Some(match kind {
        26 => recharge,
        36 => recharge + 1,
        401..=406 | 410..=414 | 430 => recharge + 4,
        11 => 6,
        10 => 5,
        18..=25 => (kind - 6) as usize,
        54 => 2,
        1 | 28 => recharge + 2,
        55 | 56 => recharge + 9,
        33 | 2 | 37 | 29 | 30 => 3,
        _ => return None,
    })
}

// soundPlayId and soundStopPlayId leave the request count at 45 once full.
// soundProcessIds walks only the counted entries, then resets the queue.
struct BattleQueue {
    remaining: usize,
}

impl BattleQueue {
    fn new() -> Self {
        Self { remaining: 45 }
    }

    fn submit(&mut self, request: BattleRequest, emit: &mut impl FnMut(BattleRequest)) {
        if matches!(request, BattleRequest::EngineStop(_)) {
            // uasoundStopCarSounds queues IDs 0, engine, 2, gun in that order.
            // The two legacy IDs have no active voice. The gun has its own event.
            let engine_fits = self.remaining >= 2;
            self.remaining = self.remaining.saturating_sub(3);
            if engine_fits {
                emit(request);
            }
        } else if self.remaining > 0 {
            self.remaining -= 1;
            // SOUND.c mode 4 replaces the active tones inside soundProcessIds.
            // A full queue discards the play request before it can stop the old voice.
            if let BattleRequest::Play(BattleSound {
                exclusive: Some(ExclusiveVoice::WeaponLaunch(7)),
                ..
            }) = &request
            {
                emit(BattleRequest::Stop(ExclusiveVoice::WeaponLaunch(7)));
            }
            emit(request);
        } else if let BattleRequest::EngineRev(slot, _) = request {
            emit(BattleRequest::EngineDeferred(slot));
        }
    }
}

// Deliver requests in simulation order, including the final hover motor request.
pub(crate) fn battle_audio(
    frame: u32,
    scene: &CollisionScene,
    listener: &tm1_core::drive::SoundListener,
    volume: u8,
    recharge: usize,
    previous_view: &tm1_core::drive::SoundListener,
    split_screen: bool,
    car: impl Fn(usize) -> Vehicle,
    mut emit: impl FnMut(BattleRequest),
) {
    use tm1_core::drive::{BattleEvent, CarSound};
    let mut queue = BattleQueue::new();
    let mut emit = |request| queue.submit(request, &mut emit);
    for (index, event) in scene.battle_events(listener).enumerate() {
        let listener = if index < scene.events_before_primary_view() {
            previous_view
        } else {
            listener
        };
        let centered = |id, exclusive, volume_percent| {
            BattleRequest::Play(BattleSound {
                frame,
                id,
                exclusive,
                volume_percent,
                channels: [i32::from(volume); 2],
                range: 0,
            })
        };
        let play = |id, exclusive, volume_percent, position, full_range| {
            BattleRequest::Play(BattleSound {
                frame,
                id,
                exclusive,
                volume_percent,
                channels: listener.channels(position, volume, full_range),
                range: if full_range {
                    0
                } else {
                    listener.sound_range(position)
                },
            })
        };

        match event {
            BattleEvent::MonsterSmash(slot) => {
                let id = if slot == 0 { 5 } else { 11 + slot };
                emit(centered(
                    id,
                    Some(ExclusiveVoice::SpecialLaunch(Vehicle::Hammerhead)),
                    99,
                ));
            }
            BattleEvent::Turbo(on) => emit(if on {
                BattleRequest::Turbo
            } else {
                BattleRequest::Stop(ExclusiveVoice::Turbo)
            }),
            BattleEvent::Guns {
                slot,
                firing,
                position,
            } => {
                let key = ExclusiveVoice::MachineGuns(car(slot));
                emit(if firing {
                    play(1, Some(key), 99, position, false)
                } else {
                    BattleRequest::Stop(key)
                });
            }
            BattleEvent::EngineStop(slot) => emit(BattleRequest::EngineStop(slot)),
            BattleEvent::Actor(slot) => {
                let effect = |(sound, position)| {
                    let (id, key, level) = car_sample(sound, car(slot), slot, recharge);
                    if split_screen {
                        centered(id, Some(key), level)
                    } else {
                        play(id, Some(key), level, position, sound == CarSound::Explosion)
                    }
                };
                let mut sounds = scene.car_sound_requests(slot).peekable();
                // UASetSoundFlags requests skid and wreck before rev, then crash and signature.
                while let Some(sound) = sounds.next_if(|(s, _)| {
                    matches!(
                        s,
                        CarSound::Contact(tm1_core::drive::ContactVoice::Skid, _)
                            | CarSound::Explosion
                    )
                }) {
                    emit(effect(sound));
                }
                let (rev, position) = scene.car_engine_request(slot);
                let (channels, range) = if split_screen {
                    ([i32::from(volume); 2], 0)
                } else {
                    (
                        listener.channels(position, volume, false),
                        listener.sound_range(position),
                    )
                };
                emit(BattleRequest::EngineRev(
                    slot,
                    EngineMix {
                        rev,
                        channels,
                        range,
                    },
                ));
                for sound in sounds {
                    emit(effect(sound));
                }
            }
            BattleEvent::Effect { kind, position } => {
                if let Some(id) = effect_sample(kind, recharge) {
                    if kind == 36 {
                        emit(centered(id, effect_voice(kind), 99));
                        continue;
                    }
                    let restart = match kind {
                        10 => Some(ExclusiveVoice::SpecialLaunch(car(0))),
                        18..=25 => Some(ExclusiveVoice::SpecialLaunch(car((kind - 17) as usize))),
                        _ => None,
                    };
                    if let Some(key) = restart {
                        emit(BattleRequest::Stop(key));
                    }
                    emit(play(
                        id,
                        restart.or_else(|| effect_voice(kind)),
                        effect_volume(kind),
                        position,
                        matches!(kind, 29 | 30 | 33 | 37),
                    ));
                }
            }
            BattleEvent::MissileSound {
                weapon,
                launch,
                position,
            } => {
                let id = missile_sample(weapon);
                let key = ExclusiveVoice::WeaponLaunch(weapon);
                if launch {
                    emit(BattleRequest::Stop(key));
                }
                emit(play(id, Some(key), 99, position, false));
            }
            BattleEvent::DropLaunch { weapon, position } => {
                let offset = match weapon {
                    6 => 11,
                    8 => 12,
                    9 => 13,
                    10 => 14,
                    _ => continue,
                };
                let key = ExclusiveVoice::WeaponLaunch(weapon);
                emit(BattleRequest::Stop(key));
                emit(play(recharge + offset, Some(key), 99, position, false));
            }
            BattleEvent::WeaponImpact { weapon, position } => {
                // Fire and rear missiles share impact ID 0x19 in UA_SOUND.c.
                let id = match weapon {
                    0 | 5 => 0x19,
                    1 => 0x1c,
                    2 => 0x25,
                    3 => 0x1f,
                    4 => 0x22,
                    6 => 0x13,
                    8 => 0x07,
                    9 => 0x0a,
                    10 => 0x0d,
                    _ => continue,
                };
                emit(BattleRequest::Stop(ExclusiveVoice::WeaponLaunch(weapon)));
                emit(play(
                    if weapon == 9 { recharge + 13 } else { 3 },
                    Some(ExclusiveVoice::ScaledEffect(id)),
                    99,
                    position,
                    false,
                ));
            }
            BattleEvent::Hover { sound, position } => {
                let (key, id) = hover_sample(sound, recharge + 3);
                emit(match id {
                    Some(id) => play(id, Some(key), 99, position, false),
                    None => BattleRequest::Stop(key),
                });
            }
        }
    }
}
use crate::pause::PauseInput;

/// Accumulate 60 Hz video fields. RT waits for three before presenting another
/// battle update. SetFrameStart passes at most five to the simulation.
#[derive(Default)]
pub(crate) struct BattleClock {
    field_nanos: u128,
}

impl BattleClock {
    pub fn advance(&mut self, elapsed: Duration) -> Option<u8> {
        self.field_nanos += elapsed.as_nanos() * 60;
        let fields = self.field_nanos / 1_000_000_000;
        if fields < 3 {
            return None;
        }
        self.field_nanos %= 1_000_000_000;
        Some(fields.min(5) as u8)
    }
}

#[derive(Resource)]
pub struct Drive {
    battle_seed: u32,
    level_seed: u32,
    reset_weapons_on_entry: bool,
    pub player: PlayerVehicle,
    pub mirror: bool,
    pub radar: bool,
    pub list_cars: bool,
    view_input_held: bool,
    helicopter_key_held: bool,
    pub windshield_holes: [[i32; 2]; 6],
    pub camera: tm1_core::drive::VehicleCamera,
    pub(crate) primary_cars: Vec<crate::render::CarPose>,
    pub(crate) primary_hidden: bool,
    pub listener: tm1_core::drive::SoundListener,
    last_listener: Option<tm1_core::drive::SoundListener>,
    pub helicopter_unlocked: bool,
    helicopter_active: bool,
    rooftop_passage: RooftopPassage,
    initial_helicopter_route: HelicopterRoute,
    helicopter: Option<tm1_core::drive::Helicopter>,
    pub deck: tm1_core::drive::ArenaDeck,
    pub opponents: Vec<Opponent>,
    pub contacts: CollisionScene,
    pub sounds: Vec<BattleRequest>,
    clock: BattleClock,
    pub pause: Option<crate::pause::Page>,
    pub settings: crate::pause::Settings,
    pub pause_sound: Option<bool>,
    pub(crate) abort_wait: crate::pause::AbortWait,
    options_idle: Duration,
    pause_input: PauseInput,
    pub(crate) control_arrows: [bool; 2],
    pub audio_restart_count: u32,
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
        self.helicopter_unlocked && self.helicopter.is_some()
    }
    pub(crate) fn helicopter_route(&self) -> HelicopterRoute {
        self.helicopter
            .as_ref()
            .map_or(self.initial_helicopter_route, |heli| heli.route())
    }
    fn stage_helicopter(
        player: &mut PlayerVehicle,
        route: HelicopterRoute,
    ) -> Option<tm1_core::drive::Helicopter> {
        use tm1_core::drive::Level;
        matches!(
            player.level(),
            Level::Arena | Level::Rooftops | Level::Arena5
        )
        .then(|| tm1_core::drive::Helicopter::from_route(player, route))
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
    fn check_view_keys(&mut self, keys: &ButtonInput<KeyCode>, pads: u32) {
        if !self.player.accepts_view_input() {
            return;
        }
        use KeyCode::*;
        if keys.pressed(KeyH) && !self.helicopter_key_held {
            self.toggle_helicopter();
        }
        self.helicopter_key_held = keys.pressed(KeyH);
        // CheckVRMode shares one latch across the five controls, in this order.
        let selected = [KeyC, KeyZ, KeyV, KeyL, KeyB]
            .into_iter()
            .zip(self.player.view_buttons(pads))
            .find_map(|(key, pad)| (keys.pressed(key) || pad).then_some(key));
        if !self.view_input_held {
            match selected {
                Some(KeyC) => self.cycle_camera(true, pads),
                Some(KeyZ) => self.cycle_camera(false, pads),
                Some(KeyV) => self.radar = !self.radar,
                Some(KeyL) => self.list_cars = !self.list_cars,
                Some(KeyB) if !self.helicopter_active => self.mirror = !self.mirror,
                _ => {}
            }
        }
        self.view_input_held = selected.is_some();
        // CheckVRMode polls ctlpadSpecial(1, 0) after cycling the view.
        self.helicopter_unlocked |= pads == 0x180b;
    }
    fn cycle_camera(&mut self, forward: bool, pads: u32) {
        use tm1_core::drive::CarView;
        let before = self.selected_view();
        let fixed = self.camera.fixed_pose.is_some() && !self.helicopter_active;
        let wraps = if forward {
            fixed
                || before == PlayerView::Helicopter
                || (before == PlayerView::Car(CarView::Far) && !self.helicopter_available())
        } else {
            !fixed && before == PlayerView::Car(CarView::Cockpit)
        };
        if wraps {
            self.player.wrap_special_view(forward, pads);
        }
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
            let heli = self.helicopter.as_mut().unwrap();
            heli.update(&self.player, self.player.last_fields(), true);
        } else {
            self.camera.update(&self.player, true);
        }
    }
    pub fn cleared(&self) -> bool {
        !self.awaiting_minion() && self.victory_fields > 0
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
            self.audio_restart_count = self.audio_restart_count.wrapping_add(1);
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
    fn tick_lost_life(&mut self, delta: Duration, selected: bool, ticks: impl FnOnce() -> u32) {
        let elapsed = self.lost_life.unwrap() + delta;
        let skipped = selected && self.lost_life.unwrap() < Duration::from_millis(500);
        if elapsed >= Duration::from_millis(1500) || skipped {
            self.pause_sound = skipped.then_some(true);
            if let Some(heli) = &mut self.helicopter {
                heli.respawn(&mut self.player);
            }
            self.last_listener = None;
            self.contacts.respawn_player(&mut self.player);
            if self.helicopter_active
                && let Some(heli) = &mut self.helicopter
            {
                heli.update(&self.player, self.player.last_fields(), true);
            }
            self.camera.update(&self.player, true);
            self.capture_primary_view();
            self.player.set_rng_state(ticks());
            self.windshield_holes = self.player.windshield_holes();
            self.lost_life = None;
            self.audio_restart_count = self.audio_restart_count.wrapping_add(1);
        } else {
            self.lost_life = Some(elapsed);
        }
    }
    fn capture_primary_view(&mut self) {
        self.primary_cars.clear();
        self.primary_cars.push(CarPose::player(&self.player));
        self.primary_cars
            .extend(self.opponents.iter().map(CarPose::rival));
        self.primary_hidden = self.helicopter_pose().is_none() && !self.camera.show_body;
        self.listener =
            tm1_core::drive::SoundListener::new(&self.player, &self.camera, self.helicopter_pose());
    }
    pub fn begin_battle(&mut self, ticks: u32) {
        self.player
            .set_battle_view(self.selected_view(), self.camera.fixed_pose.is_some());
        self.capture_primary_view();
        self.battle_seed = ticks;
        self.player.set_rng_state(ticks);
        self.windshield_holes = self.player.windshield_holes();
    }
    pub fn new(
        mut player: PlayerVehicle,
        contacts: CollisionScene,
        opponent_kinds: &[Vehicle],
        initial_helicopter_route: HelicopterRoute,
        ticks: impl FnOnce() -> u32,
        reset_weapons_on_entry: bool,
    ) -> Self {
        let opponents = Opponent::spawn_bout(&mut player, opponent_kinds);
        // uaInitDB samples the counter only after uaPickAICars reserves the rival starts.
        let level_seed = ticks();
        player.set_rng_state(level_seed);
        let helicopter = Self::stage_helicopter(&mut player, initial_helicopter_route);
        player.initialize_battle_car(reset_weapons_on_entry);
        let camera = tm1_core::drive::VehicleCamera::new(&player, tm1_core::drive::CarView::Chase);
        Self {
            battle_seed: player.rng_state(),
            level_seed,
            reset_weapons_on_entry,
            windshield_holes: player.windshield_holes(),
            primary_cars: std::iter::once(crate::render::CarPose::player(&player))
                .chain(opponents.iter().map(crate::render::CarPose::rival))
                .collect(),
            primary_hidden: !camera.show_body,
            opponents,
            contacts,
            deck: Default::default(),
            last_listener: None,
            listener: tm1_core::drive::SoundListener::new(&player, &camera, None),
            camera,
            player,
            sounds: Vec::new(),
            clock: BattleClock::default(),
            pause: None,
            pause_sound: None,
            settings: Default::default(),
            abort_wait: Default::default(),
            options_idle: Duration::ZERO,
            pause_input: PauseInput::Ready,
            control_arrows: [false; 2],
            mirror: false,
            radar: true,
            list_cars: false,
            view_input_held: false,
            helicopter_key_held: false,
            helicopter_unlocked: false,
            helicopter_active: false,
            rooftop_passage: Default::default(),
            helicopter,
            initial_helicopter_route,
            audio_restart_count: 0,
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
pub(crate) fn notice_selected(keys: &ButtonInput<KeyCode>, pad: Option<&Gamepad>) -> bool {
    keys.any_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::KeyP])
        || pad.is_some_and(|p| {
            p.any_pressed([
                GamepadButton::South,
                GamepadButton::Select,
                GamepadButton::Start,
            ])
        })
}

pub fn advance(
    mut arena: Option<ResMut<crate::render::Arena>>,
    mut drive: ResMut<Drive>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut window: Single<&mut Window>,
    mut capture: ResMut<crate::smoke::Capture>,
    mut timings: Local<Vec<f64>>,
    strategy: Option<Res<bevy::time::TimeUpdateStrategy>>,
) {
    drive.sounds.clear();
    drive.pause_sound = None;
    if capture.holding_battle_frame() {
        return;
    }
    const STEP: Duration = Duration::from_millis(50);
    let first_pad = gamepads.iter().min_by_key(|(id, _)| *id);
    let pad = first_pad.map(|(_, pad)| pad);
    let pressed = |button| pad.is_some_and(|p| p.pressed(button));
    let just_pressed = |button| pad.is_some_and(|p| p.just_pressed(button));
    let regained = window.focused && !drive.was_focused;
    drive.was_focused = window.focused;
    if drive.lives == 0 {
        return;
    }
    if window.focused
        && drive.pause != Some(crate::pause::Page::Aborted)
        && (keys.just_pressed(KeyCode::KeyR)
            || (drive.pause.is_none()
                && drive.pause_input == PauseInput::Ready
                && !drive.showing_boss_intro()
                && drive.lost_life.is_none()
                && just_pressed(GamepadButton::Select)))
    {
        drive.last_listener = None;
        drive.contacts.reset_projectiles();
        drive.pause = None;
        drive.pause_input = PauseInput::Ready;
        drive.control_arrows = [false; 2];
        let state = &mut *drive;
        state.player.reset();
        drive.rooftop_passage = Default::default();
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
        let level_seed = drive.level_seed;
        drive.player.set_rng_state(level_seed);
        let route = drive.initial_helicopter_route;
        drive.helicopter = Drive::stage_helicopter(&mut drive.player, route);
        let reset_weapons = drive.reset_weapons_on_entry;
        drive.player.initialize_battle_car(reset_weapons);
        let state = &mut *drive;
        state.player.set_pad_layout(state.settings.layout);
        state.camera.update(&state.player, true);
        let seed = drive.battle_seed;
        drive.begin_battle(seed);
        if let Some(arena) = arena.as_deref_mut() {
            arena.reset_animation();
        }
        drive.audio_restart_count = drive.audio_restart_count.wrapping_add(1);
        drive.clock = BattleClock::default();
        return;
    }
    use crate::pause::{Action, Page};
    let start = keys.just_pressed(KeyCode::KeyP) || just_pressed(GamepadButton::Start);
    let pause_input_held = || crate::pause::campaign_pad_held(&keys, pad);
    if drive.pause_input != PauseInput::Ready {
        if window.focused && !pause_input_held() {
            drive.control_arrows = [false; 2];
            if let PauseInput::ChangeOnRelease(next) = drive.pause_input {
                drive.pause = next;
            }
            drive.pause_input = PauseInput::Ready;
        }
        drive.clock = BattleClock::default();
        return;
    }
    if let Some(page) = drive.pause {
        if window.focused {
            if page == Page::Aborted {
                let confirm = keys.any_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::KeyP])
                    || pad.is_some_and(|p| {
                        p.any_pressed([
                            GamepadButton::Start,
                            GamepadButton::Select,
                            GamepadButton::South,
                        ])
                    });
                drive.abort_wait.advance(time.delta(), confirm);
            } else {
                let action = crate::pause::campaign_action(page, &keys, pad);
                if page != Page::Controls && pause_input_held() {
                    drive.pause_input = PauseInput::WaitRelease;
                }
                let expired = crate::pause::options_timed_out(
                    page,
                    &mut drive.options_idle,
                    time.delta(),
                    action.is_some(),
                );
                if let Some(action) = action.or(expired.then_some(Action::Resume)) {
                    let old_layout = drive.settings.layout;
                    drive.pause = crate::pause::navigate(page, action, &mut drive.settings);
                    if page == Page::Controls {
                        drive.control_arrows = [
                            matches!(action, Action::Left),
                            matches!(action, Action::Right),
                        ];
                    }
                    if !expired {
                        drive.pause_input = PauseInput::WaitRelease;
                        drive.pause_sound = crate::pause::navigation_sound(
                            page,
                            action,
                            old_layout != drive.settings.layout,
                        );
                    }
                    if page == Page::Controls && drive.pause.is_none() {
                        let layout = drive.settings.layout;
                        drive.player.set_pad_layout(layout);
                    }
                    let next = drive.pause;
                    drive.pause =
                        drive
                            .pause_input
                            .defer_transition(page, next, pause_input_held());
                }
            }
        }
        drive.clock = BattleClock::default();
        return;
    }
    if window.focused && start && !drive.showing_boss_intro() && drive.lost_life.is_none() {
        drive.pause = Some(Page::Choices(0));
        drive.pause_input = PauseInput::WaitRelease;
        drive.clock = BattleClock::default();
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
        drive.clock = BattleClock::default();
        return;
    }
    if !capture.demo && (!window.focused || regained) {
        drive.clock = BattleClock::default();
        return;
    }
    if drive.showing_boss_intro() || drive.lost_life.is_some() {
        let elapsed = if capture.demo && !capture.realtime {
            STEP
        } else {
            time.delta()
        };
        let selected = notice_selected(&keys, pad);
        if drive.showing_boss_intro() {
            if !drive.boss_input_released {
                // screenBossCar polls GetPadStatus(0), which reads the upper
                // controller halfword. ctlpadAnySelect(0) then reads the lower one.
                let second_pad = gamepads
                    .iter()
                    .filter(|(id, _)| first_pad.is_some_and(|(first, _)| *id != first))
                    .min_by_key(|(id, _)| *id);
                drive.boss_input_released =
                    second_pad.is_none_or(|(_, p)| p.get_pressed().next().is_none());
            }
            if drive.boss_input_released {
                drive.pause_sound = selected.then_some(true);
                drive.tick_boss_intro(elapsed, selected);
            }
        } else {
            drive.tick_lost_life(elapsed, selected, || {
                crate::campaign::timer_ticks(&time, strategy.as_deref(), capture.demo)
            });
        }
        drive.clock = BattleClock::default();
        return;
    }
    let input = Input {
        cycle_weapon: keys.pressed(KeyCode::KeyE),
        previous_weapon: keys.pressed(KeyCode::KeyQ),
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
    let elapsed = if capture.demo && !capture.realtime {
        STEP
    } else {
        time.delta()
    };
    if let Some(fields) = drive.clock.advance(elapsed) {
        if capture.realtime {
            capture.steps += 1;
            capture.fields += u64::from(fields);
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
        if !capture.demo && keys.pressed(KeyCode::Digit8) {
            pads = 0x180b;
        } else if state.player.cheat_controls() && !capture.demo {
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
        state.capture_primary_view();
        state.contacts.radar.display(state.radar);
        let prepared =
            state
                .contacts
                .begin_bout_step(&mut state.player, &mut state.opponents, fields);
        let update_player = prepared.updates_player();
        if update_player {
            let before = state.selected_view();
            state.check_view_keys(&keys, pads);
            state.update_rooftop_passage(before);
            state
                .player
                .set_battle_view(state.selected_view(), state.camera.fixed_pose.is_some());
        }
        state.contacts.finish_bout_step(
            &mut state.player,
            &mut state.opponents,
            pads,
            prepared,
            state.mirror,
        );
        battle_audio(
            state.player.frame(),
            &state.contacts,
            &state.listener,
            state.settings.effects_volume(),
            state.recharge_sound(),
            state.last_listener.as_ref().unwrap_or(&state.listener),
            false,
            |slot| {
                if slot == 0 {
                    state.player.vehicle()
                } else {
                    state.opponents[slot - 1].vehicle()
                }
            },
            |request| state.sounds.push(request),
        );
        if state.helicopter_active
            && let Some(heli) = &mut state.helicopter
        {
            heli.update(&state.player, fields, false);
        } else {
            state.camera.after_step(&state.player, update_player);
        }
        state.last_listener = Some(if state.mirror {
            tm1_core::drive::SoundListener::rear_view(&state.player, &state.camera)
        } else {
            state.listener
        });
        if let Some(arena) = arena.as_deref_mut().filter(|arena| arena.has_animations) {
            arena.advance_view_animation(
                &state.player,
                &state.contacts,
                false,
                &state.primary_cars,
                state.primary_hidden.then_some(0),
                false,
                state.listener,
            );
            if state.mirror {
                let poses: Vec<_> = std::iter::once(CarPose::player(&state.player))
                    .chain(state.opponents.iter().map(CarPose::rival))
                    .collect();
                arena.advance_view_animation(
                    &state.player,
                    &state.contacts,
                    true,
                    &poses,
                    (!state.camera.show_body).then_some(0),
                    false,
                    state.last_listener.unwrap(),
                );
            }
        }
        if let Some(start) = started {
            crate::smoke::sample("Simulation step", &mut timings, start);
        }
        if state.contacts.level_won() {
            state.advance_victory_delay(fields.into());
        } else if state.player.health().0 <= 0 {
            state.advance_death(fields.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn flamethrower_retrigger_uses_one_retail_queue_entry() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(0x8016fecc + 24, &(3357u32 * 3357).to_le_bytes());
        for fill in 41..=46 {
            cpu.put(0x8018bf7c, &u32::MAX.to_le_bytes());
            let mut queue = BattleQueue::new();
            let mut actual = Vec::new();
            for _ in 0..fill {
                cpu.call(0x80106d60, &[0x82, 0, 0, 0, 0, 0]);
                queue.submit(BattleRequest::Turbo, &mut |_| {});
            }
            for _ in 0..2 {
                cpu.call(0x80107c6c, &[7, 0, 0]);
                queue.submit(
                    BattleRequest::Play(BattleSound {
                        frame: 0,
                        exclusive: effect_voice(11),
                        id: 6,
                        volume_percent: 99,
                        channels: [33; 2],
                        range: 0,
                    }),
                    &mut |r| {
                        actual.push(match r {
                            BattleRequest::Stop(ExclusiveVoice::WeaponLaunch(7)) => 0,
                            BattleRequest::Play(_) => 1,
                            _ => unreachable!(),
                        })
                    },
                );
            }
            cpu.call(0x80106d60, &[0x82, 0, 0, 0, 0, 0]);
            queue.submit(BattleRequest::Turbo, &mut |_| actual.push(2));
            let mut expected = Vec::new();
            for i in fill..cpu.word(0x8018bf7c) {
                let at = 0x801984e8 + 28 * i;
                match (cpu.word(at), cpu.word(at + 12), cpu.word(at + 24)) {
                    (0xf, 4, 2) => expected.extend([0, 1]),
                    (0x82, 0, 2) => expected.push(2),
                    packet => panic!("unexpected sound request {packet:?}"),
                }
            }
            assert_eq!(actual, expected, "prior requests {fill}");
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_queue_matches_retail_at_the_last_request() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(0x8016fecc + 24, &(3357u32 * 3357).to_le_bytes());
        let gun = ExclusiveVoice::MachineGuns(Vehicle::YellowJacket);
        for fill in 0..=50 {
            cpu.put(0x8018bf7c, &u32::MAX.to_le_bytes());
            let mut queue = BattleQueue::new();
            let mut accepted = Vec::new();
            let mut deferred = false;
            let mut emit = |r| match r {
                BattleRequest::Turbo => accepted.push((2, 0x82)),
                BattleRequest::EngineStop(0) => accepted.push((3, 0x4e)),
                BattleRequest::Stop(k) if k == gun => accepted.push((3, 0x5b)),
                BattleRequest::EngineRev(0, _) => accepted.push((2, 0x4e)),
                BattleRequest::EngineDeferred(0) => deferred = true,
                _ => unreachable!(),
            };
            for _ in 0..fill {
                cpu.call(0x80106d60, &[0x82, 0, 0, 0, 0, 0]);
                queue.submit(BattleRequest::Turbo, &mut emit);
            }
            cpu.call(0x80107a1c, &[Vehicle::YellowJacket as u32]);
            queue.submit(BattleRequest::EngineStop(0), &mut emit);
            queue.submit(BattleRequest::Stop(gun), &mut emit);
            cpu.call(0x80106d60, &[0x4e, 0, 0, 1, 30, 0]);
            queue.submit(BattleRequest::EngineRev(0, Default::default()), &mut emit);
            let original: Vec<_> = (0..cpu.word(0x8018bf7c))
                .map(|i| {
                    let at = 0x801984e8 + 28 * i;
                    (cpu.word(at + 24), cpu.word(at))
                })
                .filter(|(_, id)| !matches!(id, 0 | 2))
                .collect();
            assert_eq!(accepted, original, "after {fill} turbo requests");
            assert_eq!(deferred, !original.contains(&(2, 0x4e)));
        }
    }

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
    fn rear_view_sparks_use_the_camera_installed_before_the_flush() {
        use crate::campaign::{Entry, Stage};
        use tm1_core::drive::{BattleEvent, Level, SoundListener};
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let stage = Stage::load(
            &disc,
            Level::Warehouse,
            Vehicle::Outlaw,
            Entry::New(Default::default()),
        )
        .unwrap();
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
        let mut previous_mirror = None;
        let mut distinct = false;
        for frame in 0..4 {
            let mut keys = ButtonInput::default();
            if frame == 0 || frame == 2 {
                keys.press(KeyCode::KeyB);
            }
            if frame == 0 {
                keys.press(KeyCode::KeyF);
            }
            app.insert_resource(keys);
            tick(&mut app, 50);
            let drive = app.world().resource::<Drive>();
            assert_eq!(drive.mirror, frame < 2);
            let mut expected = Vec::new();
            let mut carried = 0;
            for (i, event) in drive.contacts.battle_events(&drive.listener).enumerate() {
                if let BattleEvent::Effect { kind: 1, position } = event {
                    let view = if i < drive.contacts.events_before_primary_view() {
                        carried += 1;
                        previous_mirror.as_ref().unwrap()
                    } else {
                        &drive.listener
                    };
                    let range = view.sound_range(position);
                    distinct |= range != drive.listener.sound_range(position);
                    expected.push((
                        range,
                        view.channels(position, drive.settings.effects_volume(), false),
                    ));
                }
            }
            assert_eq!(carried, if matches!(frame, 1 | 2) { 2 } else { 0 });
            let actual: Vec<_> = drive
                .sounds
                .iter()
                .filter_map(|r| match r {
                    BattleRequest::Play(sound)
                        if sound.exclusive == Some(ExclusiveVoice::ScaledEffect(103)) =>
                    {
                        Some((sound.range, sound.channels))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(actual, expected);
            previous_mirror = drive
                .mirror
                .then(|| SoundListener::rear_view(&drive.player, &drive.camera));
        }
        assert!(distinct);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_notices_sample_retail_pad_halves_and_held_select() {
        use crate::campaign::{Entry, Stage};
        use tm1_core::drive::Level;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let stage = Stage::load(
            &disc,
            Level::Rooftops,
            Vehicle::RoadKill,
            Entry::New(Default::default()),
        )
        .unwrap();
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
        let mut pads = [
            app.world_mut().spawn(Gamepad::default()).id(),
            app.world_mut().spawn(Gamepad::default()).id(),
        ];
        pads.sort();
        tick(&mut app, 1);
        app.world_mut().resource_mut::<Drive>().start_with_minion();
        for (second_held, raw) in [(true, 0x1000_0100), (false, 0x100)] {
            let mut first = app.world_mut().get_mut::<Gamepad>(pads[0]).unwrap();
            first.digital_mut().press(GamepadButton::Select);
            first.digital_mut().clear();
            let mut second = app.world_mut().get_mut::<Gamepad>(pads[1]).unwrap();
            if second_held {
                second.digital_mut().press(GamepadButton::DPadUp);
            } else {
                second.digital_mut().release(GamepadButton::DPadUp);
            }
            cpu.call(0x800ffb98, &[2, raw]);
            let waiting = cpu.call(0x800ffb50, &[0]) != 0;
            let selected = cpu.call(0x800fffc4, &[0]) != 0;
            tick(&mut app, 17);
            let drive = app.world().resource::<Drive>();
            assert_eq!(drive.showing_boss_intro(), waiting || !selected);
            assert_eq!(
                drive.audio_restart_count,
                u32::from(!drive.showing_boss_intro())
            );
        }
        assert_eq!(
            app.world()
                .resource::<Drive>()
                .opponents
                .last()
                .unwrap()
                .vehicle(),
            Vehicle::Minion
        );
        app.world_mut()
            .get_mut::<Gamepad>(pads[0])
            .unwrap()
            .digital_mut()
            .reset_all();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::Space);
        keys.clear();
        app.world_mut().resource_mut::<Drive>().advance_death(181);
        tick(&mut app, 17);
        let drive = app.world().resource::<Drive>();
        assert!(drive.lost_life.is_none());
        assert_eq!(drive.pause_sound, Some(true));
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn life_loss_and_minion_intro_switch_between_battle_audio_and_shell_rev() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::sfx::{Engine, OneShot, ShellEngine, pcm::StereoPcm};
        use bevy::ecs::system::RunSystemOnce;
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            tm1_core::drive::Level::Rooftops,
            Vehicle::RoadKill,
            Entry::New(Default::default()),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let mix = |world: &mut World| {
            world.run_system_once(crate::sfx::update).unwrap();
            world.run_system_once(crate::music::control).unwrap();
        };
        let battle_engines = |world: &mut World| {
            world
                .query_filtered::<Entity, (With<Engine>, With<AudioPlayer<StereoPcm>>)>()
                .iter(world)
                .count()
        };
        let shell_engines = |world: &mut World| {
            world
                .query_filtered::<Entity, With<ShellEngine>>()
                .iter(world)
                .collect::<Vec<_>>()
        };
        let music_paused = |world: &mut World| {
            world
                .query_filtered::<&PlaybackSettings, With<crate::music::ArenaMusic>>()
                .single(world)
                .unwrap()
                .paused
        };
        let track = world
            .query_filtered::<Entity, With<crate::music::ArenaMusic>>()
            .single(&world)
            .unwrap();
        for boss in [false, true] {
            world
                .resource_mut::<Drive>()
                .sounds
                .push(BattleRequest::Play(BattleSound {
                    frame: 0,
                    id: 4,
                    exclusive: None,
                    volume_percent: 99,
                    channels: [99; 2],
                    range: 0,
                }));
            mix(&mut world);
            assert!(battle_engines(&mut world) >= 2);
            assert!(world.query::<&OneShot>().iter(&world).count() > 0);
            if boss {
                world.resource_mut::<Drive>().advance_victory_delay(181);
            } else {
                world.resource_mut::<Drive>().advance_death(181);
            }
            mix(&mut world);
            let motors = shell_engines(&mut world);
            assert_eq!(motors.len(), 2);
            assert_eq!(battle_engines(&mut world), 0);
            assert_eq!(world.query::<&OneShot>().iter(&world).count(), 0);
            assert!(music_paused(&mut world));
            mix(&mut world);
            assert_eq!(shell_engines(&mut world), motors);
            world.resource_mut::<Drive>().sounds.clear();
            let before = world
                .entity(track)
                .get_ref::<AudioPlayer<crate::music::CdTrack>>()
                .unwrap()
                .last_changed();
            if boss {
                world
                    .resource_mut::<Drive>()
                    .tick_boss_intro(Duration::from_secs(5), false);
                assert_eq!(
                    world.resource::<Drive>().opponents[3].vehicle(),
                    Vehicle::Minion
                );
            } else {
                world.resource_mut::<Drive>().tick_lost_life(
                    Duration::from_millis(1500),
                    false,
                    || 1,
                );
            }
            mix(&mut world);
            assert!(shell_engines(&mut world).is_empty());
            assert!(battle_engines(&mut world) >= 2);
            assert!(!music_paused(&mut world));
            assert_ne!(
                world
                    .entity(track)
                    .get_ref::<AudioPlayer<crate::music::CdTrack>>()
                    .unwrap()
                    .last_changed(),
                before
            );
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn keyboard_view_combinations_match_jp_check_vr_mode() {
        use crate::campaign::{Entry, Stage};
        use KeyCode::*;
        use tm1_core::drive::Level;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut drive = Stage::load(
            &disc,
            Level::Arena,
            Vehicle::YellowJacket,
            Entry::New(Default::default()),
        )
        .unwrap()
        .drive;
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        const PLAYER: u32 = 0x8019ee40;
        cpu.put(PLAYER, drive.player.state());
        cpu.put(PLAYER + 0xf4, &(drive.camera.mode as u32).to_le_bytes());
        cpu.put(0x8018bf24, &1u32.to_le_bytes()); // rear view enabled by viewInit
        // Stop after the view-input branches, before polling the cheat controller.
        for mask in 0u8..32 {
            for bits in [0, mask, mask, mask ^ 31] {
                let mut keys = ButtonInput::default();
                for (bit, (key, offset)) in [
                    (KeyC, 0xda),
                    (KeyZ, 0xd9),
                    (KeyV, 0xdc),
                    (KeyL, 0xdf),
                    (KeyB, 0xdb),
                ]
                .into_iter()
                .enumerate()
                {
                    let down = (bits >> bit) & 1;
                    if down != 0 {
                        keys.press(key);
                    }
                    cpu.put(PLAYER + offset, &[down]);
                }
                cpu.call_until(0x8014eff4, &[PLAYER], 0x8014f278);
                drive.check_view_keys(&keys, 0);
                assert_eq!(
                    (
                        drive.camera.mode as u32,
                        drive.radar,
                        drive.list_cars,
                        drive.mirror,
                        drive.view_input_held
                    ),
                    (
                        cpu.word(PLAYER + 0xf4),
                        cpu.word(0x8018c30c) != 0,
                        cpu.word(0x8018c5e8) != 0,
                        cpu.word(0x8018bf20) != 0,
                        cpu.word(0x8018c3ac) & 0xffff == 0
                    ),
                    "keys {bits:05b}"
                );
            }
        }
        cpu.put(PLAYER + 0xd9, &[0; 7]);
        for pads in [0x180a, 0x1180b, 0x180b, 0] {
            cpu.call(0x800ffb98, &[2, pads]);
            cpu.call(0x801125ac, &[PLAYER]);
            cpu.call(0x8014eff4, &[PLAYER]);
            drive.check_view_keys(&ButtonInput::default(), pads);
            assert_eq!(drive.helicopter_unlocked, cpu.ram[0x18c364] != 0);
        }
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
        app.init_resource::<Time<Real>>();
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
            tick(app, 0);
        };
        let frame = app.world().resource::<Drive>().player.frame();
        use KeyCode::*;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyP);
        tick(&mut app, 50);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(ArrowDown);
        tick(&mut app, 50);
        assert_eq!(
            app.world().resource::<Drive>().pause,
            Some(Page::Choices(0))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyP);
        tick(&mut app, 50);
        assert_eq!(
            app.world().resource::<Drive>().pause,
            Some(Page::Choices(0))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(ArrowDown);
        tick(&mut app, 0);
        for key in [ArrowDown, ArrowDown] {
            tap(&mut app, key);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(Enter);
        tick(&mut app, 50);
        tick(&mut app, 500);
        assert_eq!(
            app.world().resource::<Drive>().pause,
            Some(Page::Choices(2))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(Enter);
        tick(&mut app, 0);
        tap(&mut app, ArrowRight);
        let drive = app.world().resource::<Drive>();
        assert_eq!(drive.pause, Some(Page::Audio(0)));
        assert_eq!((drive.settings.music, drive.settings.effects), (8, 6));
        assert!(
            drive.settings.cd_gain() > crate::pause::Settings::default().cd_gain()
                && drive.settings.effects_volume() < 33
        );
        assert_eq!(drive.player.frame(), frame);
        for key in [ArrowUp, Enter, KeyP, ArrowUp, Enter] {
            tap(&mut app, key);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(ArrowRight);
        tick(&mut app, 50);
        assert_eq!(
            app.world().resource::<Drive>().control_arrows,
            [false, true]
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(Space);
        tick(&mut app, 50);
        assert_eq!(app.world().resource::<Drive>().pause, Some(Page::Controls));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        tick(&mut app, 0);
        assert_eq!(app.world().resource::<Drive>().control_arrows, [false; 2]);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(Space);
        tick(&mut app, 50);
        tick(&mut app, 1000);
        assert_eq!(app.world().resource::<Drive>().pause, Some(Page::Controls));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(Space);
        tick(&mut app, 0);
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
        world.init_resource::<Assets<crate::texture_blend::TextureBlend>>();
        world.init_resource::<Assets<AudioSource>>();
        world.insert_resource(Campaign(disc));
        world.init_resource::<Time<Real>>();
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
            drive.radar = false;
            drive.list_cars = true;
            drive.camera.mode = tm1_core::drive::CarView::Far;
            let route = drive.helicopter_route();
            assert_ne!(route, HelicopterRoute::default());
            // Inject the shell victory event at the end of its return delay.
            drive.lives = 2;
            drive.opponents.clear();
            drive.advance_victory_delay(181);
            crate::campaign::advance(&mut world);
            crate::transition::complete(&mut world);
            let drive = world.resource::<Drive>();
            assert_eq!(drive.player.level(), level);
            assert_eq!(drive.initial_helicopter_route, route);
            let expected = tm1_core::drive::SoundListener::new(&drive.player, &drive.camera, None);
            assert_eq!(drive.camera.mode, tm1_core::drive::CarView::Far);
            assert_eq!(
                drive.listener.sound_range(drive.player.position()),
                expected.sound_range(drive.player.position())
            );
            assert!(!drive.radar && drive.list_cars);
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
                    .query::<&crate::sfx::RivalEngine>()
                    .iter(&world)
                    .map(|engine| engine.slot)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
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
            let mut stage = stage;
            stage.drive.begin_battle(456);
            let opening_seed = stage.drive.player.rng_state();
            let opening_holes = stage.drive.windshield_holes;
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
                        .filter_map(|i| drive.contacts.pedestrian_pose(i, true))
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
                drive.tick_lost_life(Duration::from_millis(1499), false, || unreachable!());
                assert!(drive.lost_life.is_some());
                drive.tick_lost_life(Duration::from_millis(1), false, || 789);
                let mut expected = 789;
                for _ in 0..12 {
                    tm1_core::drive::bios_rand(&mut expected);
                }
                assert_eq!(drive.player.rng_state(), expected);
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
            assert_eq!(start.1, opening_seed);
            assert_eq!(
                app.world().resource::<Drive>().windshield_holes,
                opening_holes
            );
            let run = |app: &mut App| {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::KeyW);
                let targets = |app: &App, secondary| {
                    let scene = &app.world().resource::<Drive>().contacts;
                    (
                        std::array::from_fn::<_, 6, _>(|i| scene.pedestrian_pose(i, secondary)),
                        std::array::from_fn::<_, 2, _>(|i| scene.merc_pose(i, secondary)),
                        scene.hidden_pickups(secondary),
                        scene.health_stands(secondary),
                    )
                };
                for step in 0..180 {
                    let before = targets(app, true);
                    tick(app, 50);
                    assert_eq!(targets(app, false), before, "{level:?} step {step}");
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
                PlayerVehicle::for_vehicle(tm1_core::drive::Vehicle::YellowJacket),
                scene,
                &[Vehicle::Outlaw],
                Default::default(),
                || 1,
                true,
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
                d.contacts.hidden_pickups(true),
            )
        };
        let (mut app, _) = build();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyB);
        tick(&mut app, 50);
        assert!(app.world().resource::<Drive>().mirror);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyV);
        tick(&mut app, 50);
        assert!(app.world().resource::<Drive>().radar);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        tick(&mut app, 50);
        for key in [KeyCode::KeyB, KeyCode::KeyL, KeyCode::KeyV] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
        }
        tick(&mut app, 50);
        let drive = app.world().resource::<Drive>();
        assert!(!drive.radar && !drive.list_cars && drive.mirror);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        tick(&mut app, 50);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit8);
        tick(&mut app, 50);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Digit8);
        {
            let mut drive = app.world_mut().resource_mut::<Drive>();
            assert!(drive.helicopter_unlocked);
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
            tick(&mut app, 50);
            let drive = app.world().resource::<Drive>();
            assert_eq!(
                (drive.camera.mode, drive.helicopter_active),
                (mode, helicopter)
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(key);
            tick(&mut app, 50);
        }
        // A late presentation advances one actor update using the elapsed fields.
        // Run the retail timer with the same field intervals and a zero hardware counter.
        let (mut slow, _) = build();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        cpu.put(0x80198a94, &60u32.to_le_bytes());
        cpu.put(0x80162f40, &[8, 0, 224, 3, 0x21, 0x10, 0, 0]);
        for (ms, elapsed_fields) in [(50, 3), (67, 4), (83, 5), (200, 12)] {
            let ticks = elapsed_fields * tm1_core::TICKS_PER_FIELD;
            assert!(cpu.call(0x80109dac, &[ticks]) <= 20);
            cpu.put(0x80170248, &0u32.wrapping_sub(ticks).to_le_bytes());
            cpu.call(0x80109c08, &[]);
            let fields = cpu.call(0x80109d00, &[]) as u8;
            let before = slow.world().resource::<Drive>().player.frame();
            tick(&mut slow, ms);
            let player = &slow.world().resource::<Drive>().player;
            assert_eq!(player.frame(), before + 1);
            assert_eq!(player.last_fields(), fields);
            tick(&mut slow, 0);
            assert_eq!(slow.world().resource::<Drive>().player.frame(), before + 1);
        }
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
            let drive = key_app.world().resource::<Drive>();
            let before: Vec<_> = std::iter::once(crate::render::CarPose::player(&drive.player))
                .chain(drive.opponents.iter().map(crate::render::CarPose::rival))
                .collect();
            tick(&mut key_app, 50);
            assert_eq!(key_app.world().resource::<Drive>().primary_cars, before);
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
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyP);
        app.world_mut().despawn(id);
        tick(&mut app, 0);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        tick(&mut app, 50);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Enter);
        tick(&mut app, 0);
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
