//! SCREEN.c screenCheckMidGameOptions and the in-battle options screens.
use anyhow::Result;
use bevy::prelude::*;
use tm1_assets::{
    disc::Disc,
    screen::{self, Face, Fonts},
};
use tm1_core::drive::Level;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Choices(u8),
    Audio(u8),
    Controls,
    Aborted,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub music: u8,
    pub effects: u8,
    pub layout: u8,
    mix: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            music: 7,
            effects: 7,
            layout: 1,
            mix: 49,
        }
    }
}
impl Settings {
    fn percent(level: u8) -> u8 {
        if level == 14 { 99 } else { 7 * level }
    }
    // UA_SOUND.c uasoundSetMusicVolume. Normalize to the initial stage mix.
    pub fn music_gain(self) -> f32 {
        (match self.mix {
            0 => 0,
            99 => 127,
            v => 97 * u16::from(v) / 99 + 30,
        }) as f32
            / 78.
    }
    pub fn effects_gain(self) -> f32 {
        (match self.mix {
            0 => 57,
            99 => 0,
            v => 47 * u16::from(99 - v) / 99 + 10,
        }) as f32
            / 33.
    }
}
#[derive(Clone, Copy)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Select,
    Resume,
}
pub fn navigate(page: Page, action: Action, settings: &mut Settings) -> Option<Page> {
    use Action::*;
    use Page::*;
    match (page, action) {
        (Aborted, _) => Some(Aborted),
        (_, Resume) | (Choices(0), Select) | (Audio(2), Select) | (Controls, Select) => None,
        (Choices(1), Select) => Some(Aborted),
        (Choices(2), Select) => Some(Audio(0)),
        (Choices(3), Select) => Some(Controls),
        (Choices(i), Up) => Some(Choices((i + 3) % 4)),
        (Choices(i), Down) => Some(Choices((i + 1) % 4)),
        (Audio(i), Up) => Some(Audio((i + 2) % 3)),
        (Audio(i), Down) => Some(Audio((i + 1) % 3)),
        (Audio(i @ 0..=1), Left | Right) => {
            let dir = if matches!(action, Right) { 1 } else { -1 } * if i == 0 { 1 } else { -1 };
            settings.music = settings.music.saturating_add_signed(dir).min(14);
            settings.effects = settings.effects.saturating_add_signed(-dir).min(14);
            settings.mix = if i == 0 {
                99 - Settings::percent(settings.effects)
            } else {
                Settings::percent(settings.music)
            };
            Some(page)
        }
        (Controls, Left | Right) => {
            settings.layout = settings
                .layout
                .saturating_add_signed(if matches!(action, Right) { 1 } else { -1 })
                .min(3);
            Some(page)
        }
        _ => Some(page),
    }
}
pub struct Art {
    audio: Vec<u8>,
    controls: Vec<Vec<u8>>,
}
impl Art {
    pub fn load(disc: &Disc) -> Result<Self> {
        let load = |name: &str| screen::background(&disc.read(&format!("UASCREEN/{name}.TIM"))?);
        Ok(Self {
            audio: load("SCRNFC1")?,
            controls: (0..4)
                .map(|i| load(&format!("SCRNFB{i}")))
                .collect::<Result<_>>()?,
        })
    }
    pub fn frame(&self, fonts: &Fonts, level: Level, page: Page, settings: Settings) -> Vec<u8> {
        let face = |selected| {
            if selected {
                Face::Selected
            } else {
                Face::Choice
            }
        };
        match page {
            Page::Audio(row) => {
                let mut out = self.audio.clone();
                for (i, (y, label)) in [(120, "MUSIC:"), (140, "SFX:"), (200, "EXIT")]
                    .into_iter()
                    .enumerate()
                {
                    let f = face(i == usize::from(row));
                    fonts.at(&mut out, f, 30, y, label);
                    if i < 2 {
                        let n = usize::from([settings.music, settings.effects][i]);
                        fonts.at(
                            &mut out,
                            f,
                            160,
                            y,
                            &format!("{}{}", "|".repeat(n), "*".repeat(14 - n)),
                        );
                    }
                }
                out
            }
            Page::Controls => {
                let mut out = self.controls[usize::from(settings.layout)].clone();
                if settings.layout > 0 {
                    fonts.at(&mut out, Face::Control, 14, 63, "<");
                }
                if settings.layout < 3 {
                    fonts.at(&mut out, Face::Control, 285, 63, ">");
                }
                out
            }
            page => {
                let mut out = [0, 0, 0, 128].repeat(screen::WIDTH * screen::HEIGHT);
                if let Page::Choices(row) = page {
                    fonts.centered(&mut out, Face::Text, 60, "PAUSED");
                    if let Some(code) = level.password() {
                        let name = if level == Level::Rooftops {
                            "FINAL".into()
                        } else {
                            level.name().to_ascii_uppercase()
                        };
                        fonts.centered(&mut out, Face::Choice, 90, &format!("{name} PASSWORD:"));
                        fonts.centered(&mut out, Face::Password, 115, code);
                    }
                    for (i, label) in ["START", "QUIT", "AUDIO", "CONTROLLER OPTIONS"]
                        .into_iter()
                        .enumerate()
                    {
                        fonts.centered(
                            &mut out,
                            face(i == usize::from(row)),
                            140 + i as i32 * 20,
                            label,
                        );
                    }
                } else {
                    fonts.centered(&mut out, Face::Text, 115, "BATTLE ABORTED");
                }
                out
            }
        }
    }
}
#[derive(Component, Default)]
pub struct Overlay {
    last: Option<(Page, Settings)>,
}
pub fn draw(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    arena: Res<crate::render::Arena>,
    mut images: ResMut<Assets<Image>>,
    mut overlay: Single<(&mut Overlay, &mut Sprite, &mut Visibility)>,
) {
    let (state, sprite, visibility) = &mut *overlay;
    let Some(page) = drive.pause else {
        **visibility = Visibility::Hidden;
        state.last = None;
        return;
    };
    **visibility = Visibility::Visible;
    sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
    if state.last != Some((page, drive.settings)) {
        if let Some(mut image) = images.get_mut(&sprite.image) {
            image.data = Some(arena.pause_art.frame(
                &arena.fonts,
                drive.player.level(),
                page,
                drive.settings,
            ));
        }
        state.last = Some((page, drive.settings));
    }
}
