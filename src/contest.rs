//! Keyboard contest selection and five-slot JP stage passwords.
use crate::campaign::{self, Campaign, Stage};
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
}
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
    slots: [u8; 5],
    cursor: usize,
    released: bool,
}
impl Contest {
    fn sync_theme(&mut self, world: &mut World) {
        if self.page == Page::Title {
            if self.theme_entity.is_none() {
                self.theme_entity = Some(
                    world
                        .spawn((
                            campaign::StageEntity,
                            AudioPlayer::new(self.theme.clone()),
                            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.35)),
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
                line(88, Face::Text, "ENTER PASSWORD");
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
                    if let Some(level) =
                        Level::from_stage_password(std::str::from_utf8(&self.slots).unwrap())
                    {
                        self.level = level;
                        self.page = Page::Contest;
                    } else {
                        self.page = Page::Wrong;
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
        if keys.get_just_pressed().next().is_none() {
            return;
        }
        if menu.input(keys) {
            start = Some((menu.level, Vehicle::ROSTER[menu.vehicle], menu.difficulty));
        } else if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&menu.image) {
            img.data = Some(menu.frame());
        }
        menu.sync_theme(world);
    });
    if let Some((level, vehicle, difficulty)) = start {
        match Stage::load(
            &world.resource::<Campaign>().0,
            level,
            vehicle,
            campaign::Entry::New(difficulty),
        ) {
            Ok(stage) => {
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

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn keyboard_password_starts_bonus_contest() {
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
            let mut keys = ButtonInput::default();
            keys.press(key);
            world.insert_resource(keys);
            update(&mut world);
            world.insert_resource(ButtonInput::<KeyCode>::default());
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
