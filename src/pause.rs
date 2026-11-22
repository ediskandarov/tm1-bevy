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
    pub fn effects_volume(self) -> u8 {
        (match self.mix {
            0 => 57,
            99 => 0,
            v => 47 * u16::from(99 - v) / 99 + 10,
        }) as u8
    }
    pub fn effects_gain(self) -> f32 {
        f32::from(self.effects_volume()) / 33.
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
// The control diagram waits for VSync before input and again before display.
pub const CONTROL_IDLE_FIELDS: u128 = 2 * 18_001;

pub fn options_timed_out(
    page: Page,
    elapsed: &mut std::time::Duration,
    delta: std::time::Duration,
    input: bool,
) -> bool {
    if input || !matches!(page, Page::Audio(_) | Page::Controls) {
        *elapsed = std::time::Duration::ZERO;
        return false;
    }
    *elapsed += delta;
    let fields = if page == Page::Controls {
        CONTROL_IDLE_FIELDS
    } else {
        18_001
    };
    elapsed.as_nanos() * 60 >= fields * 1_000_000_000
}

pub fn navigation_sound(page: Page, action: Action, layout_changed: bool) -> Option<bool> {
    use Action::*;
    match (page, action) {
        (_, Select | Resume) => Some(true),
        (Page::Choices(_) | Page::Audio(_), Up | Down) | (Page::Audio(0 | 1), Left | Right) => {
            Some(false)
        }
        (Page::Controls, Left | Right) if layout_changed => Some(false),
        _ => None,
    }
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
#[derive(Default)]
pub struct DuelControlsView {
    elapsed: std::time::Duration,
    arrows: [bool; 2],
}

impl DuelControlsView {
    fn label_visible(&self) -> bool {
        // screenChooseControls waits for two VSyncs per ordinary draw loop.
        let draw = self.elapsed.as_nanos() * 30 / 1_000_000_000;
        draw >= 60 || (0xf0f0 >> (draw % 16)) & 1 != 0
    }

    pub fn advance(
        &mut self,
        delta: std::time::Duration,
        keys: &ButtonInput<KeyCode>,
        slot: usize,
    ) -> bool {
        let previous = (self.label_visible(), self.arrows);
        self.elapsed = self.elapsed.saturating_add(delta);
        let [left, right] = if slot == 0 {
            [KeyCode::KeyA, KeyCode::KeyD]
        } else {
            [KeyCode::ArrowLeft, KeyCode::ArrowRight]
        };
        self.arrows = [
            keys.pressed(left),
            !keys.pressed(left) && keys.pressed(right),
        ];
        previous != (self.label_visible(), self.arrows)
    }

    pub fn frame(&self, art: &Art, fonts: &Fonts, layout: u8, slot: usize) -> Vec<u8> {
        let mut frame = art.controls_frame(fonts, layout, self.arrows);
        if self.label_visible() {
            fonts.centered_at(
                &mut frame,
                Face::Choice,
                159,
                40,
                ["Player 1", "Player 2"][slot],
            );
        }
        frame
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
    fn controls_frame(&self, fonts: &Fonts, layout: u8, arrows: [bool; 2]) -> Vec<u8> {
        let mut frame = self.controls[usize::from(layout)].clone();
        for (visible, pressed, x, glyph) in [
            (layout > 0, arrows[0], 14, "<"),
            (layout < 3, arrows[1], 285, ">"),
        ] {
            if visible {
                fonts.at(
                    &mut frame,
                    if pressed {
                        Face::ArrowActive
                    } else {
                        Face::Control
                    },
                    x,
                    63,
                    glyph,
                );
            }
        }
        frame
    }

    pub fn audio_frame(
        &self,
        fonts: &Fonts,
        row: usize,
        settings: Settings,
        track: Option<&str>,
    ) -> Vec<u8> {
        let mut out = self.audio.clone();
        let face = |i| {
            if i == row {
                Face::Selected
            } else {
                Face::Choice
            }
        };
        for (i, (label, value)) in [("MUSIC:", settings.music), ("SFX:", settings.effects)]
            .into_iter()
            .enumerate()
        {
            let y = 120 + 20 * i as i32;
            let n = usize::from(value);
            fonts.at(&mut out, face(i), 30, y, label);
            fonts.at(
                &mut out,
                face(i),
                160,
                y,
                &format!("{}{}", "|".repeat(n), "*".repeat(14 - n)),
            );
        }
        if let Some(title) = track {
            fonts.at(&mut out, face(2), 30, 160, "PLAY TRACK:");
            fonts.centered_at(&mut out, face(2), 159, 175, title);
        }
        fonts.at(
            &mut out,
            face(2 + usize::from(track.is_some())),
            30,
            200,
            "EXIT",
        );
        out
    }
    pub fn frame(
        &self,
        fonts: &Fonts,
        level: Option<Level>,
        page: Page,
        settings: Settings,
    ) -> Vec<u8> {
        let face = |selected| {
            if selected {
                Face::Selected
            } else {
                Face::Choice
            }
        };
        match page {
            Page::Audio(row) => self.audio_frame(fonts, usize::from(row), settings, None),
            Page::Controls => self.controls_frame(fonts, settings.layout, [false; 2]),
            page => {
                let mut out = [0, 0, 0, 128].repeat(screen::WIDTH * screen::HEIGHT);
                if let Page::Choices(row) = page {
                    fonts.centered(&mut out, Face::Text, 60, "PAUSED");
                    if let Some((level, code)) = level.and_then(|l| l.password().map(|pw| (l, pw)))
                    {
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
                Some(drive.player.level()),
                page,
                drive.settings,
            ));
        }
        state.last = Some((page, drive.settings));
    }
}
