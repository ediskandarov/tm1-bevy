//! Single-player JP SCREEN.c reward and scrolling credits.
use crate::campaign::{Campaign, Stage, StageEntity};
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
use tm1_core::drive::{Level, Vehicle};

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
    credit_y: i32,
    speed: u8,
    tick: u8,
    key_delay: u8,
    wait_for_release: bool,
    fonts: Fonts,
    reward: Vec<u8>,
    backdrop: Vec<u8>,
    lines: Vec<String>,
    car: Scene,
    textures: Vram,
    vehicle: Vehicle,
    height: i32,
    image: Handle<Image>,
    track: Option<AudioSource>,
}
impl Ending {
    fn load(disc: &Disc, vehicle: Vehicle, seed: u32, height: i32) -> Result<Self> {
        let fonts = Fonts::load(disc)?;
        let mut reward = screen::background(&disc.read("UASCREEN/SCRNCALY.TIM")?)?;
        let draw = (seed.wrapping_mul(1103515245).wrapping_add(12345) >> 16) & 3;
        let (prize, password) = [
            ("Infinite Weapons", "< =@@"),
            ("Minion and Friends", "<>@=<"),
            ("Arena with 5 Opponents", "=<@=="),
            ("Helicopter View", "@@<> "),
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
        let credits = String::from_utf8(disc.read("UATEXT/CREDITS.TXT")?)?;
        let mut lines = credits.split("\r\n");
        let count = lines
            .next()
            .and_then(|s| s.strip_suffix(" C"))
            .ok_or_else(|| anyhow::anyhow!("invalid centered credits header"))?
            .parse::<usize>()?;
        let lines = lines
            .filter(|s| !s.starts_with("@S"))
            .take(count)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        ensure!(lines.len() == count, "incomplete disc credits");
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
            credit_y: 240 + fonts.line_height(),
            speed: 3,
            tick: 0,
            key_delay: 0,
            wait_for_release: false,
            fonts,
            reward,
            backdrop,
            lines,
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
                if self.speed < 6 && self.tick >= self.speed {
                    self.credit_y -= 1;
                    self.tick = 0;
                }
                self.tick = self.tick.saturating_add(1);
                if self.key_delay == 0 && (up || down) {
                    self.speed = if up {
                        self.speed.saturating_sub(1).max(1)
                    } else {
                        (self.speed + 1).min(6)
                    };
                    self.key_delay = 30;
                } else if (select && !self.wait_for_release)
                    || self.credit_y
                        + (self.lines.len() as i32 - 1) * (self.fonts.line_height() + 10)
                        < 0
                {
                    self.phase = Phase::Finished;
                } else {
                    self.key_delay = self.key_delay.saturating_sub(1);
                }
            }
            Phase::Finished => {}
        }
    }
    fn credits_frame(&self, out: &mut [u8]) {
        out.copy_from_slice(&self.backdrop);
        self.draw_car(out);
        let spacing = self.fonts.line_height() + 10;
        for (i, line) in self.lines.iter().enumerate() {
            let y = self.credit_y + i as i32 * spacing;
            if (0..240 + spacing).contains(&y) {
                self.fonts.centered(out, Face::Text, y, line);
            }
        }
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
        let mut depth = vec![f32::INFINITY; WIDTH * HEIGHT];
        for poly in &self.car.polygons {
            if poly
                .animation
                .iter()
                .any(|a| a.selected(self.frame) != a.child)
            {
                continue;
            }
            let mut screen = [Vec2::ZERO; 4];
            let mut z = [0.; 4];
            if poly.vertices.len() > 4 {
                continue;
            }
            for (i, v) in poly.vertices.iter().enumerate() {
                let p = camera * (origin - eye + model * Vec3::from(v.position));
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
                        let [va, vb, vc] =
                            [&poly.vertices[a], &poly.vertices[b], &poly.vertices[c]];
                        let mut pixel = [255; 4];
                        if let Some(key) = poly.texture {
                            let uv = Vec2::from(va.uv) * w
                                + Vec2::from(vb.uv) * u
                                + Vec2::from(vc.uv) * v;
                            let Ok(texel) = self.textures.sample(key, uv.x as u8, uv.y as u8)
                            else {
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
}

#[derive(Component)]
pub(crate) struct EndingScreen;
#[derive(Component)]
pub(crate) struct EndingEngine;
pub fn begin(world: &mut World) -> Result<()> {
    let player = &world.resource::<crate::driving::Drive>().player;
    let mut ending = Ending::load(
        &world.resource::<Campaign>().0,
        player.vehicle(),
        player.rng_state(),
        i32::from_le_bytes(player.state()[0x138 + 0x6c..0x138 + 0x70].try_into()?),
    )?;
    crate::campaign::clear_stage(world);
    let engine = world.resource::<crate::sfx::Sounds>().ending_engine();
    world.spawn((StageEntity, EndingEngine, engine));
    let mut image = Image::new(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        ending.reward.clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    ending.image = world.resource_mut::<Assets<Image>>().add(image);
    world.spawn((StageEntity, Camera2d));
    world.spawn((
        StageEntity,
        EndingScreen,
        Sprite::from_image(ending.image.clone()),
    ));
    world.insert_resource(ClearColor(Color::BLACK));
    world.insert_resource(ending);
    Ok(())
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
            commands.spawn((StageEntity, EndingEngine, sounds.ending_engine()));
            commands.spawn((StageEntity, sounds.signature(), PlaybackSettings::DESPAWN));
        }
        if let Some(track) = ending.track.take() {
            commands.spawn((
                StageEntity,
                AudioPlayer::new(audio.add(track)),
                PlaybackSettings::LOOP,
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
    let result = Stage::load(
        &world.resource::<Campaign>().0,
        Level::Arena,
        ending.vehicle,
        None,
    );
    match result {
        Ok(stage) => {
            world.remove_resource::<Ending>();
            crate::campaign::replace(world, stage);
        }
        Err(e) => {
            error!("Starting a new contest failed: {e:#}");
            world.write_message(AppExit::error());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ending.lines.len(), 153);
        ending.field(true, false, false);
        assert_eq!(ending.phase, Phase::Credits);
        ending.field(false, true, false);
        assert_eq!(ending.speed, 2);
        ending.field(true, false, false);
        assert_eq!(ending.phase, Phase::Finished);
    }
}
