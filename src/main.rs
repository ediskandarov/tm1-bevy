//! Arena driving frontend.
mod additive;
mod driving;
mod gpu_timing;
mod music;
mod render;
mod sfx;
mod smoke;

use anyhow::{Result, ensure};
use bevy::prelude::*;
use tm1_assets::{disc::Disc, scene::Dmd, texture::Vram};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let cue = anyhow::Context::context(
        args.next(),
        "usage: tm1-bevy <path-to-disc.cue> [--screenshot <path.png>]",
    )?;
    let mut flag = args.next();
    if flag.as_deref() == Some(std::ffi::OsStr::new("--object-mesh")) {
        flag = args.next();
    }
    let realtime = flag.as_deref() == Some(std::ffi::OsStr::new("--realtime-screenshot"));
    let demo = realtime || flag.as_deref() == Some(std::ffi::OsStr::new("--demo-screenshot"));
    let screenshot = match flag {
        None => None,
        Some(flag) if flag == "--screenshot" || demo => Some(std::path::PathBuf::from(
            anyhow::Context::context(args.next(), "missing screenshot path")?,
        )),
        Some(flag) => anyhow::bail!("unknown argument: {flag:?}"),
    };
    let mut capture = smoke::Capture::new(screenshot, demo, realtime);
    if demo
        && !realtime
        && let Some(frame) = args.next()
    {
        capture.demo_frame = frame
            .to_str()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .ok_or_else(|| anyhow::anyhow!("expected a positive demo frame number"))?;
    }
    ensure!(args.next().is_none(), "unexpected extra argument");
    let disc = Disc::open(cue)?;
    let data = disc.read("UADMD/XARENA1.DMD")?;
    let textures = Vram::from_tms(&disc.read("UADMD/XARENA1.TMS")?)?;
    let dmd = Dmd::parse(&data)?;
    ensure!(dmd.version == textures.version, "DMD/TMS version mismatch");
    let mut scene = dmd.extract_kind(1)?;
    let floor = dmd.extract_kind(3)?;
    scene.polygons.extend(floor.polygons);
    for issue in &scene.omitted_nodes {
        eprintln!("Preview omission: {issue}");
    }
    let taxi = tm1_core::drive::Taxi::from_executable(&disc.read("SIPS_600.07")?)
        .ok_or_else(|| anyhow::anyhow!("missing JP SDK rotation table"))?;
    let taxi_mesh = dmd.extract_kind(20)?;
    let opponent_mesh = dmd.extract_kind(50)?;
    let contacts = tm1_core::drive::CollisionScene::from_dmd(
        &data,
        scene.root_id,
        taxi_mesh.root_id,
        opponent_mesh.root_id,
    )
    .ok_or_else(|| anyhow::anyhow!("unsupported arena collision graph"))?;
    let arena = render::Arena::new(scene, taxi_mesh, opponent_mesh, textures)?;
    eprintln!(
        "W/S drive/reverse, A/D steer, Shift hard turn, T turbo, Space machine gun, E cycle, F selected weapon, P pause, R restart, Tab overview."
    );
    let mut app = App::new();
    app.insert_resource(capture)
        .insert_resource(arena)
        .insert_resource(sfx::Sounds::load(&disc)?)
        .insert_resource(music::Music::arena(&disc)?)
        .insert_resource(driving::Drive::new(taxi, contacts))
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.065)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TM1 • Arena driving test".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }));
    app.add_plugins(additive::AdditivePlugin);
    if demo {
        app.add_plugins((
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            gpu_timing::GpuTiming,
        ));
    }
    let exit = app
        .add_systems(Startup, (render::setup, music::start, sfx::start))
        .add_systems(
            Update,
            (
                driving::advance,
                render::controls,
                render::update,
                render::hud,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                smoke::capture,
                sfx::update.after(driving::advance),
                music::control.after(driving::advance),
            ),
        )
        .run();
    ensure!(exit.is_success(), "arena exited with {exit:?}");
    Ok(())
}
