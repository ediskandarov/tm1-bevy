//! SCREEN.c screenScrollText field timing, file layout and type positioning.
use anyhow::{Context, Result, ensure};
use tm1_assets::screen::{Face, Fonts};

pub struct Scroll {
    pub lines: Vec<String>,
    centered: bool,
    spacing: i32,
    y: i32,
    pub speed: u8,
    tick: u8,
    delay: u8,
}
#[derive(PartialEq, Eq)]
pub enum Outcome {
    Running,
    Finished,
    Skipped,
}
impl Scroll {
    pub fn load(bytes: Vec<u8>, fonts: &Fonts) -> Result<Self> {
        let text = String::from_utf8(bytes)?;
        let mut rows = text.split("\r\n");
        let (count, align) = rows
            .next()
            .and_then(|s| s.split_once(' '))
            .context("missing scroll header")?;
        let count = count.parse::<usize>()?;
        ensure!(
            matches!(align, "L" | "C") && count > 0,
            "unsupported scroll header"
        );
        let lines = rows
            .filter(|line| !line.starts_with("@S"))
            .take(count)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        ensure!(lines.len() == count, "incomplete scroll text");
        let mut scroll = Self {
            lines,
            centered: align == "C",
            spacing: fonts.line_height() + 10,
            y: 0,
            speed: 3,
            tick: 0,
            delay: 0,
        };
        scroll.reset();
        Ok(scroll)
    }
    pub fn reset(&mut self) {
        self.y = 230 + self.spacing;
        self.speed = 3;
        self.tick = 0;
        self.delay = 0;
    }
    pub fn field(&mut self, select: bool, up: bool, down: bool) -> Outcome {
        if self.speed < 6 && self.tick >= self.speed {
            self.y -= 1;
            self.tick = 0;
        }
        self.tick = self.tick.saturating_add(1);
        if self.delay == 0 && (up || down) {
            self.speed = if up {
                self.speed.saturating_sub(1).max(1)
            } else {
                (self.speed + 1).min(6)
            };
            self.delay = 30;
        } else if select {
            return Outcome::Skipped;
        } else {
            self.delay = self.delay.saturating_sub(1);
        }
        if self.y + (self.lines.len() as i32 - 1) * self.spacing < 0 {
            Outcome::Finished
        } else {
            Outcome::Running
        }
    }
    pub fn draw(&self, out: &mut [u8], fonts: &Fonts) {
        for (i, line) in self.lines.iter().enumerate() {
            let y = self.y + i as i32 * self.spacing;
            if (0..=240 + self.spacing).contains(&y) {
                if self.centered {
                    fonts.centered(out, Face::Text, y, line);
                } else {
                    fonts.at(out, Face::Text, 15, y, line);
                }
            }
        }
    }
}
