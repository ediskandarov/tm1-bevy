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
    pub fn cd_gain(self) -> f32 {
        (match self.mix {
            0 => 0,
            99 => 127,
            v => 97 * u16::from(v) / 99 + 30,
        }) as f32
            * (0.35 / 78.)
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseInput {
    Ready,
    WaitRelease,
    ChangeOnRelease(Option<Page>),
}

impl PauseInput {
    pub fn defer_transition(
        &mut self,
        current: Page,
        next: Option<Page>,
        held: bool,
    ) -> Option<Page> {
        if held && (next.is_none() || (current == Page::Choices(2) && next == Some(Page::Audio(0))))
        {
            *self = Self::ChangeOnRelease(next);
            Some(current)
        } else {
            next
        }
    }
}

#[derive(Default)]
pub(crate) struct AbortWait {
    elapsed: std::time::Duration,
    pub finished: bool,
}

impl AbortWait {
    pub fn advance(&mut self, dt: std::time::Duration, confirm: bool) {
        let before_poll = self.elapsed < std::time::Duration::from_millis(500);
        self.elapsed += dt;
        let fields = self.elapsed.as_nanos() * 60 / 1_000_000_000;
        self.finished |= fields >= 31 || (before_poll && fields == 30 && confirm);
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

pub fn campaign_action(
    page: Page,
    keys: &ButtonInput<KeyCode>,
    pad: Option<&Gamepad>,
) -> Option<Action> {
    let select = keys.any_pressed([KeyCode::Enter, KeyCode::Space])
        || pad.is_some_and(|p| {
            p.any_pressed([
                GamepadButton::South,
                GamepadButton::Start,
                GamepadButton::Select,
            ])
        });
    let down = |key, button| keys.pressed(key) || pad.is_some_and(|p| p.pressed(button));
    if keys.just_pressed(KeyCode::KeyP) {
        Some(Action::Resume)
    } else if page == Page::Controls && select {
        Some(Action::Select)
    } else if page != Page::Controls && down(KeyCode::ArrowUp, GamepadButton::DPadUp) {
        Some(Action::Up)
    } else if page != Page::Controls && down(KeyCode::ArrowDown, GamepadButton::DPadDown) {
        Some(Action::Down)
    } else if !matches!(page, Page::Choices(_)) && down(KeyCode::ArrowLeft, GamepadButton::DPadLeft)
    {
        Some(Action::Left)
    } else if !matches!(page, Page::Choices(_))
        && down(KeyCode::ArrowRight, GamepadButton::DPadRight)
    {
        Some(Action::Right)
    } else {
        select.then_some(Action::Select)
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
#[derive(Clone, Copy)]
pub struct ControlInput {
    pub action: Option<Action>,
    pub redraw: bool,
    pub reset_idle: bool,
}

pub fn campaign_pad_held(keys: &ButtonInput<KeyCode>, pad: Option<&Gamepad>) -> bool {
    use KeyCode::*;
    duel_pad_held(keys, 0)
        || keys.any_pressed([
            Enter, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, KeyZ, KeyV, KeyL, KeyB, Digit1,
            Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8,
        ])
        || pad.is_some_and(|p| p.get_pressed().next().is_some())
}

pub fn duel_pad_held(keys: &ButtonInput<KeyCode>, slot: usize) -> bool {
    use KeyCode::*;
    if slot == 0 {
        keys.any_pressed([
            KeyA, KeyD, KeyW, KeyS, Space, KeyE, KeyQ, KeyF, KeyT, ShiftLeft, KeyC, KeyP,
        ])
    } else {
        keys.any_pressed([
            ArrowLeft,
            ArrowRight,
            ArrowUp,
            ArrowDown,
            Comma,
            Period,
            Slash,
            ShiftRight,
            KeyL,
            KeyK,
            KeyO,
            NumpadEnter,
        ])
    }
}

#[derive(Default)]
pub struct ControlsView {
    elapsed: std::time::Duration,
    arrows: [bool; 2],
    ready: bool,
}

impl ControlsView {
    fn label_visible(&self) -> bool {
        // screenChooseControls waits for two VSyncs per ordinary draw loop.
        let draw = self.elapsed.as_nanos() * 30 / 1_000_000_000;
        draw >= 60 || (0xf0f0 >> (draw % 16)) & 1 != 0
    }

    pub fn advance(
        &mut self,
        delta: std::time::Duration,
        keys: &ButtonInput<KeyCode>,
        slot: Option<usize>,
    ) -> ControlInput {
        use KeyCode::*;
        let previous = (self.label_visible(), self.arrows);
        self.elapsed = self.elapsed.saturating_add(delta);
        let [left, right, select] = match slot {
            None => [ArrowLeft, ArrowRight, Space],
            Some(0) => [KeyA, KeyD, Space],
            Some(_) => [ArrowLeft, ArrowRight, Comma],
        };
        let waiting = !self.ready;
        let action = if keys.just_pressed(Escape) {
            Some(Action::Resume)
        } else if waiting {
            let held = match slot {
                Some(slot) => duel_pad_held(keys, slot) || keys.pressed(Enter),
                None => campaign_pad_held(keys, None),
            };
            if !held {
                self.ready = true;
                self.arrows = [false; 2];
            }
            None
        } else if keys.any_pressed([select, Enter]) {
            Some(Action::Select)
        } else if keys.pressed(left) || keys.pressed(right) {
            self.ready = false;
            self.arrows = [keys.pressed(left), !keys.pressed(left)];
            Some(if self.arrows[0] {
                Action::Left
            } else {
                Action::Right
            })
        } else {
            None
        };
        ControlInput {
            action,
            redraw: previous.1 != self.arrows
                || (slot.is_some() && previous.0 != self.label_visible()),
            reset_idle: waiting || action.is_some(),
        }
    }

    pub fn frame(&self, art: &Art, fonts: &Fonts, layout: u8, slot: Option<usize>) -> Vec<u8> {
        let mut frame = art.controls_frame(fonts, layout, self.arrows);
        if let Some(slot) = slot
            && self.label_visible()
        {
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
    last: Option<(Page, Settings, [bool; 2])>,
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
    let active = (page, drive.settings, drive.control_arrows);
    if state.last != Some(active) {
        if let Some(mut image) = images.get_mut(&sprite.image) {
            image.data = Some(if page == Page::Controls {
                arena.pause_art.controls_frame(
                    &arena.fonts,
                    drive.settings.layout,
                    drive.control_arrows,
                )
            } else {
                arena.pause_art.frame(
                    &arena.fonts,
                    Some(drive.player.level()),
                    page,
                    drive.settings,
                )
            });
        }
        state.last = Some(active);
    }
}

#[cfg(test)]
mod abort_tests {
    use super::*;
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn abort_wait_matches_jp_field_poll() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        for addr in [0x8016206c, 0x800ffb2c] {
            cpu.put(addr, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        for (confirm, pad) in [(true, 0x40), (false, 0)] {
            cpu.call(0x800ffb98, &[2, pad]);
            let mut fields = 0;
            let selected = cpu.call_observed(0x8013f728, &[30, 30], 0x80000080, |cpu| {
                if cpu.pc() == 0x8016206c {
                    fields += 1;
                }
            });
            assert_eq!(selected != 0, confirm);
            let mut wait = AbortWait::default();
            for field in 1..=fields {
                wait.advance(std::time::Duration::from_nanos(16_666_667), confirm);
                assert_eq!(wait.finished, field == fields);
            }
        }
        let mut wait = AbortWait::default();
        wait.advance(std::time::Duration::from_millis(500), false);
        wait.advance(std::time::Duration::from_millis(5), true);
        assert!(!wait.finished);
        wait.advance(std::time::Duration::from_millis(12), false);
        assert!(wait.finished);
    }
}
