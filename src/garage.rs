//! The vehicle carousel in SCREEN.c.
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
    audio: Vec<(Vec<crate::sfx::EngineSample>, AudioSource)>,
    voices: Option<(usize, Vec<Entity>)>,
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
            let (_, signature) = car.sound_programs();
            audio.push((
                crate::sfx::EngineSample::load(&bank, car)?,
                crate::sfx::Sample::load(&bank, signature, false)?.wave()?,
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
        self.voices
            .iter()
            .flat_map(|(_, voices)| voices.iter().copied())
    }
    pub fn sound(&mut self, world: &mut World, index: Option<usize>, gain: f32) {
        if self.voices.as_ref().map(|v| v.0) == index {
            return;
        }
        if let Some((_, voices)) = self.voices.take() {
            for entity in voices {
                world.despawn(entity);
            }
        }
        if let Some(i) = index {
            let (engine, signature) = &self.audio[i];
            use bevy::audio::Volume;
            world.init_resource::<Assets<crate::sfx::pcm::StereoPcm>>();
            let mut voices = Vec::with_capacity(engine.len() + 1);
            for tone in engine {
                let handle = world
                    .resource_mut::<Assets<crate::sfx::pcm::StereoPcm>>()
                    .add(tone.voice(30));
                voices.push(
                    world
                        .spawn((
                            crate::campaign::StageEntity,
                            AudioPlayer(handle),
                            PlaybackSettings::ONCE.with_volume(Volume::Linear(0.28 * gain)),
                        ))
                        .id(),
                );
            }
            let handle = world
                .resource_mut::<Assets<AudioSource>>()
                .add(signature.clone());
            voices.push(
                world
                    .spawn((
                        crate::campaign::StageEntity,
                        AudioPlayer::new(handle),
                        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.44 * gain)),
                    ))
                    .id(),
            );
            self.voices = Some((i, voices));
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
        self.carousel(fonts, i, demo, 0, None, false)
    }
    pub fn join_frame(&self, fonts: &Fonts) -> Vec<u8> {
        let mut frame = self.backdrop.clone();
        if self.field % 16 >= 8 {
            fonts.centered(&mut frame, Face::Text, 129, "PRESS ANY BUTTON");
        }
        frame
    }
    pub fn duel_frame(
        &self,
        fonts: &Fonts,
        car: usize,
        slot: usize,
        locked: Option<usize>,
    ) -> Vec<u8> {
        self.carousel(fonts, car, false, slot, locked, true)
    }
    fn carousel(
        &self,
        fonts: &Fonts,
        i: usize,
        demo: bool,
        slot: usize,
        locked: Option<usize>,
        two_players: bool,
    ) -> Vec<u8> {
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
            let step = if self.left { 1 } else { 11 };
            let mut previous = (i + step) % 12;
            if Some(previous) == locked {
                previous = (previous + step) % 12;
            }
            draw(previous, -spin);
        }
        let centers = [42, 277];
        let mut portrait = |car: usize, slot: usize, w: usize, h: usize| {
            for y in 0..h {
                for x in 0..w {
                    let at = 4 * ((80 - h / 2 + y) * 320 + centers[slot] - w / 2 + x);
                    let pixel = &self.portraits[car][4 * (64 * (y * 90 / h) + x * 64 / w)..][..4];
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
        };
        portrait(i, slot, 64, 90);
        if let Some(other) = locked {
            let fields = self.field.min(8);
            let w = 48 + 2 * (8 >> fields);
            let h = 44 + 2 * (23 >> fields);
            portrait(other, 1 - slot, w as usize, h as usize);
        }
        if !demo {
            for (row, label) in ["SELECT", "CAR INFO", "EXIT"].into_iter().enumerate() {
                fonts.sprite_at(
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
                fonts.sprite_centered_at(
                    &mut frame,
                    Face::Control,
                    centers[slot] as i32,
                    30,
                    ["Player 1", "Player 2"][slot],
                );
            }
            if two_players {
                fonts.sprite_centered_at(
                    &mut frame,
                    Face::Control,
                    centers[1 - slot] as i32,
                    30,
                    ["Player 1", "Player 2"][1 - slot],
                );
            }
            for (x, label, left) in [(14, "<", true), (285, ">", false)] {
                let face = if self.slide == 0 && self.row == 0 && self.left == left {
                    Face::ArrowActive
                } else {
                    Face::Control
                };
                fonts.sprite_at(&mut frame, face, x, 215, label);
            }
        }
        fonts.sprite_centered_at(&mut frame, Face::Choice, 159, 215, fonts.car_name(i));
        frame
    }
}
