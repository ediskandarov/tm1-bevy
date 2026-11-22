//! Keyboard contest selection and five-slot JP stage passwords.
use crate::campaign::{self, Campaign, PasswordModes, Stage};
use crate::cinema::{Cinema, CinemaStep, Clip};
use anyhow::Result;
use bevy::{audio::AudioSinkPlayback, prelude::*};
use std::time::Duration;
use tm1_assets::screen::{self, Face, Fonts};
use tm1_core::drive::{Difficulty, Level, Vehicle};

#[derive(Component)]
struct ContestScreen;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Title,
    DuelJoin(Option<usize>),
    DuelGarage {
        slot: usize,
        second: bool,
        bio: bool,
    },
    Duel(usize),
    DuelOptions(usize),
    DuelStage,
    DuelControls(usize),
    MainOptions(usize),
    Audio(usize),
    StaffRoll,
    History,
    Showcase(usize),
    Contest,
    Battle(usize),
    Options(usize),
    Controls,
    Bio,
    Password,
    Wrong(usize),
    BossPreview(std::time::Duration),
}
const SOUNDTRACK: [(usize, &str); 7] = [
    (2, "''TWISTED THEME''"),
    (3, "''CIRCUS METALLICUS''"),
    (5, "''ASPHALT ASSAULT''"),
    (7, "''CYBURB SLIDE''"),
    (6, "''CYBURB HUNT''"),
    (4, "''STALK N' ROLL''"),
    (8, "''DROP DEAD''"),
];
pub const INFINITE_WEAPONS: &str = "< =@@";
pub const MINION_AND_FRIENDS: &str = "<>@=<";
pub const HELICOPTER: &str = "@@<> ";
const GOD_MODE: &str = "=<> @";
#[derive(Resource)]
struct SecondPlayerLayout(u8);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
struct PlayerCars([Option<usize>; 2]);

impl Default for PlayerCars {
    fn default() -> Self {
        Self([Some(Vehicle::YellowJacket.card_index()), None])
    }
}

pub(crate) fn remember_battle_cars(world: &mut World, vehicles: &[Vehicle]) {
    let mut cars = world.get_resource_or_init::<PlayerCars>();
    for (slot, &vehicle) in cars.0.iter_mut().zip(vehicles) {
        if vehicle != Vehicle::Minion {
            *slot = Some(vehicle.card_index());
        }
    }
}

#[derive(Resource)]
pub struct Contest {
    cinema: Option<Cinema>,
    idle: Duration,
    fonts: Fonts,
    title: Vec<u8>,
    title_row: usize,
    duel_cars: [usize; 2],
    hidden_pick: Option<usize>,
    player_cars: PlayerCars,
    duel_stage: usize,
    second_layout: u8,
    battlegrounds: Vec<Vec<u8>>,
    duel_art: Vec<u8>,
    history_from_menu: bool,
    credits: crate::scroll::Scroll,
    credits_background: Vec<u8>,
    soundtrack: usize,
    playing_track: usize,
    garage: crate::garage::Garage,
    field_fraction: u128,
    history: crate::scroll::Scroll,
    history_background: Vec<u8>,
    battle: Vec<u8>,
    cards: Vec<Vec<[u8; 4]>>,
    options: Vec<u8>,
    controls: crate::pause::Art,
    pub(crate) settings: crate::pause::Settings,
    button_sounds: Vec<Handle<AudioSource>>,
    password_from_options: bool,
    cursor_fields: u32,
    password_wait_release: bool,
    cursor_redraw_pending: bool,
    cursor_erased: bool,
    controls_view: crate::pause::ControlsView,
    entry: Vec<u8>,
    image: Handle<Image>,
    theme: Handle<crate::music::CdTrack>,
    theme_entity: Option<Entity>,
    page: Page,
    vehicle: usize,
    level: Level,
    difficulty: Difficulty,
    modes: PasswordModes,
    slots: [u8; 5],
    cursor: usize,
    released: bool,
}
#[derive(Default, Resource)]
struct AttractCycle(u8);
impl Contest {
    pub(crate) fn duel_layouts(&self) -> [u8; 2] {
        [self.settings.layout, self.second_layout]
    }

    fn begin_showcase(&mut self) {
        self.garage.reset();
        self.field_fraction = 0;
        self.page = Page::Showcase(0);
    }
    fn presentation_fields(&mut self, elapsed: Duration) -> u32 {
        self.field_fraction += elapsed.as_nanos() * 60;
        let fields = (self.field_fraction / 1_000_000_000) as u32;
        self.field_fraction %= 1_000_000_000;
        fields
    }
    fn showcase_fields(&mut self, mut i: usize, mut fields: u32) -> bool {
        while fields > 0 {
            let n = fields.min(361 - self.garage.field);
            self.garage.tick(n);
            fields -= n;
            if self.garage.field == 361 {
                i += 1;
                if i == Vehicle::CARD_ORDER.len() {
                    return true;
                }
                self.garage.reset();
                self.garage.slide = 0;
            }
        }
        self.page = Page::Showcase(i);
        false
    }

    fn update_attract(&mut self, world: &mut World, select: bool, up: bool, down: bool) -> bool {
        let dt = menu_delta(world);
        match self.page {
            Page::Title => {
                self.idle += dt;
                if self.idle.as_nanos() * 60 < 1201 * 1_000_000_000 {
                    return false;
                }
                let mut cycle = world.get_resource_or_init::<AttractCycle>();
                let current = cycle.0;
                cycle.0 = (current + 1) % 3;
                self.idle = Duration::ZERO;
                match current {
                    0 => {
                        self.play(world, Some(Clip::Attract));
                        return true;
                    }
                    1 => {
                        self.history.reset();
                        self.field_fraction = 0;
                        self.page = Page::History;
                    }
                    _ => self.begin_showcase(),
                }
            }
            Page::StaffRoll | Page::History | Page::Showcase(_) => {
                let fields = self.presentation_fields(dt);
                if fields == 0 && !select {
                    return true;
                }
                let done = match self.page {
                    Page::StaffRoll => (0..fields.max(u32::from(select))).any(|_| {
                        self.credits.field(select, up, down) != crate::scroll::Outcome::Running
                    }),
                    Page::Showcase(i) => select || self.showcase_fields(i, fields),
                    Page::History => {
                        let mut skipped = false;
                        for _ in 0..fields.max(u32::from(select)) {
                            match self.history.field(select, up, down) {
                                crate::scroll::Outcome::Running => {}
                                crate::scroll::Outcome::Skipped => {
                                    skipped = true;
                                    break;
                                }
                                crate::scroll::Outcome::Finished => {
                                    self.begin_showcase();
                                    break;
                                }
                            }
                        }
                        skipped
                    }
                    _ => unreachable!(),
                };
                if done {
                    self.garage.reset();
                    if self.history_from_menu || self.page == Page::StaffRoll {
                        self.page = Page::MainOptions(if self.history_from_menu { 1 } else { 2 });
                        self.history_from_menu = false;
                        self.released = !select;
                        self.idle = Duration::ZERO;
                    } else {
                        self.page = Page::Title;
                        self.play(world, Some(Clip::Title));
                        return true;
                    }
                }
            }
            _ => return false,
        }
        world
            .resource_mut::<Assets<Image>>()
            .get_mut(&self.image)
            .unwrap()
            .data = Some(self.frame());
        self.sync_theme(world);
        true
    }
    fn sync_theme(&mut self, world: &mut World) {
        use crate::garage::GarageSound;
        let sound = match self.page {
            Page::Contest | Page::Bio => Some(GarageSound::Vehicle(self.vehicle)),
            Page::Battle(_)
            | Page::Options(_)
            | Page::Controls
            | Page::Password
            | Page::Wrong(_)
            | Page::BossPreview(_) => Some(GarageSound::Engine(Vehicle::CARD_ORDER[self.vehicle])),
            // The title clears gVehicleID. The fallback sound ID 0x52
            // plays the same engine as Crimson Fury.
            Page::DuelJoin(_) => Some(GarageSound::Engine(Vehicle::CrimsonFury)),
            Page::Duel(_) | Page::DuelOptions(_) | Page::DuelStage | Page::DuelControls(_) => {
                Some(GarageSound::Engine(if self.hidden_pick == Some(0) {
                    Vehicle::Minion
                } else {
                    Vehicle::CARD_ORDER[self.duel_cars[0]]
                }))
            }
            Page::DuelGarage { slot, .. } => Some(GarageSound::Vehicle(self.duel_cars[slot])),
            Page::Showcase(i) => Some(GarageSound::Vehicle(i)),
            _ => None,
        };
        self.garage
            .sound(world, sound, self.settings.effects_volume());
        if matches!(
            self.page,
            Page::Title
                | Page::MainOptions(_)
                | Page::Audio(_)
                | Page::StaffRoll
                | Page::History
                | Page::Showcase(_)
        ) && self.cinema.is_none()
        {
            if !matches!(self.page, Page::MainOptions(_) | Page::Audio(_)) {
                self.soundtrack = 0;
            }
            let volume = bevy::audio::Volume::Linear(self.settings.cd_gain());
            if self.playing_track != self.soundtrack || self.theme_entity.is_none() {
                let handle = if self.soundtrack == 0 {
                    self.theme.clone()
                } else {
                    match crate::music::CdTrack::load(
                        &world.resource::<Campaign>().0,
                        SOUNDTRACK[self.soundtrack].0,
                    ) {
                        Ok(track) => world
                            .resource_mut::<Assets<crate::music::CdTrack>>()
                            .add(track),
                        Err(err) => {
                            warn!("Loading soundtrack failed: {err:#}");
                            self.soundtrack = self.playing_track;
                            return;
                        }
                    }
                };
                if let Some(entity) = self.theme_entity.take() {
                    world.despawn(entity);
                }
                self.playing_track = self.soundtrack;
                self.theme_entity = Some(
                    world
                        .spawn((
                            campaign::StageEntity,
                            AudioPlayer(handle),
                            PlaybackSettings::ONCE.with_volume(volume),
                        ))
                        .id(),
                );
            } else if let Some(entity) = self.theme_entity {
                if let Some(mut sink) = world.get_mut::<AudioSink>(entity) {
                    sink.set_volume(volume);
                } else if let Some(mut playback) = world.get_mut::<PlaybackSettings>(entity) {
                    playback.volume = volume;
                }
            }
        } else if let Some(entity) = self.theme_entity.take() {
            world.despawn(entity);
        }
    }
    fn play_button(&self, world: &mut World, voice: Handle<AudioSource>) {
        world.spawn((
            AudioPlayer::new(voice),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(
                0.44 * self.settings.effects_gain(),
            )),
        ));
    }
    fn play(&mut self, world: &mut World, mut clip: Option<Clip>) {
        if let Some(voice) = self.cinema.take().and_then(|movie| movie.voice) {
            world.despawn(voice);
        }
        self.idle = Duration::ZERO;
        self.released = false;
        while let Some(which) = clip {
            match Cinema::load(&world.resource::<Campaign>().0, which) {
                Ok((mut movie, pixels, audio)) => {
                    if let Some(audio) = audio {
                        let sound = world.resource_mut::<Assets<AudioSource>>().add(audio);
                        movie.voice = Some(
                            world
                                .spawn((
                                    campaign::StageEntity,
                                    AudioPlayer::new(sound),
                                    PlaybackSettings::ONCE,
                                ))
                                .id(),
                        );
                    }
                    world
                        .resource_mut::<Assets<Image>>()
                        .get_mut(&self.image)
                        .unwrap()
                        .data = Some(pixels);
                    self.cinema = Some(movie);
                    self.sync_theme(world);
                    return;
                }
                Err(err) => warn!("Disc cinema unavailable: {err:#}"),
            }
            clip = which.next();
        }
        world
            .resource_mut::<Assets<Image>>()
            .get_mut(&self.image)
            .unwrap()
            .data = Some(self.frame());
        self.sync_theme(world);
    }
    fn duel_rows(&self, frame: &mut [u8], row: usize, bottom: i32, entries: &[(&str, &str)]) {
        for (i, &(label, value)) in entries.iter().enumerate() {
            let y = bottom - 20 * (entries.len() - 1 - i) as i32;
            let face = if row == i {
                Face::Selected
            } else {
                Face::Choice
            };
            self.fonts.at(frame, face, 30, y, label);
            self.fonts.at(frame, face, 160, y, value);
        }
    }
    fn frame(&self) -> Vec<u8> {
        let mut frame = match self.page {
            Page::Battle(_) => {
                let mut out = self.battle.clone();
                for (n, px) in self.cards[self.vehicle].iter().enumerate() {
                    if px[3] != 0 {
                        let at = 4 * ((65 + n / 128) * 320 + 94 + n % 128);
                        out[at..at + 4].copy_from_slice(px);
                    }
                }
                out
            }
            Page::MainOptions(_) | Page::Options(_) => self.options.clone(),
            Page::Audio(row) => {
                return self.controls.audio_frame(
                    &self.fonts,
                    row,
                    self.settings,
                    Some(SOUNDTRACK[self.soundtrack].1),
                );
            }
            Page::StaffRoll => {
                let mut out = self.credits_background.clone();
                self.credits.draw(&mut out, &self.fonts);
                return out;
            }
            Page::Controls => {
                return self.controls_view.frame(
                    &self.controls,
                    &self.fonts,
                    self.settings.layout,
                    None,
                );
            }
            Page::DuelJoin(_) => return self.garage.join_frame(&self.fonts),
            Page::DuelGarage { slot, second, bio } => {
                if bio {
                    return self.garage.bio(self.duel_cars[slot]);
                }
                return self.garage.duel_frame(
                    &self.fonts,
                    self.duel_cars[slot],
                    slot,
                    (second && self.hidden_pick.is_none()).then_some(self.duel_cars[1 - slot]),
                );
            }
            Page::DuelControls(slot) => {
                return self.controls_view.frame(
                    &self.controls,
                    &self.fonts,
                    self.duel_layouts()[slot],
                    Some(slot),
                );
            }
            Page::DuelStage => {
                let mut out = self.battlegrounds[self.duel_stage].clone();
                self.fonts.at(&mut out, Face::Control, 64, 200, "<");
                self.fonts.at(&mut out, Face::Control, 239, 200, ">");
                self.fonts.centered_at(
                    &mut out,
                    Face::Choice,
                    159,
                    200,
                    &crate::duel::NAMES[self.duel_stage].to_uppercase(),
                );
                self.fonts
                    .centered_at(&mut out, Face::Control, 159, 220, "Press SELECT To Select");
                return out;
            }
            Page::DuelOptions(row) => {
                let mut out = self.options.clone();
                self.duel_rows(
                    &mut out,
                    row,
                    180,
                    &[
                        ("SELECT CONTROLS", ""),
                        (
                            "STAGE:",
                            &crate::duel::NAMES[self.duel_stage].to_uppercase(),
                        ),
                        ("EXIT", ""),
                    ],
                );
                return out;
            }
            Page::Duel(row) => {
                let mut out = self.duel_art.clone();
                for (slot, &index) in self.duel_cars.iter().enumerate() {
                    for (n, px) in self.cards[index].iter().enumerate() {
                        if px[3] != 0 {
                            let at = 4 * ((65 + n / 128) * 320 + 19 + 151 * slot + n % 128);
                            out[at..at + 4].copy_from_slice(px);
                        }
                    }
                }
                self.duel_rows(
                    &mut out,
                    row,
                    210,
                    &[("START", ""), ("OPTIONS", ""), ("EXIT", "")],
                );
                return out;
            }
            Page::Title => self.title.clone(),
            Page::Contest => return self.garage.frame(&self.fonts, self.vehicle, false),
            Page::History => {
                let mut out = self.history_background.clone();
                self.history.draw(&mut out, &self.fonts);
                return out;
            }
            Page::Showcase(i) => return self.garage.frame(&self.fonts, i, true),
            Page::Bio => return self.garage.bio(self.vehicle),
            _ => self.entry.clone(),
        };
        let mut line = |y, face, text: &str| self.fonts.centered(&mut frame, face, y, text);
        match self.page {
            Page::MainOptions(row) | Page::Battle(row) | Page::Options(row) => {
                let difficulty = match self.difficulty {
                    Difficulty::Easy => "EASY",
                    Difficulty::Medium => "MEDIUM",
                    Difficulty::Hard => "HARD",
                };
                let controls = ["CTL FREAK", "STANDARD", "DEADLY FORCE", "RUN N' GUN"]
                    [usize::from(self.settings.layout)];
                let choices: &[(&str, &str)] = if matches!(self.page, Page::Battle(_)) {
                    &[("START", ""), ("OPTIONS", ""), ("EXIT", "")]
                } else if matches!(self.page, Page::MainOptions(_)) {
                    &[
                        ("AUDIO CONTROLS", ""),
                        ("CONTEST HISTORY", ""),
                        ("STAFF ROLL", ""),
                        ("EXIT", ""),
                    ]
                } else {
                    &[
                        ("DIFFICULTY:", difficulty),
                        ("CONTROLS:", controls),
                        ("PASSWORD", ""),
                        ("EXIT", ""),
                    ]
                };
                let bottom = if matches!(self.page, Page::Battle(_)) {
                    210
                } else {
                    180
                };
                for (i, (label, value)) in choices.iter().enumerate() {
                    let face = if i == row {
                        Face::Selected
                    } else {
                        Face::Choice
                    };
                    let y = bottom - 20 * (choices.len() - 1 - i) as i32;
                    self.fonts.at(&mut frame, face, 30, y, label);
                    self.fonts.at(&mut frame, face, 160, y, value);
                }
            }
            Page::Audio(_) | Page::StaffRoll | Page::Controls => unreachable!(),
            Page::Title => {
                for (i, text) in ["1P CONTEST", "2P BATTLE", "OPTIONS"]
                    .into_iter()
                    .enumerate()
                {
                    line(
                        155 + 20 * i as i32,
                        if self.title_row == i {
                            Face::Selected
                        } else {
                            Face::Choice
                        },
                        text,
                    );
                }
            }
            Page::Duel(_)
            | Page::DuelJoin(_)
            | Page::DuelGarage { .. }
            | Page::DuelOptions(_)
            | Page::DuelControls(_)
            | Page::DuelStage => unreachable!(),
            Page::Contest | Page::Bio | Page::History | Page::Showcase(_) => unreachable!(),
            Page::Password | Page::Wrong(_) => {
                if let Page::Wrong(row) = self.page {
                    line(95, Face::PasswordLabel, "WRONG!");
                    for (i, label) in ["TRY AGAIN?", "EXIT"].into_iter().enumerate() {
                        line(
                            133 + i as i32 * 20,
                            if i == row {
                                Face::PasswordSelected
                            } else {
                                Face::PasswordLabel
                            },
                            label,
                        );
                    }
                } else {
                    line(113, Face::PasswordLabel, "ENTER");
                    line(133, Face::PasswordLabel, "PASSWORD");
                }
                self.fonts.at(
                    &mut frame,
                    Face::Password,
                    107,
                    183,
                    std::str::from_utf8(&self.slots).unwrap(),
                );
                if self.page == Page::Password && self.cursor_fields >= 20 && !self.cursor_erased {
                    self.fonts.at(
                        &mut frame,
                        Face::Password,
                        108 + self.cursor as i32 * self.fonts.password_stride(),
                        183,
                        "_",
                    );
                }
            }
            Page::BossPreview(_) => {
                for pixel in frame.chunks_exact_mut(4) {
                    for channel in &mut pixel[..3] {
                        *channel /= 2;
                    }
                }
                crate::render::boss_text(&self.fonts, &mut frame);
            }
        }
        frame
    }
    fn leave_battle(&mut self) {
        self.page = Page::Title;
        self.level = Level::Arena;
        self.modes = Default::default();
        self.idle = Duration::ZERO;
        self.released = false;
    }
    fn tick_setup_timeout(&mut self, held: bool, dt: Duration) -> bool {
        let limit = match self.page {
            Page::MainOptions(_) | Page::Audio(_) | Page::Options(_) => 18001,
            Page::DuelJoin(None) => 1200,
            Page::Duel(_) | Page::Battle(_) | Page::Wrong(_) => 1201,
            Page::Contest
            | Page::DuelGarage { bio: false, .. }
            | Page::DuelOptions(_)
            | Page::DuelStage => 18001,
            Page::Controls | Page::DuelControls(_) => crate::pause::CONTROL_IDLE_FIELDS,
            _ => return false,
        };
        if held {
            self.idle = Duration::ZERO;
            return false;
        }
        self.idle += dt;
        if self.idle.as_nanos() * 60 < limit * 1_000_000_000 {
            return false;
        }
        self.idle = Duration::ZERO;
        match self.page {
            Page::MainOptions(_) => self.page = Page::Title,
            Page::Audio(_) => self.page = Page::MainOptions(0),
            Page::Contest
            | Page::DuelJoin(_)
            | Page::DuelGarage { .. }
            | Page::Duel(_)
            | Page::Battle(_) => self.leave_battle(),
            Page::DuelOptions(_) => self.page = Page::Duel(1),
            Page::DuelStage => self.page = Page::DuelOptions(1),
            Page::DuelControls(_) => self.page = Page::DuelOptions(0),
            Page::Controls => self.page = Page::Options(1),
            Page::Wrong(_) => self.page = self.after_password(),
            _ => self.page = Page::Battle(0),
        }
        true
    }
    fn open_password(&mut self, from_options: bool) {
        self.password_from_options = from_options;
        self.cursor_fields = 0;
        self.field_fraction = 0;
        self.cursor_redraw_pending = false;
        self.cursor_erased = false;
        self.password_wait_release = true;
        self.slots = [b' '; 5];
        self.cursor = 0;
        self.page = Page::Password;
    }
    fn password_input(&mut self, keys: &ButtonInput<KeyCode>) -> Option<usize> {
        use KeyCode::*;
        if keys.just_pressed(Escape) {
            self.page = self.after_password();
            return None;
        }
        if self.password_wait_release {
            self.password_wait_release = crate::pause::campaign_pad_held(keys, None);
            return None;
        }
        let symbol = [
            (Digit1, b'<'),
            (Digit2, b'='),
            (Digit4, b'@'),
            (Digit3, b'>'),
            (Space, b' '),
        ]
        .into_iter()
        .find(|(key, _)| keys.pressed(*key));
        let edit_sound = if let Some((_, ch)) = symbol {
            self.slots[self.cursor] = ch;
            self.cursor = (self.cursor + 1) % 5;
            Some(1)
        } else if keys.pressed(ArrowLeft) || keys.pressed(ArrowRight) {
            self.cursor = (self.cursor + if keys.pressed(ArrowLeft) { 4 } else { 1 }) % 5;
            Some(0)
        } else {
            None
        };
        if edit_sound.is_some() {
            self.password_wait_release = true;
            self.cursor_erased = true;
            self.cursor_redraw_pending = true;
            return edit_sound;
        }
        if !keys.pressed(Enter) {
            return None;
        }
        let code = std::str::from_utf8(&self.slots).unwrap();
        self.page = self.after_password();
        match code {
            INFINITE_WEAPONS => self.modes.infinite_weapons = true,
            HELICOPTER => self.modes.helicopter = true,
            GOD_MODE => self.modes.god_mode = true,
            "<><<@" => {
                self.password_wait_release = true;
                self.page = Page::BossPreview(Duration::from_secs(5));
            }
            MINION_AND_FRIENDS => {
                self.modes.minion = true;
                self.level = Level::Rooftops;
            }
            _ => {
                if let Some(level) = Level::from_stage_password(code) {
                    self.level = level;
                    self.modes.minion = false;
                } else {
                    self.page = Page::Wrong(0);
                    self.idle = Duration::ZERO;
                }
            }
        }
        match self.page {
            Page::Wrong(_) => Some(3),
            Page::BossPreview(_) => None,
            _ => Some(2),
        }
    }

    fn after_password(&self) -> Page {
        if self.password_from_options {
            Page::Options(2)
        } else {
            Page::Contest
        }
    }
    fn input_sound(
        &self,
        before: Page,
        keys: &ButtonInput<KeyCode>,
    ) -> Option<Handle<AudioSource>> {
        use KeyCode::*;
        let enter = keys.just_pressed(Enter);
        let vertical = keys.any_just_pressed([ArrowUp, ArrowDown]);
        let horizontal = keys.any_just_pressed([ArrowLeft, ArrowRight]);
        let index = match before {
            Page::DuelJoin(Some(_)) if matches!(self.page, Page::DuelGarage { .. }) => 1,
            Page::DuelJoin(_) => return None,
            Page::DuelGarage { slot, bio, .. } => {
                let (select, vertical, horizontal) = if slot == 0 {
                    (Space, [KeyW, KeyS], [KeyA, KeyD])
                } else {
                    (Comma, [ArrowUp, ArrowDown], [ArrowLeft, ArrowRight])
                };
                if !bio && keys.any_just_pressed(vertical) {
                    0
                } else if !bio && horizontal.into_iter().all(|key| keys.pressed(key)) {
                    return None;
                } else if keys.just_pressed(select) {
                    1
                } else if !bio && self.garage.row != 2 && keys.any_just_pressed(horizontal) {
                    0
                } else {
                    return None;
                }
            }
            Page::Duel(_) | Page::DuelOptions(_) | Page::DuelStage if before != self.page => {
                usize::from(keys.any_just_pressed([Enter, Space, Comma, Escape]))
            }
            Page::DuelStage if horizontal || keys.any_just_pressed([KeyA, KeyD]) => 0,
            Page::Duel(0) if keys.any_just_pressed([Space, Comma]) => 1,
            Page::Options(0 | 1) if enter => 0,
            Page::StaffRoll | Page::History | Page::Showcase(_) => return None,
            Page::Title | Page::MainOptions(_) if vertical => 0,
            Page::Audio(n) if vertical || (horizontal && n < 3) => 0,
            Page::Contest if vertical => 0,
            Page::Contest if keys.pressed(ArrowLeft) && keys.pressed(ArrowRight) => return None,
            _ if enter => 1,
            Page::Battle(_) | Page::Wrong(_) if vertical => 0,
            Page::Options(n) if vertical || (n < 2 && horizontal) => 0,
            Page::Contest if self.garage.row != 2 && horizontal => 0,
            _ => return None,
        };
        Some(self.button_sounds[index].clone())
    }
    fn input(&mut self, keys: &ButtonInput<KeyCode>) -> bool {
        let key = |k| keys.just_pressed(k);
        match self.page {
            Page::StaffRoll | Page::History | Page::Showcase(_) => {}
            Page::Title => {
                if key(KeyCode::ArrowUp) {
                    self.title_row = (self.title_row + 2) % 3;
                } else if key(KeyCode::ArrowDown) {
                    self.title_row = (self.title_row + 1) % 3;
                } else if key(KeyCode::Enter) {
                    self.page = match self.title_row {
                        1 => {
                            self.duel_stage = 0;
                            self.duel_cars = [0, 0];
                            self.hidden_pick = None;
                            self.idle = Duration::ZERO;
                            self.field_fraction = 0;
                            Page::DuelJoin(None)
                        }
                        2 => Page::MainOptions(0),
                        _ => Page::Contest,
                    };
                    self.garage.reset();
                }
            }
            Page::DuelJoin(pending) => {
                let pressed = [1, 0]
                    .into_iter()
                    .find(|&slot| crate::pause::duel_pad_held(keys, slot));
                if key(KeyCode::Escape) {
                    self.leave_battle();
                } else if let Some(slot) = pending {
                    if pressed.is_none() {
                        self.page = Page::DuelGarage {
                            slot,
                            second: false,
                            bio: false,
                        };
                        self.idle = Duration::ZERO;
                        self.garage.reset();
                    }
                } else {
                    self.page = Page::DuelJoin(pressed);
                }
            }
            Page::DuelGarage { slot, second, bio } => {
                use KeyCode::*;
                let [up, down, left, right, select] = if slot == 0 {
                    [KeyW, KeyS, KeyA, KeyD, Space]
                } else {
                    [ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Comma]
                };
                if bio {
                    if key(select) || key(Escape) {
                        self.page = Page::DuelGarage {
                            slot,
                            second,
                            bio: false,
                        };
                    }
                } else if !key(Escape) && (key(up) || key(down)) {
                    self.garage.row = (self.garage.row + if key(up) { 2 } else { 1 }) % 3;
                } else if !key(Escape) && keys.pressed(left) && keys.pressed(right) {
                    if !second {
                        self.hidden_pick = Some(slot);
                        self.duel_cars[1 - slot] = self.duel_cars[slot];
                        self.page = Page::DuelGarage {
                            slot: 1 - slot,
                            second: true,
                            bio: false,
                        };
                    }
                    return false;
                } else if key(Escape) || (key(select) && self.garage.row == 2) {
                    self.leave_battle();
                } else if key(select) {
                    if self.garage.row == 1 {
                        self.page = Page::DuelGarage {
                            slot,
                            second,
                            bio: true,
                        };
                    } else if second {
                        for player in 0..2 {
                            if self.hidden_pick != Some(player) {
                                self.player_cars.0[player] = Some(self.duel_cars[player]);
                            }
                        }
                        let [Some(first), Some(other)] = self.player_cars.0 else {
                            // The retail card lookup reads past its table for an unassigned slot.
                            self.leave_battle();
                            return false;
                        };
                        self.duel_cars = [first, other];
                        self.page = Page::Duel(0);
                        self.idle = Duration::ZERO;
                    } else {
                        self.duel_cars[1 - slot] = usize::from(self.duel_cars[slot] == 0);
                        self.page = Page::DuelGarage {
                            slot: 1 - slot,
                            second: true,
                            bio: false,
                        };
                        self.garage.lock_portrait();
                    }
                } else if self.garage.row != 2 && (key(left) || key(right)) {
                    let n = Vehicle::CARD_ORDER.len();
                    let step = if key(left) { n - 1 } else { 1 };
                    let mut next = (self.duel_cars[slot] + step) % n;
                    if second && self.hidden_pick.is_none() && next == self.duel_cars[1 - slot] {
                        next = (next + step) % n;
                    }
                    self.duel_cars[slot] = next;
                    self.garage.slide = 0;
                    self.garage.left = key(left);
                }
            }
            Page::Duel(row) | Page::DuelOptions(row) => {
                use KeyCode::*;
                let options = matches!(self.page, Page::DuelOptions(_));
                let select = key(Enter) || key(Space) || key(Comma);
                let page = if options {
                    Page::DuelOptions
                } else {
                    Page::Duel
                };
                if key(Escape) || (select && row == 2) {
                    if options {
                        self.page = Page::Duel(1);
                    } else {
                        self.leave_battle();
                    }
                } else if key(ArrowDown) || key(KeyS) {
                    self.page = page((row + 1) % 3);
                } else if key(ArrowUp) || key(KeyW) {
                    self.page = page((row + 2) % 3);
                } else if select {
                    if options {
                        self.page = if row == 0 {
                            Page::DuelControls(usize::from(crate::pause::duel_pad_held(keys, 1)))
                        } else {
                            Page::DuelStage
                        };
                    } else if row == 0 {
                        return true;
                    } else {
                        self.page = Page::DuelOptions(0);
                    }
                } else if options
                    && row == 1
                    && (key(ArrowLeft) || key(KeyA) || key(ArrowRight) || key(KeyD))
                {
                    self.duel_stage =
                        (self.duel_stage + if key(ArrowLeft) || key(KeyA) { 5 } else { 1 }) % 6;
                    self.page = Page::DuelStage;
                }
            }
            Page::DuelStage => {
                use KeyCode::*;
                if key(Enter) || key(Space) || key(Comma) || key(Escape) {
                    self.page = Page::DuelOptions(1);
                } else if key(ArrowLeft) || key(KeyA) {
                    self.duel_stage = (self.duel_stage + 5) % 6;
                } else if key(ArrowRight) || key(KeyD) {
                    self.duel_stage = (self.duel_stage + 1) % 6;
                }
            }
            Page::DuelControls(_) => {}
            Page::MainOptions(row) => {
                if key(KeyCode::ArrowUp) {
                    self.page = Page::MainOptions((row + 3) % 4);
                } else if key(KeyCode::ArrowDown) {
                    self.page = Page::MainOptions((row + 1) % 4);
                } else if key(KeyCode::Escape) || (row == 3 && key(KeyCode::Enter)) {
                    self.page = Page::Title;
                } else if key(KeyCode::Enter) {
                    match row {
                        0 => self.page = Page::Audio(0),
                        1 => {
                            self.history_from_menu = true;
                            self.history.reset();
                            self.field_fraction = 0;
                            self.released = false;
                            self.page = Page::History;
                        }
                        2 => {
                            self.credits.reset();
                            self.field_fraction = 0;
                            self.released = false;
                            self.page = Page::StaffRoll;
                        }
                        _ => {}
                    }
                }
            }
            Page::Audio(row) => {
                if key(KeyCode::ArrowUp) {
                    self.page = Page::Audio((row + 3) % 4);
                } else if key(KeyCode::ArrowDown) {
                    self.page = Page::Audio((row + 1) % 4);
                } else if key(KeyCode::Escape) || (row == 3 && key(KeyCode::Enter)) {
                    self.page = Page::MainOptions(0);
                } else if key(KeyCode::ArrowLeft) || key(KeyCode::ArrowRight) {
                    let right = key(KeyCode::ArrowRight);
                    if row < 2 {
                        crate::pause::navigate(
                            crate::pause::Page::Audio(row as u8),
                            if right {
                                crate::pause::Action::Right
                            } else {
                                crate::pause::Action::Left
                            },
                            &mut self.settings,
                        );
                    } else if row == 2 {
                        self.soundtrack = (self.soundtrack
                            + if right { 1 } else { SOUNDTRACK.len() - 1 })
                            % SOUNDTRACK.len();
                    }
                }
            }
            Page::Bio => {
                if key(KeyCode::Enter) || key(KeyCode::Escape) {
                    self.page = Page::Contest;
                }
            }
            Page::Contest => {
                if key(KeyCode::ArrowUp) {
                    self.garage.row = (self.garage.row + 2) % 3;
                } else if key(KeyCode::ArrowDown) {
                    self.garage.row = (self.garage.row + 1) % 3;
                } else if !key(KeyCode::Escape)
                    && keys.pressed(KeyCode::ArrowLeft)
                    && keys.pressed(KeyCode::ArrowRight)
                {
                    return false;
                } else if key(KeyCode::Enter) && self.garage.row == 0 {
                    self.player_cars.0[0] = Some(self.vehicle);
                    self.page = Page::Battle(0);
                    self.idle = Duration::ZERO;
                } else if key(KeyCode::Enter) && self.garage.row == 1 {
                    self.page = Page::Bio;
                } else if key(KeyCode::Escape) || key(KeyCode::Enter) {
                    self.page = Page::Title;
                    self.modes = Default::default();
                    self.level = Level::Arena;
                } else if key(KeyCode::KeyD) {
                    self.difficulty = self.difficulty.next();
                } else if key(KeyCode::KeyP) {
                    self.open_password(false);
                } else if key(KeyCode::ArrowRight) && self.garage.row != 2 {
                    self.garage.slide = 0;
                    self.garage.left = false;
                    self.vehicle = (self.vehicle + 1) % Vehicle::CARD_ORDER.len();
                } else if key(KeyCode::ArrowLeft) && self.garage.row != 2 {
                    self.garage.slide = 0;
                    self.garage.left = true;
                    self.vehicle =
                        (self.vehicle + Vehicle::CARD_ORDER.len() - 1) % Vehicle::CARD_ORDER.len();
                }
            }
            Page::Battle(row) => {
                if key(KeyCode::ArrowUp) {
                    self.page = Page::Battle((row + 2) % 3);
                } else if key(KeyCode::ArrowDown) {
                    self.page = Page::Battle((row + 1) % 3);
                } else if key(KeyCode::Enter) && row == 0 {
                    return true;
                } else if key(KeyCode::Enter) && row == 1 {
                    self.page = Page::Options(0);
                } else if key(KeyCode::Escape) || key(KeyCode::Enter) {
                    self.leave_battle();
                }
            }
            Page::Options(row) => {
                if key(KeyCode::ArrowUp) {
                    self.page = Page::Options((row + 3) % 4);
                } else if key(KeyCode::ArrowDown) {
                    self.page = Page::Options((row + 1) % 4);
                } else if key(KeyCode::Escape) || (row == 3 && key(KeyCode::Enter)) {
                    self.page = Page::Battle(0);
                } else if key(KeyCode::Enter) || key(KeyCode::ArrowLeft) || key(KeyCode::ArrowRight)
                {
                    match row {
                        0 => {
                            self.difficulty = self.difficulty.next();
                            if key(KeyCode::ArrowLeft) {
                                self.difficulty = self.difficulty.next();
                            }
                        }
                        1 => self.page = Page::Controls,
                        2 if key(KeyCode::Enter) => {
                            self.open_password(true);
                        }
                        _ => {}
                    }
                }
            }
            Page::Controls => {}
            Page::Password | Page::BossPreview(_) => {}
            Page::Wrong(row) => {
                if key(KeyCode::ArrowUp) || key(KeyCode::ArrowDown) {
                    self.page = Page::Wrong(row ^ 1);
                } else if key(KeyCode::Enter) && row == 0 {
                    self.open_password(self.password_from_options);
                } else if key(KeyCode::Escape) || key(KeyCode::Enter) {
                    self.page = self.after_password();
                }
            }
        }
        false
    }
}
pub fn open(world: &mut World) -> Result<()> {
    let player = &world.resource::<crate::driving::Drive>().player;
    show(
        world,
        Vehicle::SweetTooth,
        Level::Arena,
        player.difficulty(),
        Some(Clip::Title),
    )
}
pub fn show(
    world: &mut World,
    vehicle: Vehicle,
    level: Level,
    difficulty: Difficulty,
    first_clip: Option<Clip>,
) -> Result<()> {
    let (settings, second_layout) = if let Some(duel) = world.get_resource::<crate::duel::Duel>() {
        let mut settings = duel.settings;
        settings.layout = duel.layouts[0];
        (settings, duel.layouts[1])
    } else {
        let settings = world
            .get_resource::<crate::driving::Drive>()
            .map(|drive| drive.settings)
            .unwrap_or_default();
        let second = world
            .get_resource::<SecondPlayerLayout>()
            .map_or(1, |saved| saved.0);
        (settings, second)
    };
    let player_cars = *world.get_resource_or_init::<PlayerCars>();
    let disc = &world.resource::<Campaign>().0;
    let theme = crate::music::CdTrack::load(disc, 2)?;
    let fonts = Fonts::load(disc)?;
    let mut menu = Contest {
        cinema: None,
        idle: Duration::ZERO,
        history: crate::scroll::Scroll::load(disc.read("UATEXT/HISTORY.TXT")?, &fonts)?,
        title_row: 0,
        duel_cars: [0, 1],
        hidden_pick: None,
        player_cars,
        duel_stage: 0,
        second_layout,
        battlegrounds: (1..=6)
            .map(|i| screen::background(&disc.read(&format!("UASCREEN/SCRNE3{i}.TIM"))?))
            .collect::<Result<_>>()?,
        duel_art: screen::background(&disc.read("UASCREEN/SCRNE2.TIM")?)?,
        history_from_menu: false,
        soundtrack: 0,
        playing_track: 0,
        credits: crate::scroll::Scroll::load(disc.read("UATEXT/CREDITS.TXT")?, &fonts)?,
        credits_background: screen::background(&disc.read("UASCREEN/SCRNROAD.TIM")?)?,
        history_background: screen::background(&disc.read("UASCREEN/SCRNCALY.TIM")?)?,
        fonts,
        garage: crate::garage::Garage::load(disc)?,
        field_fraction: 0,
        title: screen::background(&disc.read("UASCREEN/SCRNC.TIM")?)?,
        battle: screen::background(&disc.read("UASCREEN/SCRNE21P.TIM")?)?,
        cards: (0..screen::CARD_FILES.len())
            .map(|i| screen::load_card(disc, i))
            .collect::<Result<_>>()?,
        options: screen::background(&disc.read("UASCREEN/SCRNF.TIM")?)?,
        controls: crate::pause::Art::load(disc)?,
        settings,
        button_sounds: Vec::new(),
        password_from_options: false,
        cursor_fields: 0,
        password_wait_release: false,
        cursor_redraw_pending: false,
        cursor_erased: false,
        controls_view: Default::default(),
        entry: screen::background(&disc.read("UASCREEN/SCRNFD.TIM")?)?,
        theme: default(),
        theme_entity: None,
        image: default(),
        page: Page::Title,
        vehicle: Vehicle::CARD_ORDER
            .iter()
            .position(|v| *v == vehicle)
            .unwrap(),
        level,
        difficulty,
        modes: Default::default(),
        slots: [b' '; 5],
        cursor: 0,
        released: false,
    };
    let bank = disc.read("SND/UACORE.VAB")?;
    for program in [57, 58, 36, 41] {
        let sample = crate::sfx::Sample::load(&bank, program, false)?.wave()?;
        menu.button_sounds
            .push(world.resource_mut::<Assets<AudioSource>>().add(sample));
    }
    campaign::clear_stage(world);
    menu.image = crate::ending::spawn_screen(world, ContestScreen, menu.frame());
    menu.theme = world
        .resource_mut::<Assets<crate::music::CdTrack>>()
        .add(theme);
    menu.play(world, first_clip);
    world.insert_resource(menu);
    Ok(())
}
pub fn open_garage(world: &mut World, battle_card: bool) {
    world.resource_scope(|world, mut menu: Mut<Contest>| {
        menu.page = if battle_card {
            Page::Battle(0)
        } else {
            Page::Contest
        };
        menu.sync_theme(world);
        world
            .resource_mut::<Assets<Image>>()
            .get_mut(&menu.image)
            .unwrap()
            .data = Some(menu.frame());
    });
}
fn menu_delta(world: &World) -> Duration {
    if world
        .get_resource::<crate::smoke::Capture>()
        .is_some_and(|c| c.demo && !c.realtime)
    {
        Duration::from_millis(50)
    } else {
        world.resource::<Time<Real>>().delta()
    }
}
pub fn update(world: &mut World) {
    let Ok(window) = world.query::<&Window>().single(world) else {
        return;
    };
    let scripted = world
        .get_resource::<crate::smoke::Capture>()
        .is_some_and(|c| c.demo && !c.realtime);
    let focused = window.focused || scripted;
    let h = window.height().min(window.width() * 0.75);
    if let Ok(mut sprite) = world
        .query_filtered::<&mut Sprite, With<ContestScreen>>()
        .single_mut(world)
    {
        sprite.custom_size = Some(Vec2::new(h * 4. / 3., h));
    }
    let menu = world.resource::<Contest>();
    let voices = menu
        .garage
        .voices()
        .chain(menu.theme_entity)
        .chain(menu.cinema.as_ref().and_then(|m| m.voice));
    for entity in voices {
        if let Some(sink) = world.get::<AudioSink>(entity) {
            use bevy::audio::AudioSinkPlayback;
            if focused && sink.is_paused() {
                sink.play();
            } else if !focused && !sink.is_paused() {
                sink.pause();
            }
        }
    }
    if !focused {
        return;
    }
    let mut start = false;
    world.resource_scope(|world, mut menu: Mut<Contest>| {
        let no_input = ButtonInput::default();
        let keys = if scripted {
            &no_input
        } else {
            world.resource::<ButtonInput<KeyCode>>()
        };
        if menu.cinema.is_some() {
            let select = keys.any_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::KeyP])
                || !scripted && world
                    .query::<(Entity, &Gamepad)>()
                    .iter(world)
                    .min_by_key(|(id, _)| *id)
                    .is_some_and(|(_, p)| p.any_pressed([GamepadButton::South, GamepadButton::Start, GamepadButton::Select]));
            let dt = menu_delta(world);
            let handle = menu.image.clone();
            let mut images = world.resource_mut::<Assets<Image>>();
            let movie = menu.cinema.as_mut().unwrap();
            let done = movie
                .advance(dt, select, &mut images, &handle)
                .unwrap_or_else(|err| {
                    warn!("Disc cinema ended early: {err:#}");
                    CinemaStep::Finished
                });
            if done != CinemaStep::Playing {
                let next = movie.clip.next();
                if done == CinemaStep::Skipped {
                    menu.play_button(world, menu.button_sounds[1].clone());
                }
                menu.play(world, next);
            }
            return;
        }
        if !menu.released {
            menu.released = keys.get_pressed().next().is_none();
            return;
        }
        let held = keys.get_pressed().next().is_some();
        let select = keys.any_pressed([KeyCode::Enter, KeyCode::Space]);
        let up = keys.pressed(KeyCode::ArrowUp);
        let down = keys.pressed(KeyCode::ArrowDown);
        if matches!(menu.page, Page::StaffRoll | Page::History | Page::Showcase(_)) || menu.page == Page::Title && !held {
            if menu.update_attract(world, select, up, down) { return; }
        } else if menu.page == Page::Title {
            menu.idle = Duration::ZERO;
        }
        let preview_confirmed = if matches!(menu.page, Page::BossPreview(_)) {
            let mut pads = world.query::<(Entity, &Gamepad)>();
            let first = pads.iter(world).min_by_key(|(id, _)| *id);
            if menu.password_wait_release {
                // screenBossCar polls GetPadStatus(0) before ctlpadAnySelect(0).
                let second = pads.iter(world).filter(|(id, _)| first.is_some_and(|(first, _)| *id != first)).min_by_key(|(id, _)| *id);
                if !scripted && second.is_some_and(|(_, p)| p.get_pressed().next().is_some()) {
                    return;
                }
                menu.password_wait_release = false;
            }
            !scripted && crate::driving::notice_selected(world.resource::<ButtonInput<KeyCode>>(), first.map(|(_, pad)| pad))
        } else {
            false
        };
        let keys = if scripted { &no_input } else { world.resource::<ButtonInput<KeyCode>>() };
        let duel_slot = match menu.page {
            Page::DuelControls(slot) => Some(slot),
            _ => None,
        };
        let controls = (menu.page == Page::Controls || duel_slot.is_some())
            .then(|| menu.controls_view.advance(menu_delta(world), keys, duel_slot));
        let joining = matches!(menu.page, Page::DuelJoin(_));
        let join_held = joining && (crate::pause::duel_pad_held(keys, 0) || crate::pause::duel_pad_held(keys, 1));
        let join_wait = join_held || matches!(menu.page, Page::DuelJoin(Some(_)));
        let reset_idle = match menu.page {
            Page::DuelJoin(_) => join_held,
            Page::Contest => keys.any_pressed([KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft,
                KeyCode::ArrowRight, KeyCode::Enter, KeyCode::KeyD, KeyCode::KeyP, KeyCode::Escape]),
            _ => controls.map_or(held, |input| input.reset_idle),
        };
        let timed_out = menu.tick_setup_timeout(reset_idle, menu_delta(world));
        let preview_finished = if let Page::BossPreview(remaining) = menu.page {
            let left = remaining.saturating_sub(menu_delta(world));
            if !preview_confirmed && !left.is_zero() {
                menu.page = Page::BossPreview(left);
                return;
            }
            true
        } else {
            false
        };
        let animate = if !join_wait && matches!(menu.page, Page::Contest | Page::DuelJoin(None) | Page::DuelGarage{bio:false,..}) {
            let fields=menu.presentation_fields(menu_delta(world));
            menu.garage.tick(fields);
            fields != 0
        } else if let Some(input) = controls {
            input.redraw || input.action.is_some()
        } else if menu.page==Page::Password {
            let mut fields = menu.presentation_fields(menu_delta(world));
            if fields > 0 && menu.cursor_redraw_pending {
                // Redrawing the password consumes a VSync without polling the cursor.
                fields -= 1;
                menu.cursor_redraw_pending = false;
            }
            let visible = menu.cursor_fields >= 20 && !menu.cursor_erased;
            if fields >= 20 - menu.cursor_fields % 20 {
                menu.cursor_erased = false;
            }
            menu.cursor_fields = (menu.cursor_fields + fields % 40) % 40;
            visible != (menu.cursor_fields >= 20 && !menu.cursor_erased)
        } else {
            false
        };
        if !join_wait && !timed_out && !animate && !preview_finished && keys.get_just_pressed().next().is_none() {
            if menu.page == Page::Password && menu.password_wait_release {
                menu.password_wait_release = crate::pause::campaign_pad_held(keys, None);
            }
            return;
        }
        let previous = menu.page;
        let (start_battle, voice) = if let Some(input) = controls {
            let old = menu.duel_layouts()[duel_slot.unwrap_or(0)];
            let mut settings = menu.settings;
            settings.layout = old;
            let sound = input.action.and_then(|action| {
                if crate::pause::navigate(crate::pause::Page::Controls, action, &mut settings).is_none() {
                    menu.page = if duel_slot.is_some() { Page::DuelOptions(0) } else { Page::Options(1) };
                }
                if duel_slot == Some(1) {
                    menu.second_layout = settings.layout;
                } else {
                    menu.settings.layout = settings.layout;
                }
                crate::pause::navigation_sound(crate::pause::Page::Controls, action, old != settings.layout)
            });
            (false, sound.map(|select| menu.button_sounds[usize::from(select)].clone()))
        } else if timed_out {
            (false, (joining && timed_out).then(|| menu.button_sounds[1].clone()))
        } else if preview_finished {
            menu.page = menu.after_password();
            (false, Some(menu.button_sounds[2].clone()))
        } else if menu.page == Page::Password {
            let sound = menu.password_input(keys);
            (false, sound.map(|index| menu.button_sounds[index].clone()))
        } else {
            let start = menu.input(keys);
            (start, menu.input_sound(previous, keys))
        };
        if preview_confirmed {
            menu.play_button(world, menu.button_sounds[1].clone());
        }
        if menu.player_cars != *world.resource::<PlayerCars>() {
            world.insert_resource(menu.player_cars);
        }
        if previous != menu.page && matches!(menu.page, Page::DuelControls(_) | Page::Controls) {
            menu.controls_view = Default::default();
        }
        if let Some(voice) = voice {
            menu.play_button(world, voice);
        }
        if start_battle {
            start = true;
        } else if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&menu.image) {
            img.data = Some(menu.frame());
        }
        menu.sync_theme(world);
        if let Ok(mut window) = world.query::<&mut Window>().single_mut(world) {
            if menu.page == Page::Contest {
                let car = Vehicle::CARD_ORDER[menu.vehicle].name();
                let level = menu.level.name();
                let diff = menu.difficulty;
                let title = format!("TM1 • {car} • {level} • {diff:?}   |   ← → Car   ↑ ↓ Menu   Enter Select   D Difficulty   P Password");
                if window.title != title {
                    window.title = title;
                }
            } else if matches!(menu.page, Page::DuelJoin(_)) {
                window.title = "TM1 • First pick   |   Player 1: Space   Player 2: Comma".into();
            } else if let Page::DuelGarage {slot, ..} = menu.page {
                window.title = if slot == 0 {
                    "TM1 • Player 1   |   A/D Car   W/S Menu   Space Select"
                } else {
                    "TM1 • Player 2   |   ← → Car   ↑ ↓ Menu   Comma Select"
                }.into();
            } else if menu.page==Page::Password {
                window.title="TM1 • Password   |   1 Triangle   2 Square   3 Cross   4 Circle   Space Blank   Enter Accept".into();
            } else if window.title != "TM1" {
                window.title = "TM1".into();
            }
        }
    });
    if start {
        if let Err(err) = start_battle(world) {
            error!("Loading encounter introduction failed: {err:#}");
            world.write_message(AppExit::error());
        }
    }
}
pub fn start_battle(world: &mut World) -> Result<()> {
    let menu = world.resource::<Contest>();
    if matches!(menu.page, Page::Duel(_)) {
        return crate::duel::start(
            world,
            crate::duel::STAGES[menu.duel_stage],
            menu.duel_cars.map(|i| Vehicle::CARD_ORDER[i]),
            menu.difficulty,
            [0; 2],
        );
    }
    let settings = menu.settings;
    let second_layout = menu.second_layout;
    let mut stage = Stage::from_world(
        world,
        menu.level,
        Vehicle::CARD_ORDER[menu.vehicle],
        campaign::Entry::New(menu.difficulty),
    )?;
    stage.apply_passwords(menu.modes);
    crate::transition::begin(world, stage, false, true, Some(settings))?;
    world.insert_resource(SecondPlayerLayout(second_layout));
    world.remove_resource::<Contest>();
    Ok(())
}

#[cfg(test)]
pub(crate) fn menu_world() -> World {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    world.init_resource::<Assets<crate::texture_blend::TextureBlend>>();
    world.init_resource::<Assets<AudioSource>>();
    world.init_resource::<Assets<crate::music::CdTrack>>();
    world.insert_resource(Campaign(disc));
    world.init_resource::<ButtonInput<KeyCode>>();
    world.spawn(Window {
        focused: true,
        ..default()
    });
    show(
        &mut world,
        Vehicle::YellowJacket,
        Level::Arena,
        Difficulty::Medium,
        None,
    )
    .unwrap();
    update(&mut world);
    world.init_resource::<Time<Real>>();
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(world: &mut World, key: KeyCode) {
        let mut keys = ButtonInput::default();
        keys.press(key);
        world.insert_resource(keys);
        update(world);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        if world.get_resource::<Contest>().is_some_and(|menu| {
            matches!(
                menu.page,
                Page::DuelJoin(Some(_)) | Page::Controls | Page::Password | Page::BossPreview(_)
            )
        }) {
            update(world);
        }
        if world.contains_resource::<crate::transition::Transition>() {
            crate::transition::complete(world);
        }
    }
    fn assert_menu_engine(world: &World, car: Vehicle) -> Vec<Entity> {
        use crate::sfx::{EngineSample, pcm::StereoPcm};
        use bevy::audio::{Decodable, Source};
        let voices: Vec<_> = world.resource::<Contest>().garage.voices().collect();
        let bank = world
            .resource::<Campaign>()
            .0
            .read("SND/UACORE.VAB")
            .unwrap();
        let expected = EngineSample::load(&bank, car).unwrap();
        let engines: Vec<_> = voices
            .iter()
            .filter_map(|&entity| world.get::<AudioPlayer<StereoPcm>>(entity))
            .filter(|p| {
                world
                    .resource::<Assets<StereoPcm>>()
                    .get(&p.0)
                    .unwrap()
                    .decoder()
                    .total_duration()
                    .is_none()
            })
            .collect();
        assert!(!engines.is_empty());
        assert_eq!(engines.len(), expected.len());
        for (player, tone) in engines.into_iter().zip(expected) {
            let clip = world
                .resource::<Assets<StereoPcm>>()
                .get(&player.0)
                .unwrap();
            assert!(
                clip.decoder().take(4096).eq(tone
                    .voice(30)
                    .with_levels(crate::sfx::pcm::Levels::new(
                        [i32::from(world.resource::<Contest>().settings.effects_volume()); 2],
                        99
                    ))
                    .decoder()
                    .take(4096))
            );
        }
        voices
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_join_waits_for_both_players_to_release() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, ArrowDown);
        press(&mut world, Enter);
        let voices = assert_menu_engine(&world, Vehicle::CrimsonFury);
        let mut keys = ButtonInput::default();
        keys.press(KeyP);
        keys.press(NumpadEnter);
        world.insert_resource(keys);
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::DuelJoin(Some(1)));
        let field = world.resource::<Contest>().garage.field;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(NumpadEnter);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(21));
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::DuelJoin(Some(1)));
        assert_eq!(world.resource::<Contest>().garage.field, field);
        assert!(
            world
                .resource::<Contest>()
                .garage
                .voices()
                .eq(voices.iter().copied())
        );
        let mut keys = ButtonInput::default();
        keys.press(KeyQ);
        world.insert_resource(keys);
        update(&mut world);
        assert!(
            voices
                .iter()
                .all(|&entity| world.get_entity(entity).is_err())
        );
        assert!(matches!(
            world.resource::<Contest>().page,
            Page::DuelGarage {
                slot: 1,
                second: false,
                ..
            }
        ));
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_join_expires_after_twenty_seconds_with_an_unmapped_key_held() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, ArrowDown);
        press(&mut world, Enter);
        world.resource_mut::<ButtonInput<KeyCode>>().press(KeyQ);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(19999));
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::DuelJoin(None));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(1));
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::Title);
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn keyboard_duel_from_title() {
        use KeyCode::*;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = menu_world();
        for key in [
            ArrowDown, Enter, Space, KeyD, Space, ArrowRight, Comma, ArrowDown, Enter, ArrowDown,
            ArrowRight, Enter, Escape, ArrowUp, Enter,
        ] {
            press(&mut world, key);
        }
        assert!(!world.contains_resource::<Contest>());
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        let duel = world.resource::<crate::duel::Duel>();
        assert_eq!(duel.cars[0].level(), Level::Warehouse);
        let starts = duel
            .cars
            .each_ref()
            .map(tm1_core::drive::PlayerVehicle::position);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        world.resource_mut::<ButtonInput<KeyCode>>().press(KeyW);
        for _ in 0..24 {
            world.run_system_once(crate::duel::advance).unwrap();
        }
        let duel = world.resource::<crate::duel::Duel>();
        assert_ne!(duel.cars[0].position()[..2], starts[0][..2]);
        let second_start = duel.cars[1].position();
        world.resource_mut::<ButtonInput<KeyCode>>().press(ArrowUp);
        for _ in 0..24 {
            world.run_system_once(crate::duel::advance).unwrap();
        }
        assert_ne!(
            world.resource::<crate::duel::Duel>().cars[1].position()[..2],
            second_start[..2]
        );
        world.resource_mut::<ButtonInput<KeyCode>>().press(Escape);
        crate::duel::round_menu(&mut world);
        assert!(world.resource::<Contest>().page == Page::Title);
        assert!(!world.contains_resource::<crate::duel::Duel>());
    }
    fn hidden_pick(world: &mut World, slot: usize) {
        use KeyCode::*;
        let mut keys = ButtonInput::default();
        let chord = if slot == 0 {
            [KeyA, KeyD]
        } else {
            [ArrowLeft, ArrowRight]
        };
        for key in chord {
            keys.press(key);
        }
        let before = world.resource::<Contest>().page;
        let displayed = world.resource::<Contest>().duel_cars[slot];
        world.insert_resource(keys);
        update(world);
        let menu = world.resource::<Contest>();
        assert_eq!(
            menu.page,
            Page::DuelGarage {
                slot: 1 - slot,
                second: true,
                bio: false
            }
        );
        assert!(
            menu.input_sound(before, world.resource::<ButtonInput<KeyCode>>())
                .is_none()
        );
        assert_eq!(
            menu.frame(),
            menu.garage
                .duel_frame(&menu.fonts, displayed, 1 - slot, None)
        );
        world.insert_resource(ButtonInput::<KeyCode>::default());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn hidden_duel_pick_uses_the_retained_slot_across_battles() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [ArrowDown, Enter, Space] {
            press(&mut world, key);
        }
        hidden_pick(&mut world, 0);
        press(&mut world, Comma);
        assert_menu_engine(&world, Vehicle::Minion);
        press(&mut world, Enter);
        let kinds = |world: &World| {
            world
                .resource::<crate::duel::Duel>()
                .cars
                .each_ref()
                .map(|car| car.vehicle())
        };
        assert_eq!(kinds(&world), [Vehicle::YellowJacket, Vehicle::SweetTooth]);

        world.resource_mut::<ButtonInput<KeyCode>>().press(Escape);
        crate::duel::round_menu(&mut world);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        update(&mut world);
        for key in [ArrowDown, Enter, Comma, ArrowRight] {
            press(&mut world, key);
        }
        hidden_pick(&mut world, 1);
        press(&mut world, Space);
        assert_menu_engine(&world, Vehicle::YellowJacket);
        press(&mut world, Enter);
        assert_eq!(kinds(&world), [Vehicle::YellowJacket, Vehicle::SweetTooth]);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn hidden_duel_pick_with_a_cold_second_slot_returns_to_title() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [ArrowDown, Enter, Comma] {
            press(&mut world, key);
        }
        hidden_pick(&mut world, 1);
        press(&mut world, Space);
        assert_eq!(world.resource::<Contest>().page, Page::Title);
        assert_eq!(
            world.resource::<PlayerCars>().0,
            [Some(Vehicle::SweetTooth.card_index()), None]
        );
        assert!(!world.contains_resource::<crate::duel::Duel>());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn jp_hidden_duel_card_retains_the_previously_registered_vehicle() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let data = disc.read("UADMD/CARS.DMD").unwrap();
        let garage = tm1_assets::scene::Dmd::parse(&data).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        // uaInit through its write to playerCar[0]. Omit unrelated subsystem setup.
        for address in [0x8014_b37c, 0x8014_b3b8, 0x8015_0274] {
            cpu.put(address, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        cpu.call_until(0x8014_b294, &[], 0x8014_b358);
        // DB.c registers these showroom root types with UAAddCs.
        let names: Vec<_> = garage
            .roots
            .iter()
            .map(|&(_, kind)| kind)
            .filter(|&kind| (10..=130).contains(&kind) && kind % 10 == 0)
            .collect();
        assert_eq!(names.len(), 12);
        assert!(!names.contains(&(Vehicle::Minion as u16)));
        cpu.put(0x8018_c336, &(names.len() as u16).to_le_bytes());
        for (i, kind) in names.into_iter().enumerate() {
            cpu.put(0x8019_edd8 + 4 * i as u32, &u32::from(kind).to_le_bytes());
        }
        // A fresh second slot reaches the battle-options picture lookup with ID zero.
        let picture = cpu.call(0x8015_09e8, &[0]);
        assert_eq!(picture, 12);
        assert_eq!(cpu.word(0x8017_0e24 + 4 * picture), 0x00d7_000e);
        let hidden = cpu.call(0x8013_b988, &[12]);
        assert_eq!(hidden, Vehicle::Minion as u32);
        for (slot, cold) in [Vehicle::YellowJacket as u32, 0].into_iter().enumerate() {
            let slot = slot as u32;
            assert_eq!(cpu.call(0x8014_d32c, &[slot]), cold);
            for previous in [cold, Vehicle::RoadKill as u32, Vehicle::Outlaw as u32] {
                cpu.call(0x8014_d2b4, &[slot, previous, 1]);
                cpu.call(0x8014_d2b4, &[slot, hidden, 1]);
                assert_eq!(cpu.call(0x8014_d32c, &[slot]), previous);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_handoff_preserves_the_carousel_and_anchors_the_locked_portrait() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [ArrowDown, Enter, Space, KeyD] {
            press(&mut world, key);
        }
        let mut menu = world.resource_mut::<Contest>();
        menu.garage.tick(9);
        let heading = menu.garage.heading;
        let field = menu.garage.field;
        let slide = menu.garage.slide;
        let mut keys = ButtonInput::default();
        keys.press(Space);
        menu.input(&keys);
        assert_eq!(
            (menu.garage.heading, menu.garage.field, menu.garage.slide),
            (heading, field, slide)
        );
        assert!(matches!(
            menu.page,
            Page::DuelGarage {
                slot: 1,
                second: true,
                bio: false
            }
        ));
        let picks = menu.duel_cars;
        let mut chord = ButtonInput::default();
        chord.press(ArrowLeft);
        chord.press(ArrowRight);
        chord.press(Comma);
        let page = menu.page;
        menu.input(&chord);
        assert_eq!(menu.page, page);
        assert_eq!(menu.duel_cars, picks);
        assert!(menu.input_sound(page, &chord).is_none());
        let disc = &world.resource::<Campaign>().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let dmd = disc.read("UADMD/CARS.DMD").unwrap();
        let textures =
            tm1_assets::texture::Vram::from_tms(&disc.read("UADMD/CARS.TMS").unwrap()).unwrap();
        let portraits = tm1_assets::scene::Dmd::parse(&dmd)
            .unwrap()
            .occupant_portraits(&textures)
            .unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        let mut right = 32;
        let mut bottom = 45;
        for _ in 0..9 {
            let menu = world.resource::<Contest>();
            let i = menu.duel_cars[0];
            let rendered = menu.frame();
            let with_no_portrait = menu
                .garage
                .duel_frame(&menu.fonts, menu.duel_cars[1], 1, None);
            let w = (2 * right) as usize;
            let h = (45 + bottom) as usize;
            for y in 35..125 {
                for x in 10..74 {
                    let at = 4 * (y * 320 + x);
                    let mut expected: [u8; 4] = with_no_portrait[at..at + 4].try_into().unwrap();
                    if (42 - w / 2..42 + w / 2).contains(&x) && y < 35 + h {
                        let src = 4 * (64 * ((y - 35) * 90 / h) + (x + w / 2 - 42) * 64 / w);
                        let pixel = &portraits[i][src..src + 4];
                        if pixel[3] == 128 {
                            for c in 0..3 {
                                expected[c] = expected[c] / 2 + pixel[c] / 2;
                            }
                        } else if pixel[3] != 0 {
                            expected.copy_from_slice(pixel);
                        }
                    }
                    assert_eq!(rendered[at..at + 4], expected);
                }
            }
            right = cpu.call(0x801511d8, &[right, 24, 50]);
            bottom = cpu.call(0x801511d8, &[bottom, 22, 50]);
            world.resource_mut::<Contest>().garage.tick(1);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn second_player_first_pick_and_independent_layouts() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, ArrowDown);
        press(&mut world, Enter);
        let mut keys = ButtonInput::default();
        keys.press(Space);
        keys.press(Comma);
        world.insert_resource(keys);
        update(&mut world);
        press(&mut world, Space);
        press(&mut world, ArrowDown);
        press(&mut world, Comma);
        assert!(matches!(
            world.resource::<Contest>().page,
            Page::DuelGarage {
                slot: 1,
                bio: true,
                ..
            }
        ));
        for key in [Comma, ArrowUp, Comma, KeyA] {
            press(&mut world, key);
        }
        assert_eq!(
            world.resource::<Contest>().duel_cars,
            [Vehicle::CARD_ORDER.len() - 1, 0]
        );
        for key in [KeyD, Space, ArrowDown, Enter] {
            press(&mut world, key);
        }
        let motor = assert_menu_engine(&world, Vehicle::CARD_ORDER[1]);
        // screenGetWhichPad samples all of pad two, including a held non-select button.
        let mut keys = ButtonInput::default();
        keys.press(KeyL);
        keys.press(Space);
        world.insert_resource(keys);
        update(&mut world);
        assert!(world.resource::<Contest>().page == Page::DuelControls(1));
        assert!(world.resource::<Contest>().garage.voices().eq(motor));
        world.insert_resource(ButtonInput::<KeyCode>::default());
        let displayed = |world: &World| {
            let menu = world.resource::<Contest>();
            world
                .resource::<Assets<Image>>()
                .get(&menu.image)
                .unwrap()
                .data
                .clone()
                .unwrap()
        };
        let base_frame = |world: &World, layout| {
            let menu = world.resource::<Contest>();
            let mut settings = menu.settings;
            settings.layout = layout;
            menu.controls
                .frame(&menu.fonts, None, crate::pause::Page::Controls, settings)
        };
        let plain = base_frame(&world, 1);
        let mut labelled = plain.clone();
        world.resource::<Contest>().fonts.centered_at(
            &mut labelled,
            Face::Choice,
            159,
            40,
            "Player 2",
        );
        assert_ne!(plain, labelled);
        let mut blink = 0xf0f0u16;
        for draw in 0..66 {
            if draw > 0 {
                world
                    .resource_mut::<Time<Real>>()
                    .advance_by(Duration::from_nanos(33_333_334));
                update(&mut world);
            }
            let expected = if draw >= 60 || blink & 1 != 0 {
                &labelled
            } else {
                &plain
            };
            assert!(displayed(&world) == *expected, "control screen draw {draw}");
            if blink == 0 {
                blink = 0xf0f0;
            }
            blink >>= 1;
        }
        press(&mut world, ArrowRight);
        let mut expected = base_frame(&world, 2);
        let fonts = &world.resource::<Contest>().fonts;
        fonts.centered_at(&mut expected, Face::Choice, 159, 40, "Player 2");
        let released = expected.clone();
        fonts.at(&mut expected, Face::ArrowActive, 285, 63, ">");
        assert!(displayed(&world) == expected);
        update(&mut world);
        assert!(displayed(&world) == released);
        press(&mut world, Comma);
        press(&mut world, Space);
        update(&mut world);
        press(&mut world, KeyA);
        press(&mut world, Escape);
        let menu = world.resource::<Contest>();
        assert!(menu.page == Page::DuelOptions(0));
        assert_eq!(menu.duel_layouts(), [0, 2]);
        assert_eq!(menu.settings.layout, 0);
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn duel_pause_freezes_both_cars_and_returns_to_title_movie() {
        use crate::{duel, pause::Page as Pause};
        use KeyCode::*;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = menu_world();
        duel::start(
            &mut world,
            Level::Warehouse,
            [Vehicle::Outlaw, Vehicle::RoadKill],
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let step = |world: &mut World, pressed: &[KeyCode]| {
            let mut keys = ButtonInput::default();
            for &key in pressed {
                keys.press(key);
            }
            world.insert_resource(keys);
            world.run_system_once(duel::pause_menu).unwrap();
            world.run_system_once(duel::advance).unwrap();
            world.run_system_once(crate::sfx::duel::mix).unwrap();
            duel::round_menu(world);
        };
        let active_engines = |world: &mut World| {
            world
                .query_filtered::<Entity, (
                    With<crate::sfx::duel::DuelEngine>,
                    With<AudioPlayer<crate::sfx::pcm::StereoPcm>>,
                )>()
                .iter(world)
                .count()
        };
        step(&mut world, &[KeyW, ArrowUp]);
        let engine_count = active_engines(&mut world);
        assert!(engine_count >= 2);
        let positions = |world: &World| {
            world
                .resource::<duel::Duel>()
                .cars
                .each_ref()
                .map(tm1_core::drive::PlayerVehicle::position)
        };
        let start = positions(&world);
        step(&mut world, &[NumpadEnter]);
        for _ in 0..12 {
            step(&mut world, &[KeyT, ShiftRight]);
        }
        assert_eq!(positions(&world), start);
        assert_eq!(active_engines(&mut world), 0);
        step(&mut world, &[ShiftRight, ArrowUp]);
        assert_eq!(
            world.resource::<duel::Duel>().pause,
            Some(Pause::Choices(0))
        );
        step(&mut world, &[]);
        step(&mut world, &[Space, ArrowUp]);
        assert_eq!(
            world.resource::<duel::Duel>().pause,
            Some(Pause::Choices(3))
        );
        step(&mut world, &[]);
        step(&mut world, &[ArrowDown]);
        step(&mut world, &[]);
        step(&mut world, &[Space, Slash]);
        step(&mut world, &[Slash]);
        assert_eq!(
            world.resource::<duel::Duel>().pause,
            Some(Pause::Choices(0))
        );
        assert_eq!(positions(&world), start);
        assert_eq!(active_engines(&mut world), 0);
        step(&mut world, &[]);
        assert!(world.resource::<duel::Duel>().pause.is_none());
        assert_eq!(positions(&world), start);
        step(&mut world, &[KeyP]);
        step(&mut world, &[]);
        step(&mut world, &[ArrowUp]);
        step(&mut world, &[]);

        step(&mut world, &[KeyL, Space]);
        assert!(world.resource::<duel::Duel>().pause == Some(Pause::Controls));
        step(&mut world, &[Comma, ArrowRight]);
        assert_eq!(world.resource::<duel::Duel>().settings.layout, 1);
        step(&mut world, &[Space]);
        step(&mut world, &[Space, ArrowRight]);
        step(&mut world, &[Space, ArrowRight, ArrowLeft, Comma]);
        assert_eq!(world.resource::<duel::Duel>().settings.layout, 2);
        step(&mut world, &[Space]);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(600));
        step(&mut world, &[]);
        assert_eq!(world.resource::<duel::Duel>().pause, Some(Pause::Controls));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(34));
        step(&mut world, &[]);
        assert!(world.resource::<duel::Duel>().pause.is_none());
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        for _ in 0..8 {
            step(&mut world, &[KeyW, ArrowUp]);
        }
        assert_ne!(positions(&world), start);
        assert_eq!(active_engines(&mut world), engine_count);
        for key in [KeyP, KeyS, KeyS, Space, KeyD] {
            step(&mut world, &[key]);
            step(&mut world, &[]);
        }
        for (player, playback, engine) in world
            .query_filtered::<(
                Option<&AudioPlayer<crate::music::CdTrack>>,
                &PlaybackSettings,
                Has<crate::sfx::duel::DuelEngine>,
            ), Or<(With<duel::DuelMusic>, With<crate::sfx::duel::DuelEngine>)>>()
            .iter(&world)
        {
            assert_eq!(playback.paused, engine);
            if !engine {
                use bevy::audio::Decodable;
                let audio = world
                    .resource::<Assets<crate::music::CdTrack>>()
                    .get(&player.unwrap().0)
                    .unwrap();
                let track = world.resource::<Campaign>().0.audio_track(7).unwrap();
                for (sample, bytes) in audio
                    .decoder()
                    .take(4096)
                    .zip(track[150 * 2352..].chunks_exact(2))
                {
                    assert_eq!(
                        sample,
                        f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.
                    );
                }
            }
        }
        assert_eq!(world.resource::<duel::Duel>().settings.music, 8);
        let menu_voices = |world: &mut World| {
            world
                .query_filtered::<Entity, With<duel::PauseVoice>>()
                .iter(world)
                .count()
        };
        let before = menu_voices(&mut world);
        step(&mut world, &[Space]);
        assert_eq!(menu_voices(&mut world), before + 1);
        step(&mut world, &[]);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(300));
        step(&mut world, &[]);
        assert_eq!(world.resource::<duel::Duel>().pause, Some(Pause::Audio(0)));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f64(1. / 60.));
        step(&mut world, &[]);
        assert!(world.resource::<duel::Duel>().pause.is_none());
        assert_eq!(menu_voices(&mut world), before + 1);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        for key in [NumpadEnter, ArrowDown, Comma] {
            step(&mut world, &[key]);
            step(&mut world, &[]);
        }
        assert!(world.resource::<duel::Duel>().pause == Some(Pause::Aborted));
        for _ in 0..10 {
            step(&mut world, &[]);
        }
        assert!(world.contains_resource::<duel::Duel>());
        step(&mut world, &[]);
        assert!(!world.contains_resource::<duel::Duel>());
        assert_eq!(world.resource::<Contest>().duel_layouts(), [1, 2]);
        assert_eq!(world.resource::<Contest>().settings.layout, 1);
        assert!(world.resource::<Contest>().cinema.as_ref().unwrap().clip == Clip::Title);
    }
    fn password(world: &mut World, keys: &[KeyCode]) {
        press(world, KeyCode::KeyP);
        for &key in keys {
            press(world, key);
        }
        press(world, KeyCode::Enter);
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn title_options_scrolls_tracks_and_battle_mix() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [
            ArrowUp, Enter, Enter, ArrowRight, ArrowDown, ArrowDown, ArrowLeft,
        ] {
            press(&mut world, key);
        }
        let menu = world.resource::<Contest>();
        assert!(menu.page == Page::Audio(2));
        assert_eq!(menu.playing_track, 6);
        assert_eq!((menu.settings.music, menu.settings.effects), (8, 6));
        let last_track = menu.theme_entity.unwrap();
        let player = world
            .get::<AudioPlayer<crate::music::CdTrack>>(last_track)
            .unwrap();
        let tracks = world.resource::<Assets<crate::music::CdTrack>>();
        let mut decoder = bevy::audio::Decodable::decoder(tracks.get(&player.0).unwrap());
        let pcm = world.resource::<Campaign>().0.audio_track(8).unwrap();
        for bytes in pcm[150 * 2352..].chunks_exact(2).take(4096) {
            assert_eq!(
                decoder.next(),
                Some(f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.)
            );
        }
        let before = menu.frame();
        press(&mut world, ArrowRight);
        let menu = world.resource::<Contest>();
        assert_eq!(menu.playing_track, 0);
        assert_ne!(before, menu.frame());
        assert!(world.get_entity(last_track).is_err());
        for key in [ArrowDown, Enter, ArrowDown, Enter] {
            press(&mut world, key);
        }
        assert!(world.resource::<Contest>().page == Page::History);
        update(&mut world);
        press(&mut world, Enter);
        assert!(world.resource::<Contest>().page == Page::MainOptions(1));
        update(&mut world);
        for key in [ArrowDown, Enter] {
            press(&mut world, key);
        }
        assert!(world.resource::<Contest>().page == Page::StaffRoll);
        update(&mut world);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(10));
        update(&mut world);
        let menu = world.resource::<Contest>();
        assert_ne!(menu.frame(), menu.credits_background);
        press(&mut world, Enter);
        assert!(world.resource::<Contest>().page == Page::MainOptions(2));
        update(&mut world);
        for key in [Escape, ArrowDown, Enter, Enter, Enter] {
            press(&mut world, key);
        }
        let d = world.resource::<crate::driving::Drive>();
        assert_eq!((d.settings.music, d.settings.effects), (8, 6));
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn solo_controls_draw_the_held_arrow_and_wait_for_full_release() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [Enter, Enter, ArrowDown, Enter, ArrowDown] {
            press(&mut world, key);
        }
        let step = |world: &mut World, pressed: &[KeyCode]| {
            let mut keys = ButtonInput::default();
            for &key in pressed {
                keys.press(key);
            }
            world.insert_resource(keys);
            update(world);
        };
        let check = |world: &World, layout, highlighted| {
            let menu = world.resource::<Contest>();
            assert_eq!(menu.page, Page::Controls);
            assert_eq!(menu.settings.layout, layout);
            let mut expected = menu.controls.frame(
                &menu.fonts,
                None,
                crate::pause::Page::Controls,
                menu.settings,
            );
            if highlighted {
                menu.fonts
                    .at(&mut expected, Face::ArrowActive, 285, 63, ">");
            }
            let displayed = &world
                .resource::<Assets<Image>>()
                .get(&menu.image)
                .unwrap()
                .data;
            assert!(displayed.as_deref() == Some(expected.as_slice()));
        };
        step(&mut world, &[Enter]);
        step(&mut world, &[Enter, ArrowRight]);
        check(&world, 1, false);
        step(&mut world, &[]);
        step(&mut world, &[ArrowRight]);
        check(&world, 2, true);
        step(&mut world, &[KeyW, Enter]);
        check(&world, 2, true);
        step(&mut world, &[]);
        check(&world, 2, false);
        step(&mut world, &[Space, ArrowRight]);
        assert_eq!(world.resource::<Contest>().page, Page::Options(1));
        assert_eq!(world.resource::<Contest>().settings.layout, 2);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_setup_applies_keyboard_choices_to_the_encounter() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [
            Enter, Enter, ArrowDown, Enter, ArrowLeft, ArrowDown, Enter, ArrowRight, Enter,
            ArrowDown, Enter, Enter,
        ] {
            press(&mut world, key);
        }
        assert!(world.resource::<Contest>().page == Page::Wrong(0));
        for key in [ArrowDown, Enter] {
            press(&mut world, key);
        }
        assert!(world.resource::<Contest>().page == Page::Options(2));
        for key in [
            Enter, Digit2, Digit1, Digit4, Digit2, Digit2, Enter, ArrowDown, Enter,
        ] {
            press(&mut world, key);
        }
        assert!(world.resource::<Contest>().page == Page::Battle(0));
        press(&mut world, Enter);
        let drive = world.resource::<crate::driving::Drive>();
        assert_eq!(drive.player.level(), Level::Arena5);
        assert_eq!(drive.player.difficulty(), Difficulty::Easy);
        assert_eq!(drive.settings.layout, 2);
        assert_eq!(drive.player.state()[0xc8 + 22], 2);
        open(&mut world).unwrap();
        assert_eq!(world.resource::<Contest>().duel_layouts(), [2, 1]);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_setup_idle_deadlines() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [Enter, Enter, ArrowDown, Enter] {
            press(&mut world, key);
        }
        let mut elapsed = |ms| {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(ms));
            update(&mut world);
            match world.resource::<Contest>().page {
                Page::Battle(_) => 1,
                Page::Options(_) => 2,
                Page::Title => 3,
                _ => 0,
            }
        };
        assert_eq!(elapsed(300_000), 2);
        assert_eq!(elapsed(17), 1);
        assert_eq!(elapsed(20_000), 1);
        assert_eq!(elapsed(17), 3);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE and FFmpeg"]
    fn history_showcase_and_attract_rotation() {
        let mut world = menu_world();
        world.insert_resource(AttractCycle(1));
        let tick = |world: &mut World, millis| {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(millis));
            update(world);
        };
        tick(&mut world, 20_017);
        assert!(world.resource::<Contest>().page == Page::History);
        assert_eq!(world.resource::<Contest>().history.lines.len(), 31);
        let theme = world.resource::<Contest>().theme_entity;
        tick(&mut world, 10_000);
        let menu = world.resource::<Contest>();
        assert!(menu.frame() != menu.history_background);
        tick(&mut world, 80_000);
        assert!(world.resource::<Contest>().page == Page::Showcase(0));
        assert_eq!(world.resource::<Contest>().theme_entity, theme);
        for car in 1..12 {
            tick(&mut world, 6017);
            assert!(world.resource::<Contest>().page == Page::Showcase(car));
        }
        tick(&mut world, 6017);
        assert!(matches!(
            world.resource::<Contest>().cinema.as_ref().map(|c| c.clip),
            Some(Clip::Title)
        ));
        tick(&mut world, 0);
        tick(&mut world, 4500);
        tick(&mut world, 0);
        tick(&mut world, 20_017);
        assert!(world.resource::<Contest>().page == Page::Showcase(0));
        press(&mut world, KeyCode::Enter);
        assert!(world.resource::<Contest>().page == Page::Title);
        assert_eq!(world.resource::<AttractCycle>().0, 0);
        assert_eq!(world.resource::<Contest>().vehicle, 1);
        tick(&mut world, 0);
        tick(&mut world, 4500);
        tick(&mut world, 0);
        world.insert_resource(AttractCycle(1));
        tick(&mut world, 20_017);
        press(&mut world, KeyCode::Enter);
        assert!(world.resource::<Contest>().page == Page::Title);
        assert!(world.resource::<Contest>().cinema.is_some());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_setup_retains_showroom_audio_and_direct_entry_only_starts_the_motor() {
        use KeyCode::*;
        let mut world = menu_world();
        open_garage(&mut world, true);
        let voices = assert_menu_engine(&world, Vehicle::YellowJacket);
        let bank = world
            .resource::<Campaign>()
            .0
            .read("SND/UACORE.VAB")
            .unwrap();
        let tone_count = |prog| tm1_assets::sound::program(&bank, prog).unwrap().len();
        assert_eq!(
            voices.len(),
            tone_count(Vehicle::YellowJacket.sound_programs().0)
        );

        open_garage(&mut world, false);
        crate::pause::navigate(
            crate::pause::Page::Audio(1),
            crate::pause::Action::Right,
            &mut world.resource_mut::<Contest>().settings,
        );
        press(&mut world, ArrowRight);
        let menu = world.resource::<Contest>();
        let vehicle = Vehicle::CARD_ORDER[menu.vehicle];
        let volume = menu.settings.effects_volume();
        assert_ne!(volume, 33);
        let showroom = assert_menu_engine(&world, vehicle);
        let (motor, signature) = vehicle.sound_programs();
        let signatures = &showroom[tone_count(motor)..];
        let expected = tm1_assets::sound::program(&bank, signature).unwrap();
        assert_eq!(signatures.len(), expected.len());
        for (&entity, tone) in signatures.iter().zip(expected) {
            use crate::sfx::pcm::{Levels, StereoPcm};
            use bevy::audio::{Decodable, Volume};
            let player = world.get::<AudioPlayer<StereoPcm>>(entity).unwrap();
            let actual = world
                .resource::<Assets<StereoPcm>>()
                .get(&player.0)
                .unwrap();
            let clip =
                StereoPcm::one_shot(&tone).with_levels(Levels::new([i32::from(volume); 2], 99));
            assert_eq!(
                world.get::<PlaybackSettings>(entity).unwrap().volume,
                Volume::Linear(0.44)
            );
            assert!(actual.decoder().eq(clip.decoder()));
        }
        for key in [ArrowDown, Enter, Enter, ArrowUp, Enter] {
            press(&mut world, key);
            assert!(
                world
                    .resource::<Contest>()
                    .garage
                    .voices()
                    .eq(showroom.iter().copied())
            );
        }
        assert_eq!(world.resource::<Contest>().page, Page::Battle(0));
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn solo_showroom_input_priority_and_idle_exit() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, Enter);
        let car = world.resource::<Contest>().vehicle;
        let mut keys = ButtonInput::default();
        for key in [ArrowLeft, ArrowRight, Enter] {
            keys.press(key);
        }
        world.insert_resource(keys);
        update(&mut world);
        let menu = world.resource::<Contest>();
        assert_eq!((menu.page, menu.vehicle), (Page::Contest, car));
        assert!(
            menu.input_sound(Page::Contest, world.resource::<ButtonInput<KeyCode>>())
                .is_none()
        );
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(ArrowDown);
        update(&mut world);
        assert_eq!(world.resource::<Contest>().garage.row, 1);
        let mut keys = ButtonInput::default();
        keys.press(KeyQ);
        world.insert_resource(keys);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(300));
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::Contest);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_nanos(16_666_667));
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::Title);
        assert_eq!(world.resource::<Contest>().garage.voices().count(), 0);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn garage_bios_and_card_order() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, Enter);
        press(&mut world, ArrowLeft);
        press(&mut world, ArrowDown);
        for (i, vehicle) in Vehicle::CARD_ORDER.into_iter().enumerate() {
            press(&mut world, Enter);
            let menu = world.resource::<Contest>();
            assert!(menu.page == Page::Bio);
            assert_eq!(Vehicle::CARD_ORDER[menu.vehicle], vehicle);
            let expected = screen::background(
                &world
                    .resource::<Campaign>()
                    .0
                    .read(&format!("UABIOS/SCNDB{:02}B.TIM", i + 1))
                    .unwrap(),
            )
            .unwrap();
            assert!(
                world
                    .resource::<Assets<Image>>()
                    .get(&menu.image)
                    .unwrap()
                    .data
                    .as_deref()
                    == Some(expected.as_slice())
            );
            press(&mut world, Enter);
            press(&mut world, ArrowRight);
        }
        assert_eq!(world.resource::<Contest>().vehicle, 0);
        press(&mut world, ArrowDown);
        press(&mut world, ArrowLeft);
        assert_eq!(world.resource::<Contest>().vehicle, 0);
        press(&mut world, Enter);
        assert!(world.resource::<Contest>().page == Page::Title);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE and FFmpeg"]
    fn opening_movies_attract_and_title_theme() {
        let mut world = menu_world();
        world
            .resource_scope(|world, mut menu: Mut<Contest>| menu.play(world, Some(Clip::Presents)));
        let tick = |world: &mut World, ms| {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(ms));
            update(world);
        };
        let clip = |world: &World| world.resource::<Contest>().cinema.as_ref().map(|m| m.clip);
        for (card, ms) in [
            (Clip::Presents, 2017),
            (Clip::Production, 2334),
            (Clip::Developer, 2334),
        ] {
            assert!(clip(&world) == Some(card));
            tick(&mut world, 0);
            tick(&mut world, ms);
        }
        assert!(clip(&world) == Some(Clip::Logo));
        tick(&mut world, 0);
        let confirmations = |world: &mut World| {
            let sound = world.resource::<Contest>().button_sounds[1].clone();
            world
                .query::<&AudioPlayer>()
                .iter(world)
                .filter(|p| p.0 == sound)
                .count()
        };
        let mut pads = [
            world.spawn(Gamepad::default()).id(),
            world.spawn(Gamepad::default()).id(),
        ];
        pads.sort();
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .press(GamepadButton::South);
        world
            .get_mut::<Gamepad>(pads[0])
            .unwrap()
            .digital_mut()
            .press(GamepadButton::Select);
        tick(&mut world, 1933);
        assert!(clip(&world) == Some(Clip::Logo));
        assert_eq!(confirmations(&mut world), 0);
        world
            .get_mut::<Gamepad>(pads[0])
            .unwrap()
            .digital_mut()
            .release(GamepadButton::Select);
        tick(&mut world, 67);
        assert!(clip(&world) == Some(Clip::Logo));
        world
            .get_mut::<Gamepad>(pads[0])
            .unwrap()
            .digital_mut()
            .press(GamepadButton::Select);
        tick(&mut world, 10);
        assert!(clip(&world) == Some(Clip::Logo));
        tick(&mut world, 57);
        assert!(clip(&world) == Some(Clip::Title));
        assert_eq!(confirmations(&mut world), 1);
        world
            .get_mut::<Gamepad>(pads[0])
            .unwrap()
            .digital_mut()
            .release(GamepadButton::Select);
        let handle = world.resource::<Contest>().image.clone();
        let original = world
            .resource::<Assets<Image>>()
            .get(&handle)
            .unwrap()
            .data
            .clone();
        tick(&mut world, 0);
        tick(&mut world, 2000);
        assert_ne!(
            original.as_deref(),
            world
                .resource::<Assets<Image>>()
                .get(&handle)
                .unwrap()
                .data
                .as_deref()
        );
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        tick(&mut world, 2334); // Last displayed frame has no input callback.
        assert!(clip(&world) == Some(Clip::Title));
        tick(&mut world, 166);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        assert!(clip(&world).is_none());
        assert_eq!(confirmations(&mut world), 1);
        assert!(world.resource::<Contest>().theme_entity.is_some());
        tick(&mut world, 0);
        tick(&mut world, 20017);
        assert!(clip(&world) == Some(Clip::Attract));
        assert!(world.resource::<Contest>().theme_entity.is_none());
        tick(&mut world, 0);
        press(&mut world, KeyCode::KeyP);
        assert!(clip(&world) == Some(Clip::Title));
        assert_eq!(confirmations(&mut world), 2);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn password_edits_wait_for_release_and_redraw_the_blinking_cursor() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, Enter);
        let step = |world: &mut World, keys: &[KeyCode], nanos| {
            let mut input = ButtonInput::default();
            for &key in keys {
                input.press(key);
            }
            world.insert_resource(input);
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_nanos(nanos));
            update(world);
        };
        step(&mut world, &[KeyP], 0);
        step(&mut world, &[KeyP, Digit1], 0);
        step(&mut world, &[], 0);
        assert_eq!(world.resource::<Contest>().slots, *b"     ");
        let frame = |world: &World| {
            let menu = world.resource::<Contest>();
            world
                .resource::<Assets<Image>>()
                .get(&menu.image)
                .unwrap()
                .data
                .clone()
                .unwrap()
        };
        let plain = frame(&world);
        step(&mut world, &[], 333_333_334);
        let mut cursor = plain;
        let menu = world.resource::<Contest>();
        menu.fonts.at(&mut cursor, Face::Password, 108, 183, "_");
        assert!(frame(&world) == cursor);
        step(&mut world, &[Enter, ArrowLeft, Digit3, Digit4], 0);
        let menu = world.resource::<Contest>();
        assert_eq!((menu.slots, menu.cursor), (*b"@    ", 1));
        let plain = frame(&world);
        step(&mut world, &[Digit4, Digit1, ArrowRight], 0);
        step(&mut world, &[ArrowRight], 0);
        assert_eq!(world.resource::<Contest>().cursor, 1);
        step(&mut world, &[], 666_666_667);
        assert!(frame(&world) == plain);
        step(&mut world, &[], 16_666_667);
        let menu = world.resource::<Contest>();
        let mut cursor = plain;
        menu.fonts.at(
            &mut cursor,
            Face::Password,
            108 + menu.fonts.password_stride(),
            183,
            "_",
        );
        assert!(frame(&world) == cursor);
        step(&mut world, &[ArrowLeft, Enter], 0);
        assert_eq!(world.resource::<Contest>().cursor, 0);
        step(&mut world, &[], 0);
        step(&mut world, &[ArrowLeft], 0);
        assert_eq!(world.resource::<Contest>().cursor, 4);
        step(&mut world, &[], 0);
        step(&mut world, &[KeyQ], 3_600_000_000_000);
        assert_eq!(world.resource::<Contest>().page, Page::Password);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn boss_password_waits_for_release_before_timing_and_playing_the_reward() {
        use KeyCode::*;
        let mut world = menu_world();
        for key in [Enter, KeyP, Digit1, Digit3, Digit1, Digit1, Digit4] {
            press(&mut world, key);
        }
        let reward = world.resource::<Contest>().button_sounds[2].clone();
        let rewards = |world: &mut World| {
            world
                .query::<&AudioPlayer>()
                .iter(world)
                .filter(|p| p.0 == reward)
                .count()
        };
        let mut pads = [
            world.spawn(Gamepad::default()).id(),
            world.spawn(Gamepad::default()).id(),
        ];
        pads.sort();
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .press(GamepadButton::DPadUp);
        world.resource_mut::<ButtonInput<KeyCode>>().press(Enter);
        update(&mut world);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(6));
        update(&mut world);
        assert_eq!(
            world.resource::<Contest>().page,
            Page::BossPreview(Duration::from_secs(5))
        );
        assert_eq!(rewards(&mut world), 0);
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .reset_all();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::ZERO);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        update(&mut world);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(4));
        update(&mut world);
        assert_eq!(
            world.resource::<Contest>().page,
            Page::BossPreview(Duration::from_secs(1))
        );
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::ZERO);
        press(&mut world, Space);
        assert_eq!(world.resource::<Contest>().page, Page::Contest);
        assert_eq!(rewards(&mut world), 1);
        for key in [KeyP, Digit1, Digit3, Digit1, Digit1, Digit4] {
            press(&mut world, key);
        }
        world.resource_mut::<ButtonInput<KeyCode>>().press(Enter);
        update(&mut world);
        assert!(matches!(
            world.resource::<Contest>().page,
            Page::BossPreview(_)
        ));
        // The upper halfword is clear, so a held primary confirm already dismisses it.
        update(&mut world);
        assert_eq!(world.resource::<Contest>().page, Page::Contest);
        assert_eq!(rewards(&mut world), 2);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn special_passwords_and_boss_preview() {
        use KeyCode::*;
        let mut world = menu_world();
        press(&mut world, Enter);
        password(&mut world, &[Digit1, Digit3, Digit1, Digit1, Digit4]);
        assert!(matches!(
            world.resource::<Contest>().page,
            Page::BossPreview(_)
        ));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_secs(5));
        update(&mut world);
        assert!(matches!(world.resource::<Contest>().page, Page::Contest));
        password(&mut world, &[Digit2, Digit1, Digit3, Space, Digit4]);
        password(&mut world, &[Digit1, Space, Digit2, Digit4, Digit4]);
        password(&mut world, &[Digit4, Digit4, Digit1, Digit3, Space]);
        password(&mut world, &[Digit1, Digit3, Digit4, Digit2, Digit1]);
        press(&mut world, Enter);
        press(&mut world, Enter);
        let drive = world.resource::<crate::driving::Drive>();
        assert_eq!(drive.player.level(), Level::Rooftops);
        assert!(drive.showing_boss_intro());
        assert_eq!(drive.player.state()[0x184], 0);
        assert_eq!(drive.player.state()[0x4a1], 1);
        assert!(drive.opponents.iter().all(|ai| ai.health().0 > 0));
        use bevy::ecs::system::RunSystemOnce;
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        // Resume driving at a battle frame interval after the five-second menu preview.
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(50));
        world.run_system_once(crate::driving::advance).unwrap();
        world.run_system_once(crate::driving::advance).unwrap();
        world.resource_mut::<ButtonInput<KeyCode>>().press(Enter);
        world.run_system_once(crate::driving::advance).unwrap();
        world.insert_resource(ButtonInput::<KeyCode>::default());
        let drive = world.resource::<crate::driving::Drive>();
        assert_eq!(drive.opponents.len(), 4);
        assert_eq!(drive.opponents[3].vehicle(), Vehicle::Minion);
        assert!(drive.opponents.iter().all(|ai| ai.health().0 > 0));
        for key in [KeyH, Digit1, Digit2, Digit1, Digit5] {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(std::time::Duration::from_millis(50));
            let mut keys = ButtonInput::default();
            keys.press(key);
            world.insert_resource(keys);
            world.run_system_once(crate::driving::advance).unwrap();
        }
        let drive = world.resource::<crate::driving::Drive>();
        assert!(drive.helicopter_pose().is_some());
        assert_eq!(drive.player.state()[0x184], 1);
        assert_eq!(drive.player.state()[0x4a1], 0);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        world.run_system_once(crate::driving::advance).unwrap();
        let drive = world.resource::<crate::driving::Drive>();
        assert!(drive.showing_boss_intro());
        assert_eq!(drive.opponents.len(), 3);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE and FFmpeg"]
    fn last_life_returns_to_title_and_allows_a_fresh_contest() {
        let mut world = menu_world();
        press(&mut world, KeyCode::Enter);
        press(&mut world, KeyCode::Enter);
        press(&mut world, KeyCode::Enter);
        world.resource_mut::<crate::driving::Drive>().lives = 0;
        campaign::advance(&mut world);
        let signature = world
            .resource::<crate::sfx::Sounds>()
            .lost_game_signature()
            .to_vec();
        let defeat_voices = |world: &mut World| {
            world
                .query::<(Entity, &AudioPlayer<crate::sfx::pcm::StereoPcm>)>()
                .iter(world)
                .filter_map(|(entity, voice)| signature.contains(&voice.0).then_some(entity))
                .collect::<Vec<_>>()
        };
        let voices = defeat_voices(&mut world);
        assert!(!voices.is_empty());
        assert_eq!(voices.len(), signature.len());
        // Six separately rounded file transfers, 522 sectors at double speed.
        for millis in [0, 1000, 2479] {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(millis));
            campaign::advance(&mut world);
            assert!(!world.contains_resource::<Contest>());
            assert_eq!(defeat_voices(&mut world), voices);
        }
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(1));
        campaign::advance(&mut world);
        assert!(world.get_entity(voices[0]).is_err());
        assert!(matches!(world.resource::<Contest>().page, Page::Title));
        assert_eq!(world.resource::<Contest>().level, Level::Arena);
        assert!(matches!(
            world.resource::<Contest>().cinema.as_ref().map(|m| m.clip),
            Some(Clip::Title)
        ));
        update(&mut world);
        press(&mut world, KeyCode::Enter);
        assert!(world.resource::<Contest>().cinema.is_none());
        update(&mut world);
        press(&mut world, KeyCode::Enter);
        press(&mut world, KeyCode::Enter);
        press(&mut world, KeyCode::Enter);
        assert_eq!(world.resource::<crate::driving::Drive>().lives, 3);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn keyboard_password_starts_bonus_contest() {
        let mut world = menu_world();
        for key in [
            KeyCode::Enter,
            KeyCode::ArrowRight,
            KeyCode::KeyD,
            KeyCode::KeyP,
            KeyCode::Enter,
            KeyCode::Enter,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            KeyCode::Digit2,
            KeyCode::Digit1,
            KeyCode::Digit4,
            KeyCode::Digit2,
            KeyCode::Digit2,
            KeyCode::Enter,
            KeyCode::Enter,
            KeyCode::Enter,
        ] {
            press(&mut world, key);
        }
        assert!(!world.contains_resource::<Contest>());
        let drive = world.resource::<crate::driving::Drive>();
        assert_eq!(drive.player.difficulty(), Difficulty::Hard);
        assert_eq!(drive.player.level(), Level::Arena5);
        assert_eq!(drive.player.vehicle(), Vehicle::Darkside);
        assert_eq!(drive.opponents.len(), 5);
        assert_eq!(
            world
                .query_filtered::<Entity, With<ContestScreen>>()
                .iter(&world)
                .count(),
            0
        );
    }
}
