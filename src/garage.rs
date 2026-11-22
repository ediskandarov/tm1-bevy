//! The one-player vehicle carousel in SCREEN.c.
use anyhow::{Result, ensure};
use bevy::prelude::*;
use tm1_assets::{
    disc::Disc,
    scene::{Dmd, Scene},
    screen::{self, Face, Fonts},
    texture::Vram,
};
use tm1_core::drive::Vehicle;

pub struct Garage {
    backdrop: Vec<u8>,
    demo_backdrop: Vec<u8>,
    audio: Vec<(AudioSource, AudioSource, f32)>,
    voices: Option<(usize, [Entity; 2])>,
    cars: Vec<Scene>,
    bios: Vec<Vec<u8>>,
    portraits: Vec<Vec<u8>>,
    textures: Vram,
    pub row: usize,
    pub field: u32,
    pub heading: u32,
    pub slide: u32,
    pub left: bool,
}
impl Garage {
    pub fn load(disc: &Disc) -> Result<Self> {
        let data = disc.read("UADMD/CARS.DMD")?;
        let dmd = Dmd::parse(&data)?;
        let textures = Vram::from_tms(&disc.read("UADMD/CARS.TMS")?)?;
        ensure!(
            textures.version == dmd.version,
            "garage texture version mismatch"
        );
        let bank = disc.read("SND/UACORE.VAB")?;
        let mut audio = Vec::new();
        let mut cars = Vec::new();
        let mut bios = Vec::new();
        for (i, car) in Vehicle::CARD_ORDER.into_iter().enumerate() {
            let (engine, signature) = car.sound_programs();
            let motor = crate::sfx::Sample::load(&bank, engine, true)?;
            audio.push((
                motor.wave(0.)?,
                crate::sfx::Sample::load(&bank, signature, false)?.wave(0.)?,
                motor.range,
            ));
            cars.push(dmd.garage_vehicle(car as u16)?);
            bios.push(screen::background(
                &disc.read(&format!("UABIOS/SCNDB{:02}B.TIM", i + 1))?,
            )?);
        }
        Ok(Self {
            demo_backdrop: screen::background(&disc.read("UASCREEN/SCRND2.TIM")?)?,
            backdrop: screen::background(&disc.read("UASCREEN/SCRND.TIM")?)?,
            cars,
            audio,
            voices: None,
            bios,
            portraits: dmd.occupant_portraits(&textures)?,
            textures,
            row: 0,
            field: 0,
            heading: 1365,
            slide: 22,
            left: false,
        })
    }
    pub fn voices(&self) -> impl Iterator<Item = Entity> {
        self.voices.iter().flat_map(|(_, voices)| *voices)
    }
    pub fn sound(&mut self, world: &mut World, index: Option<usize>) {
        if self.voices.as_ref().map(|v| v.0) == index {
            return;
        }
        if let Some((_, voices)) = self.voices.take() {
            for entity in voices {
                world.despawn(entity);
            }
        }
        if let Some(i) = index {
            let gain = world
                .get_resource::<crate::driving::Drive>()
                .map_or(1., |d| d.settings.effects_gain());
            let (motor, signature, range) = &self.audio[i];
            let mut audio = world.resource_mut::<Assets<AudioSource>>();
            let engine = audio.add(motor.clone());
            let signature = audio.add(signature.clone());
            let spawn = |world: &mut World, handle, settings| {
                world
                    .spawn((
                        crate::campaign::StageEntity,
                        AudioPlayer::new(handle),
                        settings,
                    ))
                    .id()
            };
            use bevy::audio::Volume;
            self.voices = Some((
                i,
                [
                    spawn(
                        world,
                        engine,
                        PlaybackSettings::LOOP
                            .with_speed(crate::sfx::engine_speed(*range, 30))
                            .with_volume(Volume::Linear(0.28 * gain)),
                    ),
                    spawn(
                        world,
                        signature,
                        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.44 * gain)),
                    ),
                ],
            ));
        }
    }
    pub fn bio(&self, i: usize) -> Vec<u8> {
        self.bios[i].clone()
    }
    pub fn tick(&mut self, n: u32) {
        self.heading = self.heading.wrapping_add(n.wrapping_mul(11)) & 4095;
        self.field = self.field.wrapping_add(n);
        self.slide = self.slide.saturating_add(n).min(22);
    }
    pub fn reset(&mut self) {
        self.row = 0;
        self.field = 0;
        self.heading = 1365;
        self.slide = 22;
        self.left = false;
    }
    pub fn frame(&self, fonts: &Fonts, i: usize, demo: bool) -> Vec<u8> {
        let mut frame = if demo {
            self.demo_backdrop.clone()
        } else {
            self.backdrop.clone()
        };
        let radians = std::f32::consts::TAU / 4096.;
        let side = if self.left { -1. } else { 1. };
        let spin = side * ((self.slide * 16384 / 360) as f32 * radians).sin() * 7680.;
        let model = Quat::from_rotation_z(self.heading as f32 * radians);
        let camera = Quat::from_rotation_x(182. * radians);
        // CarInitDeltas unk6C in screenGetCarsEnum order.
        let heights = [32., 16., 32., 15., 14., 16., 20., 32., 16., 16., 66., 16.];
        let mut draw = |index: usize, x: f32| {
            let origin = Vec3::new(x, 7680., heights[index] * 32. - 4096.);
            crate::ending::draw_model(
                &mut frame,
                &self.cars[index],
                &self.textures,
                self.field,
                |p| camera * (origin + model * p),
            );
        };
        draw(i, side * 7680. - spin);
        if self.slide < 22 {
            draw((i + if self.left { 1 } else { 11 }) % 12, -spin);
        }
        for y in 0..90 {
            for x in 0..64 {
                let at = 4 * ((35 + y) * 320 + 10 + x);
                let pixel = &self.portraits[i][4 * (64 * y + x)..][..4];
                match pixel[3] {
                    0 => {}
                    128 => {
                        for c in 0..3 {
                            frame[at + c] =
                                (u16::from(frame[at + c]) / 2 + u16::from(pixel[c]) / 2) as u8;
                        }
                    }
                    _ => frame[at..at + 4].copy_from_slice(pixel),
                }
            }
        }
        if !demo {
            for (row, label) in ["SELECT", "CAR INFO", "EXIT"].into_iter().enumerate() {
                fonts.at(
                    &mut frame,
                    if row == self.row {
                        Face::Selected
                    } else {
                        Face::Choice
                    },
                    110,
                    40 + 20 * row as i32,
                    label,
                );
            }
            if self.field % 16 >= 8 {
                fonts.centered_at(&mut frame, Face::Control, 42, 30, "Player 1");
            }
            for (x, label, left) in [(14, "<", true), (285, ">", false)] {
                let face = if self.slide == 0 && self.row == 0 && self.left == left {
                    Face::ArrowActive
                } else {
                    Face::Control
                };
                fonts.at(&mut frame, face, x, 215, label);
            }
        }
        let name = Vehicle::CARD_ORDER[i]
            .name()
            .replace('.', "")
            .to_ascii_uppercase();
        fonts.centered(&mut frame, Face::Choice, 215, &name);
        frame
    }
}
