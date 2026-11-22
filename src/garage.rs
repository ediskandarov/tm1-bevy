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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GarageSound {
    Engine(Vehicle),
    Vehicle(usize),
}

pub struct Garage {
    backdrop: Vec<u8>,
    demo_backdrop: Vec<u8>,
    audio: Vec<(Vec<crate::sfx::EngineSample>, AudioSource)>,
    boss_engine: Vec<crate::sfx::EngineSample>,
    voices: Option<(GarageSound, Vec<Entity>)>,
    cars: Vec<Scene>,
    bios: Vec<Vec<u8>>,
    portraits: Vec<Vec<u8>>,
    textures: Vram,
    trig: tm1_core::math::TrigTables,
    pub row: usize,
    pub field: u32,
    portrait_fields: u32,
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
            boss_engine: crate::sfx::EngineSample::load(&bank, Vehicle::Minion)?,
            voices: None,
            bios,
            portraits: dmd.occupant_portraits(&textures)?,
            textures,
            trig: tm1_core::math::TrigTables::from_executable(&disc.read("SIPS_600.07")?)
                .ok_or_else(|| anyhow::anyhow!("invalid showroom rotation tables"))?,
            row: 0,
            field: 0,
            portrait_fields: 0,
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
    pub fn sound(&mut self, world: &mut World, sound: Option<GarageSound>, gain: f32) {
        if self.voices.as_ref().map(|v| v.0) == sound {
            return;
        }
        if let Some((_, voices)) = self.voices.take() {
            for entity in voices {
                world.despawn(entity);
            }
        }
        if let Some(sound) = sound {
            let engine = match sound {
                GarageSound::Engine(Vehicle::Minion) => &self.boss_engine,
                GarageSound::Engine(vehicle) => &self.audio[vehicle.card_index()].0,
                GarageSound::Vehicle(index) => &self.audio[index].0,
            };
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
            if let GarageSound::Vehicle(index) = sound {
                let handle = world
                    .resource_mut::<Assets<AudioSource>>()
                    .add(self.audio[index].1.clone());
                voices.push(
                    world
                        .spawn((
                            crate::campaign::StageEntity,
                            AudioPlayer::new(handle),
                            PlaybackSettings::ONCE.with_volume(Volume::Linear(0.44 * gain)),
                        ))
                        .id(),
                );
            }
            self.voices = Some((sound, voices));
        }
    }
    pub fn bio(&self, i: usize) -> Vec<u8> {
        self.bios[i].clone()
    }
    pub fn tick(&mut self, n: u32) {
        self.heading = self.heading.wrapping_add(n.wrapping_mul(11)) & 4095;
        self.field = self.field.wrapping_add(n);
        self.slide = self.slide.saturating_add(n).min(22);
        self.portrait_fields = self.portrait_fields.saturating_add(n).min(8);
    }
    pub fn lock_portrait(&mut self) {
        self.portrait_fields = 0;
    }
    pub fn reset(&mut self) {
        self.portrait_fields = 0;
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
        // The first field draws with an empty mask before loading 0xff00.
        if self.field.saturating_sub(2) % 16 >= 8 {
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
    fn display_cars(&self, index: usize, locked: Option<usize>) -> [Option<(usize, [i32; 3])>; 2] {
        let side = if self.left { -1 } else { 1 };
        let spin = side * ((self.trig.sine((self.slide * 16384 / 360) as i32) * 30) >> 4);
        // CarInitDeltas unk6C in screenGetCarsEnum order.
        let heights = [32, 16, 32, 15, 14, 16, 20, 32, 16, 16, 66, 16];
        let position = |index: usize, x| (index, [x, 7680, heights[index] * 32 - 4096]);
        let previous = (self.slide < 22).then(|| {
            let step = if self.left { 1 } else { 11 };
            let mut previous = (index + step) % 12;
            if Some(previous) == locked {
                previous = (previous + step) % 12;
            }
            position(previous, -spin)
        });
        [Some(position(index, side * 7680 - spin)), previous]
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
        let model = self.trig.rotation_yxz([0, 0, self.heading as i16]);
        let camera = self.trig.rotation_yxz([182, 0, 0]);
        for (index, relative) in self.display_cars(i, locked).into_iter().flatten() {
            crate::ending::draw_model(
                &mut frame,
                &self.cars[index],
                &self.textures,
                self.field,
                crate::ending::model_transform(&camera, &model, relative),
                8,
            );
        }
        let centers = [42, 277];
        let mut portrait = |car: usize, slot: usize, w: usize, h: usize| {
            for y in 0..h {
                for x in 0..w {
                    let at = 4 * ((35 + y) * 320 + centers[slot] - w / 2 + x);
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
            let fields = self.portrait_fields;
            let w = 48 + 2 * (8 >> fields);
            // Only the lower two Y coordinates approach +22; the upper edge stays at -45.
            let h = 45 + 22 + (23 >> fields);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn showroom_submissions_follow_the_jp_carousel() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut garage = Garage::load(&disc).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        // Supply a writable CS for each lookup and observe submissions to hierProcOne.
        cpu.put(0x8014f98c, &[8, 0, 224, 3, 1, 128, 2, 60]);
        for address in [0x8010a774, 0x800fe81c] {
            cpu.put(address, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        let cs = 0x80010000;
        for (index, car) in Vehicle::CARD_ORDER.into_iter().enumerate() {
            for left in [false, true] {
                garage.reset();
                garage.left = left;
                garage.slide = 0;
                let locked = (index + if left { 1 } else { 11 }) % 12;
                cpu.put(0x8018c584, &1365u32.to_le_bytes());
                cpu.put(0x8018c0f0, &0u16.to_le_bytes());
                cpu.put(0x8018c580, &0u32.to_le_bytes());
                for _ in 0..24 {
                    let mut seen = Vec::new();
                    let mut kind = 0;
                    cpu.call_observed(
                        0x8013ba60,
                        &[0, car as u32, u32::from(left), locked as u32],
                        0x80000080,
                        |c| {
                            if c.pc() == 0x8014f98c {
                                kind = c.r[4];
                            }
                            if c.pc() == 0x800fe81c {
                                let xyz = [
                                    c.word(cs + 0x5c) as i32,
                                    c.word(cs + 0x60) as i32 + 7680,
                                    c.word(cs + 0x64) as i32 - 4096,
                                ];
                                assert_eq!(c.word(cs + 0x58) as u16, garage.heading as u16);
                                let model = garage.trig.rotation_yxz([0, 0, garage.heading as i16]);
                                for (j, &expected) in model.rotation.iter().flatten().enumerate() {
                                    let p = cs + 0x14 + j as u32 * 2;
                                    assert_eq!((c.word(p & !3) >> (8 * (p & 3))) as i16, expected);
                                }
                                seen.push((kind, xyz));
                            }
                        },
                    );
                    let expected: Vec<_> = garage
                        .display_cars(index, Some(locked))
                        .into_iter()
                        .flatten()
                        .map(|(i, pos)| (Vehicle::CARD_ORDER[i] as u32, pos))
                        .collect();
                    assert_eq!(seen, expected, "{car:?} left={left} slide={}", garage.slide);
                    garage.tick(1);
                }
            }
        }
    }
}
