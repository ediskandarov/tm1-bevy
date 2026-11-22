//! Single-player JP SCREEN.c reward and scrolling credits.
use crate::campaign::{Campaign, StageEntity};
use anyhow::{Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::time::Duration;
use tm1_assets::{
    disc::Disc,
    scene::{Dmd, Scene},
    screen::{self, Face, Fonts, HEIGHT, WIDTH},
    texture::Vram,
};
use tm1_core::drive::Vehicle;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Phase {
    Reward,
    WaitingForCredits,
    Credits,
    Finished,
}

#[derive(Default, Copy, Clone)]
struct EndingInput {
    select: bool,
    up: bool,
    down: bool,
    primary_held: bool,
    secondary_held: bool,
}

fn ending_pads<'a>(pads: impl Iterator<Item = (Entity, &'a Gamepad)>) -> [Option<&'a Gamepad>; 2] {
    let mut pads = pads.collect::<Vec<_>>();
    pads.sort_unstable_by_key(|(id, _)| *id);
    std::array::from_fn(|slot| pads.get(slot).map(|(_, p)| *p))
}

#[derive(Resource)]
pub struct Ending {
    phase: Phase,
    frame: u32,
    waits: u8,
    remainder: Duration,
    credits: crate::scroll::Scroll,
    wait_for_release: bool,
    fonts: Fonts,
    reward: Vec<u8>,
    backdrop: Vec<u8>,
    car: Scene,
    textures: Vram,
    vehicle: Vehicle,
    height: i32,
    image: Handle<Image>,
    track: Option<crate::music::CdTrack>,
}
impl Ending {
    fn load(disc: &Disc, vehicle: Vehicle, random: u32, height: i32) -> Result<Self> {
        let fonts = Fonts::load(disc)?;
        let mut reward = screen::background(&disc.read("UASCREEN/SCRNCALY.TIM")?)?;
        let draw = random & 3;
        let (prize, password) = [
            ("Infinite Weapons", crate::contest::INFINITE_WEAPONS),
            ("Minion and Friends", crate::contest::MINION_AND_FRIENDS),
            (
                "Arena with 5 Opponents",
                tm1_core::drive::Level::Arena5.password().unwrap(),
            ),
            ("Helicopter View", crate::contest::HELICOPTER),
        ][draw as usize];
        for (y, line) in [
            (70, "CONGRATULATIONS!"),
            (110, "You shall have a "),
            (125, "special password"),
            (175, prize),
        ] {
            fonts.centered(&mut reward, Face::Text, y, line);
        }
        fonts.centered(&mut reward, Face::Password, 205, password);
        let (background, track) = match vehicle {
            Vehicle::Outlaw => ("SCRNCOP", Some(8)),
            Vehicle::RoadKill => ("SCRNROAD", Some(2)),
            _ => ("SCRNLARO", None),
        };
        let track = track
            .map(|n| crate::music::CdTrack::load(disc, n))
            .transpose()?;
        let backdrop = screen::background(&disc.read(&format!("UASCREEN/{background}.TIM"))?)?;
        let credits = crate::scroll::Scroll::load(disc.read("UATEXT/CREDITS.TXT")?, &fonts)?;
        let data = disc.read("UADMD/CARSEND.DMD")?;
        let car = Dmd::parse(&data)?.ending_vehicle(vehicle as u16)?;
        let textures = Vram::from_tms(&disc.read("UADMD/CARSEND.TMS")?)?;
        ensure!(
            car.version == textures.version,
            "CARSEND texture version mismatch"
        );
        Ok(Self {
            phase: Phase::Reward,
            frame: 0,
            waits: 0,
            remainder: Duration::ZERO,
            credits,
            wait_for_release: true,
            fonts,
            reward,
            backdrop,
            car,
            textures,
            vehicle,
            height,
            image: default(),
            track,
        })
    }
    fn field(&mut self, input: EndingInput) {
        match self.phase {
            Phase::Reward => {
                if self.wait_for_release && input.primary_held {
                    return;
                }
                self.wait_for_release = false;
                self.frame += 1;
                if input.select || self.frame > 120 {
                    self.waits += 1;
                    self.frame = 0;
                    if self.waits == 5 {
                        self.phase = if input.secondary_held {
                            Phase::WaitingForCredits
                        } else {
                            Phase::Credits
                        };
                    }
                }
            }
            Phase::WaitingForCredits => {
                if !input.secondary_held {
                    self.phase = Phase::Credits;
                }
            }
            Phase::Credits => {
                self.frame += 1;
                if self.credits.field(input.select, input.up, input.down)
                    != crate::scroll::Outcome::Running
                {
                    self.phase = Phase::Finished;
                }
            }
            Phase::Finished => {}
        }
    }
    fn credits_frame(&self, out: &mut [u8]) {
        out.copy_from_slice(&self.backdrop);
        self.draw_car(out);
        self.credits.draw(out, &self.fonts);
    }
    fn draw_car(&self, out: &mut [u8]) {
        let (eye, angles, spin, start, step) = match self.vehicle {
            Vehicle::Outlaw => (
                Vec3::new(0., -240., 128.),
                Vec3::ZERO,
                Vec3::new(11., 22., 11.),
                -1280.,
                2.,
            ),
            Vehicle::RoadKill => (
                Vec3::new(0., 0., 112.),
                Vec3::new(79., 0., -34.),
                Vec3::ZERO,
                -480.,
                16.,
            ),
            _ => (
                Vec3::new(40., 0., 48.),
                Vec3::new(-102., 0., -22.),
                Vec3::ZERO,
                240.,
                8.,
            ),
        };
        let rotation = |a: Vec3| {
            let r = a * (std::f32::consts::TAU / 4096.);
            Quat::from_euler(EulerRot::YXZ, r.y, r.x, r.z)
        };
        let origin = Vec3::new(0., start + step * self.frame as f32, self.height as f32);
        let camera = rotation(angles).inverse();
        let model = rotation(spin * self.frame as f32);
        draw_model(
            out,
            &self.car,
            &self.textures,
            self.frame,
            model.inverse() * (eye - origin),
            3,
            |p| camera * (origin - eye + model * p),
        );
    }
}

pub(crate) fn draw_model(
    out: &mut [u8],
    scene: &Scene,
    textures: &Vram,
    frame: u32,
    model_eye: Vec3,
    lod_scale: u32,
    transform: impl Fn(Vec3) -> Vec3,
) {
    let mut depth = vec![f32::INFINITY; WIDTH * HEIGHT];
    let mut hidden = vec![false; scene.polygons.len()];
    for lod in scene.lods.iter().flatten() {
        lod.hide_other_branches(model_eye.to_array(), lod_scale, &mut hidden);
    }
    for (poly, hidden) in scene.polygons.iter().zip(hidden) {
        if hidden || poly.animation.iter().any(|a| a.selected(frame) != a.child) {
            continue;
        }
        let mut screen = [Vec2::ZERO; 4];
        let mut z = [0.; 4];
        if poly.vertices.len() > 4 {
            continue;
        }
        for (i, v) in poly.vertices.iter().enumerate() {
            let p = transform(Vec3::from(v.position));
            z[i] = p.y;
            screen[i] = Vec2::new(160. + 256. * p.x / p.y, 120. - 256. * p.z / p.y);
        }
        for &[a, b, c] in &[[0, 1, 2], [2, 1, 3]][..poly.vertices.len().saturating_sub(2)] {
            if z[a].min(z[b]).min(z[c]) < 1. {
                continue;
            }
            let [p, q, r] = [screen[a], screen[b], screen[c]];
            let area = (q - p).perp_dot(r - p);
            if area.abs() < 0.001 {
                continue;
            }
            let lo = p.min(q).min(r).floor().max(Vec2::ZERO).as_ivec2();
            let hi = p.max(q).max(r).ceil().min(Vec2::new(319., 239.)).as_ivec2();
            for y in lo.y..=hi.y {
                for x in lo.x..=hi.x {
                    let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - p;
                    let u = d.perp_dot(r - p) / area;
                    let v = (q - p).perp_dot(d) / area;
                    let w = 1. - u - v;
                    if w.min(u).min(v) < 0. {
                        continue;
                    }
                    let at = y as usize * WIDTH + x as usize;
                    let distance = w * z[a] + u * z[b] + v * z[c];
                    if distance >= depth[at] {
                        continue;
                    }
                    let [va, vb, vc] = [&poly.vertices[a], &poly.vertices[b], &poly.vertices[c]];
                    let mut pixel = [255; 4];
                    if let Some(key) = poly.texture {
                        let uv =
                            Vec2::from(va.uv) * w + Vec2::from(vb.uv) * u + Vec2::from(vc.uv) * v;
                        let Ok(texel) = textures.sample(key, uv.x as u8, uv.y as u8) else {
                            continue;
                        };
                        if texel[3] == 0 {
                            continue;
                        }
                        pixel = texel;
                    }
                    for i in 0..3 {
                        pixel[i] = (f32::from(pixel[i])
                            * (va.color[i] * w + vb.color[i] * u + vc.color[i] * v))
                            as u8;
                    }
                    out[at * 4..at * 4 + 4].copy_from_slice(&pixel);
                    depth[at] = distance;
                }
            }
        }
    }
}

#[derive(Component)]
pub(crate) struct EndingScreen;
#[derive(Component)]
pub(crate) struct EndingEngine;
pub fn begin(world: &mut World) -> Result<()> {
    let mut pads = world.query::<(Entity, &Gamepad)>();
    if ending_pads(pads.iter(world))[1].is_some_and(|pad| pad.get_pressed().next().is_some()) {
        return Ok(());
    }
    let random = world
        .resource_mut::<crate::driving::Drive>()
        .player
        .random();
    let player = &world.resource::<crate::driving::Drive>().player;
    let mut ending = Ending::load(
        &world.resource::<Campaign>().0,
        player.vehicle(),
        random,
        i32::from_le_bytes(player.state()[0x138 + 0x6c..0x138 + 0x70].try_into()?),
    )?;
    crate::campaign::clear_stage(world);
    let gain = world
        .resource::<crate::driving::Drive>()
        .settings
        .effects_gain();
    let engine = world.resource::<crate::sfx::Sounds>().ending_engine(gain);
    for tone in engine {
        world.spawn((StageEntity, EndingEngine, tone));
    }
    ending.image = spawn_screen(world, EndingScreen, ending.reward.clone());
    world.insert_resource(ending);
    Ok(())
}

pub(crate) fn spawn_screen(
    world: &mut World,
    marker: impl Component,
    pixels: Vec<u8>,
) -> Handle<Image> {
    let handle = screen_image(world, pixels);
    world.spawn((StageEntity, Camera2d));
    world.spawn((StageEntity, marker, Sprite::from_image(handle.clone())));
    world.insert_resource(ClearColor(Color::BLACK));
    handle
}

pub(crate) fn screen_image(world: &mut World, pixels: Vec<u8>) -> Handle<Image> {
    let mut image = Image::new(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    world.resource_mut::<Assets<Image>>().add(image)
}

pub fn update(
    mut ending: ResMut<Ending>,
    mut images: ResMut<Assets<Image>>,
    mut audio: ResMut<Assets<crate::music::CdTrack>>,
    mut commands: Commands,
    window: Single<&Window>,
    mut screen: Single<&mut Sprite, With<EndingScreen>>,
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<(Entity, &Gamepad)>,
    drive: Res<crate::driving::Drive>,
    sounds: Res<crate::sfx::Sounds>,
    engines: Query<Entity, With<EndingEngine>>,
    sinks: Query<&AudioSink>,
    capture: Res<crate::smoke::Capture>,
) {
    let h = window.height().min(window.width() * 0.75);
    screen.custom_size = Some(Vec2::new(h * 4. / 3., h));
    let paused = !window.focused && !capture.demo;
    for sink in &sinks {
        use bevy::audio::AudioSinkPlayback;
        if paused {
            sink.pause()
        } else if sink.is_paused() {
            sink.play()
        }
    }
    if paused {
        return;
    }
    let before = ending.phase;
    let first_frame = ending.frame;
    ending.remainder += if capture.demo && !capture.realtime {
        Duration::from_millis(50)
    } else {
        time.delta()
    };
    let field = Duration::from_nanos(16_666_667);
    let [pad, secondary] = ending_pads(pads.iter());
    let select = keys.any_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::KeyP])
        || pad.is_some_and(|p| {
            p.any_pressed([
                GamepadButton::South,
                GamepadButton::Start,
                GamepadButton::Select,
            ])
        });
    let input = EndingInput {
        select,
        up: keys.pressed(KeyCode::ArrowUp) || pad.is_some_and(|p| p.pressed(GamepadButton::DPadUp)),
        down: keys.pressed(KeyCode::ArrowDown)
            || pad.is_some_and(|p| p.pressed(GamepadButton::DPadDown)),
        primary_held: crate::pause::campaign_pad_held(&keys, pad),
        secondary_held: secondary.is_some_and(|p| p.get_pressed().next().is_some()),
    };
    while ending.remainder >= field {
        ending.remainder -= field;
        ending.field(input);
        if ending.phase != before {
            break;
        }
    }
    if before == Phase::Reward && ending.phase != Phase::Reward {
        for entity in &engines {
            commands.entity(entity).despawn();
        }
    }
    if ending.phase == Phase::Credits {
        let signature_field = match ending.vehicle {
            Vehicle::Outlaw => 721,
            Vehicle::RoadKill => 61,
            _ => 31,
        };
        if before == Phase::Credits
            && first_frame < signature_field
            && ending.frame >= signature_field
        {
            for tone in sounds.ending_engine(drive.settings.effects_gain()) {
                commands.spawn((StageEntity, EndingEngine, tone));
            }
            commands.spawn((
                StageEntity,
                sounds.signature(),
                PlaybackSettings::DESPAWN
                    .with_volume(bevy::audio::Volume::Linear(drive.settings.effects_gain())),
            ));
        }
        if let Some(track) = ending.track.take() {
            commands.spawn((
                StageEntity,
                AudioPlayer(audio.add(track)),
                PlaybackSettings::ONCE
                    .with_volume(bevy::audio::Volume::Linear(drive.settings.cd_gain())),
            ));
        }
        if let Some(data) = images
            .get_mut(&ending.image)
            .and_then(|i| i.into_inner().data.as_mut())
        {
            ending.credits_frame(data);
        }
    }
}

pub fn finish(world: &mut World) {
    let ending = world.resource::<Ending>();
    if ending.phase != Phase::Finished {
        return;
    }
    world.remove_resource::<Ending>();
    if world
        .resource::<crate::driving::Drive>()
        .player
        .difficulty()
        == tm1_core::drive::Difficulty::Hard
    {
        crate::campaign::advance_to(world, tm1_core::drive::Level::Arena5);
    } else if let Err(e) = crate::contest::open(world) {
        error!("Opening contest screen failed: {e:#}");
        world.write_message(AppExit::error());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn ending_music_matches_the_disc_for_each_background() {
        use bevy::audio::Decodable;
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        for (vehicle, number) in [(Vehicle::Outlaw, 8), (Vehicle::RoadKill, 2)] {
            let ending = Ending::load(&disc, vehicle, 0, 0).unwrap();
            let mut decoder = ending.track.unwrap().decoder();
            let pcm = disc.audio_track(number).unwrap();
            for bytes in pcm[150 * 2352..].chunks_exact(2).take(4096) {
                assert_eq!(
                    decoder.next(),
                    Some(f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.)
                );
            }
        }
        assert!(
            Ending::load(&disc, Vehicle::YellowJacket, 0, 0)
                .unwrap()
                .track
                .is_none()
        );
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn hard_credits_continue_with_remaining_lives() {
        use crate::campaign::{Entry, Stage};
        use tm1_core::drive::{Difficulty, Level};
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let stage = Stage::load(
            &disc,
            Level::Rooftops,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Hard),
        )
        .unwrap();
        let mut world = World::new();
        world.init_resource::<Time<Real>>();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<crate::texture_blend::TextureBlend>>();
        world.init_resource::<Assets<AudioSource>>();
        world.insert_resource(Campaign(disc));
        crate::campaign::replace(&mut world, stage);
        world.resource_mut::<crate::driving::Drive>().lives = 1;
        use bevy::ecs::system::RunSystemOnce;
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<Assets<crate::music::CdTrack>>();
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world.spawn(Window {
            focused: true,
            ..default()
        });
        let mut pads = [
            world.spawn(Gamepad::default()).id(),
            world.spawn(Gamepad::default()).id(),
        ];
        pads.sort();
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .press(GamepadButton::DPadLeft);
        begin(&mut world).unwrap();
        assert!(!world.contains_resource::<Ending>());
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .release(GamepadButton::DPadLeft);
        begin(&mut world).unwrap();
        let tick = |world: &mut World, fields| {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_nanos(16_666_667) * fields);
            world.run_system_once(update).unwrap();
        };
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        tick(&mut world, 8);
        assert_eq!(world.resource::<Ending>().frame, 0);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Space);
        tick(&mut world, 1);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .press(GamepadButton::Start);
        tick(&mut world, 5);
        assert_eq!(world.resource::<Ending>().phase, Phase::WaitingForCredits);
        tick(&mut world, 5);
        assert_eq!(world.resource::<Ending>().phase, Phase::WaitingForCredits);
        world
            .get_mut::<Gamepad>(pads[1])
            .unwrap()
            .digital_mut()
            .release(GamepadButton::Start);
        tick(&mut world, 1);
        assert_eq!(world.resource::<Ending>().phase, Phase::Credits);
        tick(&mut world, 1);
        assert_eq!(world.resource::<Ending>().phase, Phase::Finished);
        finish(&mut world);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Space);
        crate::transition::complete(&mut world);
        let drive = world.resource::<crate::driving::Drive>();
        assert_eq!(drive.player.level(), Level::Arena5);
        assert_eq!(drive.player.difficulty(), Difficulty::Hard);
        assert_eq!(drive.player.vehicle(), Vehicle::RoadKill);
        assert_eq!(drive.opponents.len(), 5);
        assert_eq!(drive.lives, 1);
        assert!(!world.contains_resource::<Ending>());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn reward_times_out_and_credits_read_the_primary_pad() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut ending = Ending::load(&disc, Vehicle::YellowJacket, 1, 0).unwrap();
        for _ in 0..604 {
            ending.field(default());
        }
        assert_eq!(ending.phase, Phase::Reward);
        ending.field(EndingInput {
            select: true,
            ..default()
        });
        assert_eq!(ending.phase, Phase::Credits);
        assert_eq!(ending.credits.lines.len(), 153);
        let mut pixels = ending.backdrop.clone();
        ending.draw_car(&mut pixels);
        assert_ne!(pixels.as_slice(), ending.backdrop.as_slice());
        ending.field(EndingInput {
            up: true,
            ..default()
        });
        assert_eq!(ending.credits.speed, 2);
        ending.field(EndingInput {
            select: true,
            ..default()
        });
        assert_eq!(ending.phase, Phase::Finished);
    }
}
