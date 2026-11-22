//! Keyboard contest selection and five-slot JP stage passwords.
use crate::campaign::{self, Campaign, PasswordModes, Stage};
use anyhow::Result;
use bevy::prelude::*;
use tm1_assets::screen::{self, Face, Fonts};
use tm1_core::drive::{Difficulty, Level, Vehicle};

#[derive(Component)]
struct ContestScreen;
#[derive(PartialEq, Eq)]
enum Page {
    Title,
    Contest,
    Password,
    Wrong,
    BossPreview(std::time::Duration),
}
pub const INFINITE_WEAPONS: &str = "< =@@";
pub const MINION_AND_FRIENDS: &str = "<>@=<";
pub const HELICOPTER: &str = "@@<> ";
const GOD_MODE: &str = "=<> @";
#[derive(Resource)]
pub struct Contest {
    fonts: Fonts,
    title: Vec<u8>,
    cars: Vec<u8>,
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
impl Contest {
    fn sync_theme(&mut self, world: &mut World) {
        if self.page == Page::Title {
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
    fn frame(&self) -> Vec<u8> {
        let mut frame = match self.page {
            Page::Title => self.title.clone(),
            Page::Contest => self.cars.clone(),
            _ => self.entry.clone(),
        };
        let mut line = |y, face, text: &str| self.fonts.centered(&mut frame, face, y, text);
        match self.page {
            Page::Title => {
                line(165, Face::Text, "1P CONTEST");
                line(193, Face::Text, "PRESS ENTER");
            }
            Page::Contest => {
                line(61, Face::Text, "LEFT RIGHT  CAR");
                line(115, Face::Text, Vehicle::ROSTER[self.vehicle].name());
                line(142, Face::Text, self.level.name());
                line(165, Face::Text, &format!("D  {:?}", self.difficulty));
                line(188, Face::Text, "P  PASSWORD");
                line(211, Face::Text, "ENTER  PLAY");
            }
            Page::Password => {
                line(72, Face::Text, "ENTER PASSWORD");
                line(98, Face::Text, "SPACE  BLANK");
                line(125, Face::Text, "1 TRIANGLE   2 SQUARE");
                line(145, Face::Text, "3 CROSS   4 CIRCLE");
                // Draw one fixed-width slot per glyph, including spaces in cheat codes.
                for (i, &symbol) in self.slots.iter().enumerate() {
                    let x = 107 + i as i32 * 22;
                    self.fonts.at(
                        &mut frame,
                        Face::Password,
                        x,
                        183,
                        std::str::from_utf8(&[symbol]).unwrap(),
                    );
                    if i == self.cursor {
                        for col in x..x + 14 {
                            let at = 4 * (188 * screen::WIDTH + col as usize);
                            frame[at..at + 4].copy_from_slice(&[233, 184, 88, 255]);
                        }
                    }
                }
                self.fonts
                    .centered(&mut frame, Face::Text, 221, "ENTER  ACCEPT");
            }
            Page::BossPreview(_) => {
                for pixel in frame.chunks_exact_mut(4) {
                    for channel in &mut pixel[..3] {
                        *channel /= 2;
                    }
                }
                crate::render::boss_text(&self.fonts, &mut frame);
            }
            Page::Wrong => {
                line(115, Face::Text, "WRONG PASSWORD");
                line(155, Face::Text, "ENTER  TRY AGAIN");
                line(185, Face::Text, "ESC  EXIT");
            }
        }
        frame
    }
    fn input(&mut self, keys: &ButtonInput<KeyCode>) -> bool {
        let key = |k| keys.just_pressed(k);
        match self.page {
            Page::Title => {
                if key(KeyCode::Enter) {
                    self.page = Page::Contest;
                }
            }
            Page::Contest => {
                if key(KeyCode::Enter) {
                    return true;
                }
                if key(KeyCode::Escape) {
                    self.page = Page::Title;
                    self.modes = Default::default();
                    self.level = Level::Arena;
                } else if key(KeyCode::KeyD) {
                    self.difficulty = self.difficulty.next();
                } else if key(KeyCode::KeyP) {
                    self.page = Page::Password;
                    self.slots = [b' '; 5];
                    self.cursor = 0;
                } else if key(KeyCode::ArrowRight) {
                    self.vehicle = (self.vehicle + 1) % Vehicle::ROSTER.len();
                } else if key(KeyCode::ArrowLeft) {
                    self.vehicle =
                        (self.vehicle + Vehicle::ROSTER.len() - 1) % Vehicle::ROSTER.len();
                }
            }
            Page::Password => {
                if key(KeyCode::Escape) {
                    self.page = Page::Contest;
                } else if key(KeyCode::Enter) {
                    let code = std::str::from_utf8(&self.slots).unwrap();
                    self.page = Page::Contest;
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
                                self.page = Page::Wrong;
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
                    self.page = Page::Contest;
                }
            }
            Page::Wrong => {
                if key(KeyCode::Enter) {
                    self.slots = [b' '; 5];
                    self.cursor = 0;
                    self.page = Page::Password;
                } else if key(KeyCode::Escape) {
                    self.page = Page::Contest;
                }
            }
        }
        false
    }
}
pub fn open(world: &mut World) -> Result<()> {
    let player = &world.resource::<crate::driving::Drive>().player;
    show(world, player.vehicle(), Level::Arena, player.difficulty())
}
pub fn show(
    world: &mut World,
    vehicle: Vehicle,
    level: Level,
    difficulty: Difficulty,
) -> Result<()> {
    let disc = &world.resource::<Campaign>().0;
    let theme = crate::music::track(disc, 2)?;
    let mut menu = Contest {
        fonts: Fonts::load(disc)?,
        cars: screen::background(&disc.read("UASCREEN/SCRND.TIM")?)?,
        title: screen::background(&disc.read("UASCREEN/SCRNC.TIM")?)?,
        entry: screen::background(&disc.read("UASCREEN/SCRNFD.TIM")?)?,
        theme: default(),
        theme_entity: None,
        image: default(),
        page: Page::Title,
        vehicle: Vehicle::ROSTER.iter().position(|v| *v == vehicle).unwrap(),
        level,
        difficulty,
        modes: Default::default(),
        slots: [b' '; 5],
        cursor: 0,
        released: false,
    };
    campaign::clear_stage(world);
    menu.image = crate::ending::spawn_screen(world, ContestScreen, menu.frame());
    menu.theme = world.resource_mut::<Assets<AudioSource>>().add(theme);
    menu.sync_theme(world);
    world.insert_resource(menu);
    Ok(())
}
pub fn update(world: &mut World) {
    let Ok(window) = world.query::<&Window>().single(world) else {
        return;
    };
    let focused = window.focused;
    let h = window.height().min(window.width() * 0.75);
    if let Ok(mut sprite) = world
        .query_filtered::<&mut Sprite, With<ContestScreen>>()
        .single_mut(world)
    {
        sprite.custom_size = Some(Vec2::new(h * 4. / 3., h));
    }
    if !focused {
        return;
    }
    let mut start = None;
    world.resource_scope(|world, mut menu: Mut<Contest>| {
        let keys = world.resource::<ButtonInput<KeyCode>>();
        if !menu.released {
            menu.released = keys.get_pressed().next().is_none();
            return;
        }
        let expired = if let Page::BossPreview(remaining) = &mut menu.page {
            *remaining = remaining.saturating_sub(world.resource::<Time<Real>>().delta());
            remaining.is_zero()
        } else {
            false
        };
        if expired {
            menu.page = Page::Contest;
        }
        if !expired && keys.get_just_pressed().next().is_none() {
            return;
        }
        if !expired && menu.input(keys) {
            start = Some((
                menu.level,
                Vehicle::ROSTER[menu.vehicle],
                menu.difficulty,
                menu.modes,
            ));
        } else if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&menu.image) {
            img.data = Some(menu.frame());
        }
        menu.sync_theme(world);
    });
    if let Some((level, vehicle, difficulty, modes)) = start {
        match Stage::load(
            &world.resource::<Campaign>().0,
            level,
            vehicle,
            campaign::Entry::New(difficulty),
        ) {
            Ok(mut stage) => {
                stage.apply_passwords(modes);
                world.remove_resource::<Contest>();
                campaign::replace(world, stage);
            }
            Err(err) => {
                error!("Loading contest failed: {err:#}");
                world.write_message(AppExit::error());
            }
        }
    }
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
    #[ignore = "requires the supported TM1_CUE"]
    fn last_life_returns_to_title_and_allows_a_fresh_contest() {
        let mut world = menu_world();
        press(&mut world, KeyCode::Enter);
        press(&mut world, KeyCode::Enter);
        world.resource_mut::<crate::driving::Drive>().lives = 0;
        campaign::advance(&mut world);
        assert!(!world.contains_resource::<Contest>());
        campaign::advance(&mut world);
        assert!(matches!(world.resource::<Contest>().page, Page::Title));
        assert_eq!(world.resource::<Contest>().level, Level::Arena);
        update(&mut world);
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
