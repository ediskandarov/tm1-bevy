//! SHELL.c shellDrawCards and SCREEN.c screenTransitionLevel.
use crate::campaign::{Campaign, Stage, StageEntity};
use anyhow::{Result, ensure};
use bevy::prelude::*;
use tm1_assets::screen::{self, Face, Fonts, HEIGHT, WIDTH};
use tm1_core::drive::{Level, Vehicle};

struct Card {
    pixels: Vec<[u8; 4]>,
    center: [i32; 2],
    angle: i32,
}
#[derive(Component)]
struct Overlay;
#[derive(Component)]
struct CardThrow;
#[derive(Resource)]
pub struct Transition {
    stage: Stage,
    fonts: Fonts,
    trig: tm1_core::math::TrigTables,
    lines: Vec<&'static str>,
    cards: Vec<Card>,
    half_size: [i32; 2],
    deal: u32,
    finish: u32,
    field: u32,
    fraction: u128,
    released: bool,
    letterbox: bool,
    settings: Option<crate::pause::Settings>,
    image: Handle<Image>,
    throw: Vec<Handle<crate::sfx::pcm::StereoPcm>>,
}
fn layout(level: Level) -> (&'static [&'static str], &'static [[i32; 2]], [i32; 2]) {
    match level {
        Level::Arena => (&["'ARENA DUEL'"], &[[160, 100]], [80, 40]),
        Level::Warehouse => (
            &["'WAREHOUSE DISTRICT", "WARFARE'"],
            &[[83, 74], [236, 74], [160, 164]],
            [70, 35],
        ),
        Level::Freeway => (
            &["'FREEWAY", "FREE FOR ALL'"],
            &[[88, 50], [247, 50], [160, 120], [88, 190], [247, 190]],
            [60, 30],
        ),
        Level::CityPark => (
            &["'RIVER PARK", "RUMBLE'"],
            &[
                [86, 50],
                [249, 50],
                [90, 120],
                [245, 120],
                [86, 190],
                [249, 190],
            ],
            [60, 30],
        ),
        Level::Cyburbia => (
            &["'ASSAULT on", "CYBURBIA'"],
            &[
                [59, 60],
                [159, 60],
                [259, 60],
                [104, 120],
                [214, 120],
                [59, 180],
                [159, 180],
                [259, 180],
            ],
            [45, 23],
        ),
        Level::Rooftops => (
            &["'ROOFTOP COMBAT'", "The Final Battle"],
            &[[83, 74], [236, 74], [160, 164]],
            [70, 35],
        ),
        Level::Arena5 => (
            &["'THE FIGHT OF", "YOUR LIFE!'"],
            &[[88, 50], [247, 50], [160, 120], [88, 190], [247, 190]],
            [60, 30],
        ),
    }
}
impl Transition {
    fn intro_fields(fonts: &Fonts, line: &str) -> u32 {
        ((320 + fonts.width(Face::Text, line) / 2 - 151) / 20 + 1) as u32
    }
    fn meet(&self) -> &'static str {
        if self.cards.len() == 1 {
            "Meet Your Opponent"
        } else {
            "Meet Your Opponents"
        }
    }
    fn frame(&self) -> Vec<u8> {
        let mut pixels = [0, 0, 0, 128].repeat(WIDTH * HEIGHT);
        if self.field < self.deal {
            let mut elapsed = self.field;
            for (i, &line) in self.lines.iter().enumerate() {
                let duration = Self::intro_fields(&self.fonts, line);
                let x = if elapsed >= duration {
                    159
                } else {
                    (320 + self.fonts.width(Face::Text, line) / 2 - 20 * elapsed as i32).max(159)
                };
                self.fonts.centered_at(
                    &mut pixels,
                    if i == 4 { Face::Password } else { Face::Text },
                    x,
                    75 + i as i32 * 30,
                    line,
                );
                if elapsed < duration {
                    break;
                }
                elapsed -= duration;
            }
            return pixels;
        }
        let elapsed = self.field - self.deal;
        // AddPrim prepends the settled cards, then the OT draws the moving card last.
        let index = (elapsed / 11) as usize;
        for card in self.cards[..index.min(self.cards.len())].iter().rev() {
            card.draw(
                &self.trig,
                &mut pixels,
                card.center,
                self.half_size,
                360 + card.angle,
            );
        }
        if let Some(card) = self.cards.get(index) {
            let (center, angle) = card.pose(elapsed % 11);
            card.draw(&self.trig, &mut pixels, center, self.half_size, angle);
        }
        if let Some(t) = elapsed.checked_sub(self.cards.len() as u32 * 11 + 2) {
            let x = -(self.fonts.width(Face::Text, self.meet()) / 2) + 20 * t as i32;
            self.fonts
                .centered_at(&mut pixels, Face::Text, x, 220, self.meet());
        }
        pixels
    }
}
impl Card {
    fn pose(&self, tick: u32) -> ([i32; 2], i32) {
        if tick == 10 {
            (self.center, 360 + self.angle)
        } else {
            let mut center = [160, 320];
            let mut angle = 0;
            for _ in 0..=tick {
                for (position, target) in center.iter_mut().zip(self.center) {
                    *position = (85 * *position + 15 * target) / 100;
                }
                angle = (85 * angle + 15 * 1080) / 100;
            }
            (center, angle)
        }
    }
    fn corners(
        trig: &tm1_core::math::TrigTables,
        [cx, cy]: [i32; 2],
        [w, h]: [i32; 2],
        degrees: i32,
    ) -> [[i32; 2]; 4] {
        let [sin, cos] = trig.sin_cos(i64::from(degrees * 4096 / 360)).map(i32::from);
        [[w, -h], [w, h], [-w, -h], [-w, h]].map(|[x, y]| {
            [
                cx + ((cos * x - sin * y) >> 12),
                cy + ((sin * x + cos * y) >> 12),
            ]
        })
    }
    fn draw(
        &self,
        trig: &tm1_core::math::TrigTables,
        out: &mut [u8],
        center: [i32; 2],
        size: [i32; 2],
        degrees: i32,
    ) {
        let xy = Self::corners(trig, center, size, degrees);
        let uv = [[127, 0], [127, 63], [0, 0], [0, 63]];
        for tri in [[0, 1, 2], [1, 3, 2]] {
            screen::raster_triangle(tri.map(|i| xy[i]), tri.map(|i| uv[i]), |at, [u, v]| {
                let color = self.pixels[v * 128 + u];
                if color[3] != 0 {
                    out[at..at + 4].copy_from_slice(&color);
                }
            });
        }
    }
}
pub fn begin(
    world: &mut World,
    stage: Stage,
    advanced: bool,
    letterbox: bool,
    settings_override: Option<crate::pause::Settings>,
) -> Result<()> {
    let disc = &world.resource::<Campaign>().0;
    let level = stage.drive.player.level();
    let (labels, centers, half_size) = layout(level);
    let fonts = Fonts::load(disc)?;
    let trig = tm1_core::math::TrigTables::from_executable(&disc.read("SIPS_600.07")?)
        .ok_or_else(|| anyhow::anyhow!("missing SDK rotation table"))?;
    let mut lines = vec![
        if advanced {
            "CONGRATULATIONS!"
        } else {
            "WELCOME!"
        },
        "Prepare for...",
    ];
    lines.extend_from_slice(labels);
    lines.extend(level.password());
    let mut cards = Vec::new();
    ensure!(
        stage.drive.opponents.len() == centers.len() && centers.len() == stage.card_angles.len(),
        "unexpected opponent card count"
    );
    for ((opponent, &angle), &center) in stage
        .drive
        .opponents
        .iter()
        .zip(&stage.card_angles)
        .zip(centers)
    {
        let index = Vehicle::CARD_ORDER
            .iter()
            .position(|&v| v == opponent.vehicle())
            .ok_or_else(|| anyhow::anyhow!("opponent has no card"))?;
        cards.push(Card {
            pixels: screen::load_card(disc, index)?,
            center,
            angle,
        });
    }
    let deal = lines
        .iter()
        .map(|s| Transition::intro_fields(&fonts, s))
        .sum::<u32>()
        + 1;
    let tones = tm1_assets::sound::program(&disc.read("SND/UACORE.VAB")?, 57)?;
    let volume = settings_override
        .or_else(|| {
            world
                .get_resource::<crate::driving::Drive>()
                .map(|d| d.settings)
        })
        .unwrap_or_default()
        .effects_volume();
    let levels = crate::sfx::pcm::Levels::new([i32::from(volume); 2], 99);
    let mut clips = world.get_resource_or_init::<Assets<crate::sfx::pcm::StereoPcm>>();
    let throw = tones
        .iter()
        .map(|tone| {
            clips.add(crate::sfx::pcm::StereoPcm::one_shot(tone).with_levels(levels.clone()))
        })
        .collect();
    let mut transition = Transition {
        stage,
        fonts,
        trig,
        lines,
        cards,
        half_size,
        deal,
        finish: 0,
        field: 0,
        fraction: 0,
        released: false,
        letterbox,
        settings: settings_override,
        image: default(),
        throw,
    };
    let width = transition.fonts.width(Face::Text, transition.meet());
    transition.finish = deal + centers.len() as u32 * 11 + 2 + ((335 + width + 19) / 20) as u32;
    transition.image = crate::ending::screen_image(world, transition.frame());
    let mut sprite = Sprite::from_image(transition.image.clone());
    if let Ok(window) = world.query::<&Window>().single(world) {
        sprite.custom_size = Some(size(window, letterbox));
    }
    world.spawn((
        StageEntity,
        Overlay,
        sprite,
        Transform::from_xyz(0., 0., 10.),
    ));
    // LOAD_LEVEL interrupts the CD. A battle victory also clears all combat
    // voices and requests the shell engine before the new stage is loaded.
    let audio: Vec<_> = world
        .query_filtered::<(Entity, Has<AudioPlayer<crate::sfx::pcm::StereoPcm>>), (
            With<StageEntity>,
            Or<(
                With<AudioPlayer>,
                With<AudioPlayer<crate::music::CdTrack>>,
                With<AudioPlayer<crate::sfx::pcm::StereoPcm>>,
            )>,
        )>()
        .iter(world)
        .filter_map(|(e, pcm)| (advanced || !pcm).then_some(e))
        .collect();
    for e in audio {
        world.despawn(e);
    }
    if advanced {
        let sounds = world.resource::<crate::sfx::Sounds>();
        sounds.set_screen_volume(volume);
        for engine in sounds.ending_engine() {
            world.spawn((StageEntity, crate::sfx::ShellEngine, engine));
        }
    }
    world.insert_resource(transition);
    Ok(())
}
fn size(window: &Window, letterbox: bool) -> Vec2 {
    let h = window.height();
    if letterbox {
        let h = h.min(window.width() * 0.75);
        Vec2::new(h * 4. / 3., h)
    } else {
        Vec2::new(window.width(), h)
    }
}
pub fn update(world: &mut World) {
    let scripted = world
        .get_resource::<crate::smoke::Capture>()
        .is_some_and(|c| c.demo && !c.realtime);
    let (focused, dim) = if let Ok(window) = world.query::<&Window>().single(world) {
        (
            window.focused,
            size(window, world.resource::<Transition>().letterbox),
        )
    } else {
        return;
    };
    let letterbox = world.resource::<Transition>().letterbox;
    for (mut sprite, overlay) in world
        .query_filtered::<(&mut Sprite, Has<Overlay>), With<StageEntity>>()
        .iter_mut(world)
    {
        if overlay || letterbox {
            sprite.custom_size = Some(dim);
        }
    }
    if !scripted && !focused {
        return;
    }
    let dt = if scripted {
        std::time::Duration::from_millis(50)
    } else {
        world.resource::<Time<Real>>().delta()
    };
    if !world.resource::<Transition>().released {
        // The one-player screenTransitionLevel entry polls GetPadStatus(0),
        // which returns the second port's halfword.
        let mut pads = world.query::<(Entity, &Gamepad)>();
        let first = pads.iter(world).map(|(id, _)| id).min();
        let held = !scripted
            && pads
                .iter(world)
                .filter(|(id, _)| Some(*id) != first)
                .min_by_key(|(id, _)| *id)
                .is_some_and(|(_, pad)| pad.get_pressed().next().is_some());
        if held {
            return;
        }
        world.resource_mut::<Transition>().released = true;
    }
    let mut t = world.resource_mut::<Transition>();
    let prev = t.field;
    t.fraction += dt.as_nanos() * 60;
    t.field = t.field.saturating_add((t.fraction / 1_000_000_000) as u32);
    t.fraction %= 1_000_000_000;
    if t.field == prev {
        return;
    }
    if t.field >= t.finish {
        finish(world);
        return;
    }
    let changed = t.frame();
    let image = t.image.clone();
    let cue = (0..=t.cards.len()).any(|i| {
        let field = t.deal + i as u32 * 11;
        prev < field && field <= t.field
    });
    let throw = cue.then(|| t.throw.clone());
    world
        .resource_mut::<Assets<Image>>()
        .get_mut(&image)
        .unwrap()
        .data = Some(changed);
    if let Some(tones) = throw {
        // Mode 5 restarts this sound ID on each throw, including the final cue.
        let previous: Vec<_> = world
            .query_filtered::<Entity, With<CardThrow>>()
            .iter(world)
            .collect();
        for e in previous {
            world.despawn(e);
        }
        for tone in tones {
            world.spawn((
                StageEntity,
                CardThrow,
                AudioPlayer(tone),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.44)),
            ));
        }
    }
}
fn finish(world: &mut World) {
    let mut t = world.remove_resource::<Transition>().unwrap();
    if let Some(settings) = t.settings {
        t.stage.drive.settings = settings;
        t.stage.drive.player.set_pad_layout(settings.layout);
        if let Some(mut previous) = world.get_resource_mut::<crate::driving::Drive>() {
            previous.settings = settings;
        }
    }
    crate::campaign::replace(world, t.stage);
}
#[cfg(test)]
pub fn complete(world: &mut World) {
    let window = if world.query::<&Window>().iter(world).next().is_none() {
        Some(
            world
                .spawn(Window {
                    focused: true,
                    ..default()
                })
                .id(),
        )
    } else {
        None
    };
    world.init_resource::<ButtonInput<KeyCode>>();
    world.init_resource::<Time<Real>>();
    update(world);
    for _ in 0..400 {
        if !world.contains_resource::<Transition>() {
            break;
        }
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(17));
        update(world);
    }
    assert!(!world.contains_resource::<Transition>());
    if let Some(e) = window {
        world.despawn(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sfx::pcm::{Levels, StereoPcm};
    use bevy::audio::Decodable;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn deal_vertices_match_the_retail_card_packets() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let trig = tm1_core::math::TrigTables::from_executable(&exe).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        for addr in [0x8015e39c, 0x8015e808, 0x80160c74, 0x80160d34] {
            cpu.put(addr, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        let mut seed = cpu.random_state;
        for level in [
            Level::Arena,
            Level::Warehouse,
            Level::Freeway,
            Level::CityPark,
            Level::Cyburbia,
            Level::Rooftops,
            Level::Arena5,
        ] {
            let (_, centers, size) = layout(level);
            cpu.call(0x8013dac8, &[level as u32, centers.len() as u32, 0]);
            for (index, &center) in centers.iter().enumerate() {
                cpu.call(0x8013de04, &[u32::MAX]);
                let random = tm1_core::drive::bios_rand(&mut seed) as i32;
                let card = Card {
                    pixels: vec![],
                    center,
                    angle: if random & 8 == 0 {
                        random & 6
                    } else {
                        -(random & 6)
                    },
                };
                for tick in 0..11 {
                    cpu.call_until(
                        0x8013ded0,
                        &[index as u32, u32::from(tick == 10), 0],
                        0x8013e2ec,
                    );
                    let (center, angle) = card.pose(tick);
                    for (vertex, [x, y]) in Card::corners(&trig, center, size, angle)
                        .into_iter()
                        .enumerate()
                    {
                        let xy = cpu.word(0x8019cc2c + 8 + vertex as u32 * 8);
                        assert_eq!(
                            [x, y],
                            [i32::from(xy as i16), i32::from((xy >> 16) as i16)],
                            "{level:?} card {index} tick {tick} vertex {vertex}"
                        );
                    }
                }
                assert_eq!(cpu.random_state, seed);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn warehouse_card_throws_replace_each_other_after_second_port_release() {
        let mut world = crate::contest::menu_world();
        let first_stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Arena,
            Vehicle::RoadKill,
            crate::campaign::Entry::New(Default::default()),
        )
        .unwrap();
        crate::campaign::replace(&mut world, first_stage);
        let old_engines: Vec<_> = world
            .query_filtered::<Entity, (With<crate::sfx::Engine>, With<AudioPlayer<StereoPcm>>)>()
            .iter(&world)
            .collect();
        let old_music = world
            .query_filtered::<Entity, With<crate::music::ArenaMusic>>()
            .single(&world)
            .unwrap();
        assert_eq!(old_engines.len(), 2);
        let disc = &world.resource::<Campaign>().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let record = 2048 + (0x8016fefc - base) as usize + 6 * 0x88;
        let prog = u16::from_le_bytes(exe[record + 2..record + 4].try_into().unwrap()) as usize;
        let tones =
            tm1_assets::sound::program(&disc.read("SND/UACORE.VAB").unwrap(), prog).unwrap();
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            Vehicle::RoadKill,
            crate::campaign::Entry::New(Default::default()),
        )
        .unwrap();
        let mut settings = crate::pause::Settings::default();
        crate::pause::navigate(
            crate::pause::Page::Audio(1),
            crate::pause::Action::Right,
            &mut settings,
        );
        begin(&mut world, stage, true, false, Some(settings)).unwrap();
        assert!(world.get_entity(old_music).is_err());
        assert!(old_engines.iter().all(|&e| world.get_entity(e).is_err()));
        let revs: Vec<_> = world
            .query_filtered::<Entity, With<crate::sfx::ShellEngine>>()
            .iter(&world)
            .collect();
        assert_eq!(revs.len(), 2);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_nanos(16_666_667));
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        let mut pads = [
            world.spawn(Gamepad::default()).id(),
            world.spawn(Gamepad::default()).id(),
        ];
        pads.sort();
        for id in pads {
            world
                .get_mut::<Gamepad>(id)
                .unwrap()
                .digital_mut()
                .press(GamepadButton::South);
        }
        update(&mut world);
        assert_eq!(world.resource::<Transition>().field, 0);
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .release(GamepadButton::South);
        update(&mut world);
        assert_eq!(world.resource::<Transition>().field, 1);
        let deal = world.resource::<Transition>().deal;
        let mut previous = Vec::new();
        for field in 2..=deal + 33 {
            update(&mut world);
            let mut voices = world.query_filtered::<(Entity, &AudioPlayer<StereoPcm>, &PlaybackSettings), With<CardThrow>>();
            let playing: Vec<_> = voices.iter(&world).collect();
            let entities: Vec<_> = playing.iter().map(|(id, _, _)| *id).collect();
            if field < deal || (field - deal) % 11 != 0 {
                assert_eq!(entities, previous);
                continue;
            }
            assert!(previous.iter().all(|&e| world.get_entity(e).is_err()));
            assert_eq!(playing.len(), tones.len());
            for ((_, player, playback), tone) in playing.into_iter().zip(&tones) {
                assert_eq!(playback.volume, bevy::audio::Volume::Linear(0.44));
                let expected = StereoPcm::one_shot(tone)
                    .with_levels(Levels::new([settings.effects_volume().into(); 2], 99));
                let clip = world
                    .resource::<Assets<StereoPcm>>()
                    .get(&player.0)
                    .unwrap();
                assert!(expected.decoder().eq(clip.decoder()));
            }
            previous = entities;
        }
        complete(&mut world);
        assert!(revs.iter().all(|&e| world.get_entity(e).is_err()));
        assert_eq!(
            world.resource::<crate::driving::Drive>().player.level(),
            Level::Warehouse
        );
        assert!(
            world
                .query_filtered::<Entity, With<crate::music::ArenaMusic>>()
                .single(&world)
                .is_ok()
        );
    }
}
