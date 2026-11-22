//! Arena driving frontend.
mod additive;
mod campaign;
mod driving;
mod gpu_timing;
mod music;
mod render;
mod sfx;
mod smoke;

use anyhow::{Result, ensure};
use bevy::prelude::*;
use tm1_assets::disc::Disc;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let cue = anyhow::Context::context(
        args.next(),
        "usage: tm1-bevy <path-to-disc.cue> [--level arena|warehouse|freeway|city-park|cyburbia|rooftops] [--car <vehicle>] [--screenshot <path.png>]",
    )?;
    let mut flag = args.next();
    use tm1_core::drive::{Level, Vehicle};
    let mut level = Level::Arena;
    let mut vehicle = Vehicle::YellowJacket;
    while matches!(
        flag.as_ref().and_then(|s| s.to_str()),
        Some("--car" | "--level")
    ) {
        if flag.as_deref() == Some(std::ffi::OsStr::new("--level")) {
            level = match args.next().as_ref().and_then(|s| s.to_str()) {
                Some("warehouse") => Level::Warehouse,
                Some("rooftops") => Level::Rooftops,
                Some("cyburbia") => Level::Cyburbia,
                Some("city-park") => Level::CityPark,
                Some("freeway") => Level::Freeway,
                Some("arena") => Level::Arena,
                _ => anyhow::bail!(
                    "--level requires arena, warehouse, freeway, city-park, cyburbia or rooftops"
                ),
            };
            flag = args.next();
            continue;
        }
        let name = args.next();
        vehicle = Vehicle::ROSTER
            .into_iter()
            .find(|v| name.as_deref() == Some(std::ffi::OsStr::new(v.slug())))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--car requires one of {}",
                    Vehicle::ROSTER.map(Vehicle::slug).join(", ")
                )
            })?;
        flag = args.next();
    }
    if flag.as_deref() == Some(std::ffi::OsStr::new("--object-mesh")) {
        flag = args.next();
    }
    let realtime = flag.as_deref() == Some(std::ffi::OsStr::new("--realtime-screenshot"));
    let special_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-special-screenshot"));
    let demo = realtime
        || special_demo
        || flag.as_deref() == Some(std::ffi::OsStr::new("--demo-screenshot"));
    let screenshot = match flag {
        None => None,
        Some(flag) if flag == "--screenshot" || demo => Some(std::path::PathBuf::from(
            anyhow::Context::context(args.next(), "missing screenshot path")?,
        )),
        Some(flag) => anyhow::bail!("unknown argument: {flag:?}"),
    };
    let mut capture = smoke::Capture::new(screenshot, demo, realtime);
    capture.special_demo = special_demo;
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
    let stage = campaign::Stage::load(&disc, level, vehicle, None)?;
    eprintln!(
        "W/S drive/reverse, A/D steer, Shift hard turn, T turbo, Space machine gun, E cycle, F selected weapon, P pause, R restart, Tab overview, V radar."
    );
    let mut app = App::new();
    app.insert_resource(capture)
        .insert_resource(campaign::Campaign(disc))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("TM1 • {level:?}"),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }));
    stage.install(app.world_mut());
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
                render::boss_intro,
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
        .add_systems(
            Update,
            campaign::advance
                .after(render::boss_intro)
                .after(sfx::update)
                .after(music::control)
                .after(smoke::capture),
        )
        .run();
    ensure!(exit.is_success(), "arena exited with {exit:?}");
    Ok(())
}
