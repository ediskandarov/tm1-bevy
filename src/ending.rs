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
    Credits,
    Finished,
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
    track: Option<AudioSource>,
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
        let track = track.map(|n| crate::music::track(disc, n)).transpose()?;
        let backdrop = screen::background(&disc.read(&format!("UASCREEN/{background}.TIM"))?)?;
        let credits = crate::scroll::Scroll::load(disc.read("UATEXT/CREDITS.TXT")?, &fonts)?;
        let data = disc.read("UADMD/CARSEND.DMD")?;
        let car = Dmd::parse(&data)?.extract_kind(vehicle as u16)?;
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
            wait_for_release: false,
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
    fn field(&mut self, select: bool, up: bool, down: bool) {
        self.frame += 1;
        match self.phase {
            Phase::Reward => {
                if select || self.frame > 120 {
                    self.waits += 1;
                    self.frame = 0;
                    if self.waits == 5 {
                        self.phase = Phase::Credits;
                        self.wait_for_release = select;
                    }
                }
            }
            Phase::Credits => {
                self.wait_for_release &= select;
                if self
                    .credits
                    .field(select && !self.wait_for_release, up, down)
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
        draw_model(out, &self.car, &self.textures, self.frame, |p| {
            camera * (origin - eye + model * p)
        });
    }
}

pub(crate) fn draw_model(
    out: &mut [u8],
    scene: &Scene,
    textures: &Vram,
    frame: u32,
    transform: impl Fn(Vec3) -> Vec3,
) {
    let mut depth = vec![f32::INFINITY; WIDTH * HEIGHT];
    for poly in &scene.polygons {
        if poly.animation.iter().any(|a| a.selected(frame) != a.child) {
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
    world.spawn((StageEntity, EndingEngine, engine));
    ending.image = spawn_screen(world, EndingScreen, ending.reward.clone());
    world.insert_resource(ending);
    Ok(())
}

pub(crate) fn spawn_screen(
    world: &mut World,
    marker: impl Component,
    pixels: Vec<u8>,
) -> Handle<Image> {
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
    let handle = world.resource_mut::<Assets<Image>>().add(image);
    world.spawn((StageEntity, Camera2d));
    world.spawn((StageEntity, marker, Sprite::from_image(handle.clone())));
    world.insert_resource(ClearColor(Color::BLACK));
    handle
}

pub fn update(
    mut ending: ResMut<Ending>,
    mut images: ResMut<Assets<Image>>,
    mut audio: ResMut<Assets<AudioSource>>,
    mut commands: Commands,
    window: Single<&Window>,
    mut screen: Single<&mut Sprite, With<EndingScreen>>,
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
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
    let pad = pads.iter().next();
    let select =
        keys.pressed(KeyCode::Enter) || pad.is_some_and(|p| p.pressed(GamepadButton::South));
    let up =
        keys.pressed(KeyCode::ArrowUp) || pad.is_some_and(|p| p.pressed(GamepadButton::DPadUp));
    let down =
        keys.pressed(KeyCode::ArrowDown) || pad.is_some_and(|p| p.pressed(GamepadButton::DPadDown));
    while ending.remainder >= field {
        ending.remainder -= field;
        ending.field(select, up, down);
        if ending.phase != before {
            break;
        }
    }
    if ending.phase == Phase::Credits {
        if before == Phase::Reward {
            for entity in &engines {
                commands.entity(entity).despawn();
            }
        }
        let signature_field = match ending.vehicle {
            Vehicle::Outlaw => 721,
            Vehicle::RoadKill => 61,
            _ => 31,
        };
        if before == Phase::Credits
            && first_frame < signature_field
            && ending.frame >= signature_field
        {
            commands.spawn((
                StageEntity,
                EndingEngine,
                sounds.ending_engine(drive.settings.effects_gain()),
            ));
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
                AudioPlayer::new(audio.add(track)),
                PlaybackSettings::LOOP
                    .with_volume(bevy::audio::Volume::Linear(drive.settings.music_gain())),
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
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<crate::additive::Additive>>();
        world.init_resource::<Assets<AudioSource>>();
        world.insert_resource(Campaign(disc));
        crate::campaign::replace(&mut world, stage);
        world.resource_mut::<crate::driving::Drive>().lives = 1;
        begin(&mut world).unwrap();
        let mut ending = world.resource_mut::<Ending>();
        for _ in 0..5 {
            ending.field(true, false, false);
        }
        ending.field(false, false, false);
        ending.field(true, false, false);
        finish(&mut world);
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
    fn reward_waits_and_credits_require_a_fresh_selection() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut ending = Ending::load(&disc, Vehicle::YellowJacket, 1, 0).unwrap();
        for _ in 0..604 {
            ending.field(false, false, false);
        }
        assert_eq!(ending.phase, Phase::Reward);
        ending.field(true, false, false);
        assert_eq!(ending.phase, Phase::Credits);
        assert_eq!(ending.credits.lines.len(), 153);
        ending.field(true, false, false);
        assert_eq!(ending.phase, Phase::Credits);
        ending.field(false, true, false);
        assert_eq!(ending.credits.speed, 2);
        ending.field(true, false, false);
        assert_eq!(ending.phase, Phase::Finished);
    }
}
