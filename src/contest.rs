//! Keyboard contest selection and five-slot JP stage passwords.
use crate::campaign::{self, Campaign, PasswordModes, Stage};
use crate::cinema::{Cinema, Clip};
use anyhow::Result;
use bevy::prelude::*;
use std::time::Duration;
use tm1_assets::screen::{self, Face, Fonts};
use tm1_core::drive::{Difficulty, Level, Vehicle};

#[derive(Component)]
struct ContestScreen;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Title,
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
pub const INFINITE_WEAPONS: &str = "< =@@";
pub const MINION_AND_FRIENDS: &str = "<>@=<";
pub const HELICOPTER: &str = "@@<> ";
const GOD_MODE: &str = "=<> @";
#[derive(Resource)]
pub struct Contest {
    cinema: Option<Cinema>,
    idle: Duration,
    fonts: Fonts,
    title: Vec<u8>,
    garage: crate::garage::Garage,
    field_fraction: u128,
    history: crate::scroll::Scroll,
    history_background: Vec<u8>,
    battle: Vec<u8>,
    cards: Vec<Vec<[u8; 4]>>,
    options: Vec<u8>,
    controls: crate::pause::Art,
    settings: crate::pause::Settings,
    button_sounds: Vec<Handle<AudioSource>>,
    password_from_options: bool,
    cursor_fields: u32,
    entry: Vec<u8>,
    image: Handle<Image>,
    theme: Handle<AudioSource>,
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
            Page::History | Page::Showcase(_) => {
                let fields = self.presentation_fields(dt);
                if fields == 0 && !select {
                    return true;
                }
                let done = match self.page {
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
                    self.page = Page::Title;
                    self.play(world, Some(Clip::Title));
                    return true;
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
        let car = match self.page {
            Page::Contest
            | Page::Bio
            | Page::Battle(_)
            | Page::Options(_)
            | Page::Controls
            | Page::Password
            | Page::Wrong(_) => Some(self.vehicle),
            Page::Showcase(i) => Some(i),
            _ => None,
        };
        self.garage.sound(world, car);
        if matches!(self.page, Page::Title | Page::History | Page::Showcase(_))
            && self.cinema.is_none()
        {
            if self.theme_entity.is_none() {
                let gain = world
                    .get_resource::<crate::driving::Drive>()
                    .map_or(1., |d| d.settings.music_gain());
                self.theme_entity = Some(
                    world
                        .spawn((
                            campaign::StageEntity,
                            AudioPlayer::new(self.theme.clone()),
                            PlaybackSettings::LOOP
                                .with_volume(bevy::audio::Volume::Linear(0.35 * gain)),
                        ))
                        .id(),
                );
            }
        } else if let Some(entity) = self.theme_entity.take() {
            world.despawn(entity);
        }
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
            Page::Options(_) => self.options.clone(),
            Page::Controls => {
                return self.controls.frame(
                    &self.fonts,
                    self.level,
                    crate::pause::Page::Controls,
                    self.settings,
                );
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
            Page::Battle(row) | Page::Options(row) => {
                let difficulty = match self.difficulty {
                    Difficulty::Easy => "EASY",
                    Difficulty::Medium => "MEDIUM",
                    Difficulty::Hard => "HARD",
                };
                let controls = ["CTL FREAK", "STANDARD", "DEADLY FORCE", "RUN N' GUN"]
                    [usize::from(self.settings.layout)];
                let choices: &[(&str, &str)] = if matches!(self.page, Page::Battle(_)) {
                    &[("START", ""), ("OPTIONS", ""), ("EXIT", "")]
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
            Page::Controls => unreachable!(),
            Page::Title => {
                line(165, Face::Text, "1P CONTEST");
                line(193, Face::Text, "PRESS ENTER");
            }
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
                if self.page == Page::Password && self.cursor_fields / 20 % 2 == 1 {
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
            Page::Options(_) => 18001,
            Page::Battle(_) | Page::Wrong(_) => 1201,
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
            Page::Battle(_) => self.leave_battle(),
            Page::Wrong(_) => self.page = self.after_password(),
            _ => self.page = Page::Battle(0),
        }
        true
    }
    fn open_password(&mut self, from_options: bool) {
        self.password_from_options = from_options;
        self.cursor_fields = 0;
        self.slots = [b' '; 5];
        self.cursor = 0;
        self.page = Page::Password;
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
            Page::Password if enter => {
                if matches!(self.page, Page::Wrong(_)) {
                    3
                } else {
                    2
                }
            }
            Page::Options(0 | 1) if enter => 0,
            Page::History | Page::Showcase(_) => return None,
            _ if enter => 1,
            Page::Password if keys.any_just_pressed([Digit1, Digit2, Digit3, Digit4, Space]) => 1,
            Page::Password | Page::Controls if horizontal => 0,
            Page::Battle(_) | Page::Wrong(_) if vertical => 0,
            Page::Options(n) if vertical || (n < 2 && horizontal) => 0,
            Page::Contest if vertical || (self.garage.row != 2 && horizontal) => 0,
            _ => return None,
        };
        Some(self.button_sounds[index].clone())
    }
    fn input(&mut self, keys: &ButtonInput<KeyCode>) -> bool {
        let key = |k| keys.just_pressed(k);
        match self.page {
            Page::History | Page::Showcase(_) => {}
            Page::Title => {
                if key(KeyCode::Enter) {
                    self.page = Page::Contest;
                    self.garage.reset();
                }
            }
            Page::Bio => {
                if key(KeyCode::Enter) || key(KeyCode::Escape) {
                    self.page = Page::Contest;
                }
            }
            Page::Contest => {
                if key(KeyCode::Enter) && self.garage.row == 0 {
                    self.page = Page::Battle(0);
                    self.idle = Duration::ZERO;
                    return false;
                }
                if key(KeyCode::Enter) && self.garage.row == 1 {
                    self.page = Page::Bio;
                } else if key(KeyCode::ArrowUp) {
                    self.garage.row = (self.garage.row + 2) % 3;
                } else if key(KeyCode::ArrowDown) {
                    self.garage.row = (self.garage.row + 1) % 3;
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
            Page::Controls => {
                if key(KeyCode::Enter) || key(KeyCode::Escape) {
                    self.page = Page::Options(1);
                } else if key(KeyCode::ArrowLeft) {
                    self.settings.layout = self.settings.layout.saturating_sub(1);
                } else if key(KeyCode::ArrowRight) {
                    self.settings.layout = (self.settings.layout + 1).min(3);
                }
            }
            Page::Password => {
                if key(KeyCode::Escape) {
                    self.page = self.after_password();
                } else if key(KeyCode::Enter) {
                    let code = std::str::from_utf8(&self.slots).unwrap();
                    self.page = self.after_password();
                    match code {
                        INFINITE_WEAPONS => self.modes.infinite_weapons = true,
                        HELICOPTER => self.modes.helicopter = true,
                        GOD_MODE => self.modes.god_mode = true,
                        "<><<@" => self.page = Page::BossPreview(std::time::Duration::from_secs(5)),
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
                } else if key(KeyCode::ArrowLeft) {
                    self.cursor = (self.cursor + 4) % 5;
                } else if key(KeyCode::ArrowRight) {
                    self.cursor = (self.cursor + 1) % 5;
                } else if let Some((_, ch)) = [
                    (KeyCode::Digit1, b'<'),
                    (KeyCode::Digit2, b'='),
                    (KeyCode::Digit3, b'>'),
                    (KeyCode::Digit4, b'@'),
                    (KeyCode::Space, b' '),
                ]
                .into_iter()
                .find(|(k, _)| key(*k))
                {
                    self.slots[self.cursor] = ch;
                    self.cursor = (self.cursor + 1) % 5;
                }
            }
            Page::BossPreview(_) => {
                if key(KeyCode::Enter) {
                    self.page = self.after_password();
                }
            }
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
    let disc = &world.resource::<Campaign>().0;
    let theme = crate::music::track(disc, 2)?;
    let fonts = Fonts::load(disc)?;
    let mut menu = Contest {
        cinema: None,
        idle: Duration::ZERO,
        history: crate::scroll::Scroll::load(disc.read("UATEXT/HISTORY.TXT")?, &fonts)?,
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
        settings: world
            .get_resource::<crate::driving::Drive>()
            .map_or_else(Default::default, |d| d.settings),
        button_sounds: Vec::new(),
        password_from_options: false,
        cursor_fields: 0,
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
        let sample = crate::sfx::Sample::load(&bank, program, false)?.wave(0.)?;
        menu.button_sounds
            .push(world.resource_mut::<Assets<AudioSource>>().add(sample));
    }
    campaign::clear_stage(world);
    menu.image = crate::ending::spawn_screen(world, ContestScreen, menu.frame());
    menu.theme = world.resource_mut::<Assets<AudioSource>>().add(theme);
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
            let select = keys.any_pressed([KeyCode::Enter, KeyCode::Space])
                || !scripted && world
                    .query::<&Gamepad>()
                    .iter(world)
                    .any(|p| p.any_pressed([GamepadButton::South, GamepadButton::Start]));
            let dt = menu_delta(world);
            let handle = menu.image.clone();
            let mut images = world.resource_mut::<Assets<Image>>();
            let movie = menu.cinema.as_mut().unwrap();
            let done = movie
                .advance(dt, select, &mut images, &handle)
                .unwrap_or_else(|err| {
                    warn!("Disc cinema ended early: {err:#}");
                    true
                });
            if done {
                let next = movie.clip.next();
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
        if matches!(menu.page, Page::History | Page::Showcase(_)) || menu.page == Page::Title && !held {
            if menu.update_attract(world, select, up, down) { return; }
        } else if menu.page == Page::Title {
            menu.idle = Duration::ZERO;
        }
        let keys = if scripted { &no_input } else { world.resource::<ButtonInput<KeyCode>>() };
        let timed_out = menu.tick_setup_timeout(held,menu_delta(world));
        let expired = if let Page::BossPreview(remaining) = &mut menu.page {
            *remaining = remaining.saturating_sub(world.resource::<Time<Real>>().delta());
            remaining.is_zero()
        } else {
            false
        };
        if expired {
            menu.page = menu.after_password();
        }
        let animate = if menu.page == Page::Contest {
            let fields=menu.presentation_fields(menu_delta(world));
            menu.garage.tick(fields);
            fields != 0
        } else if menu.page==Page::Password {
            let before=menu.cursor_fields/20;
            let fields=menu.presentation_fields(menu_delta(world));
            menu.cursor_fields=menu.cursor_fields.wrapping_add(fields);
            before!=menu.cursor_fields/20
        } else {
            false
        };
        if !timed_out && !animate && !expired && keys.get_just_pressed().next().is_none() {
            return;
        }
        let previous = menu.page;
        let start_battle = !expired && menu.input(keys);
        if !expired && let Some(voice) = menu.input_sound(previous, keys) {
            world.spawn((
                AudioPlayer::new(voice),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.44 * menu.settings.effects_gain())),
            ));
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
    let layout = menu.settings.layout;
    let mut stage = Stage::load(
        &world.resource::<Campaign>().0,
        menu.level,
        Vehicle::CARD_ORDER[menu.vehicle],
        campaign::Entry::New(menu.difficulty),
    )?;
    stage.apply_passwords(menu.modes);
    crate::transition::begin(world, stage, false, true, Some(layout))?;
    world.remove_resource::<Contest>();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu_world() -> World {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<crate::additive::Additive>>();
        world.init_resource::<Assets<AudioSource>>();
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
    fn press(world: &mut World, key: KeyCode) {
        let mut keys = ButtonInput::default();
        keys.press(key);
        world.insert_resource(keys);
        update(world);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        if world.contains_resource::<crate::transition::Transition>() {
            crate::transition::complete(world);
        }
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
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        tick(&mut world, 1000);
        assert!(clip(&world) == Some(Clip::Logo));
        tick(&mut world, 1000);
        assert!(clip(&world) == Some(Clip::Title));
        world.insert_resource(ButtonInput::<KeyCode>::default());
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
        tick(&mut world, 2500);
        assert!(clip(&world).is_none());
        assert!(world.resource::<Contest>().theme_entity.is_some());
        tick(&mut world, 0);
        tick(&mut world, 20017);
        assert!(clip(&world) == Some(Clip::Attract));
        assert!(world.resource::<Contest>().theme_entity.is_none());
        tick(&mut world, 0);
        press(&mut world, KeyCode::Enter);
        assert!(clip(&world) == Some(Clip::Title));
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
        assert!(!world.contains_resource::<Contest>());
        campaign::advance(&mut world);
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
