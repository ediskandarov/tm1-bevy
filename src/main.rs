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
        "usage: tm1-bevy <path-to-disc.cue> [--level arena|warehouse|freeway|city-park] [--car <vehicle>] [--screenshot <path.png>]",
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
                Some("city-park") => Level::CityPark,
                Some("freeway") => Level::Freeway,
                Some("arena") => Level::Arena,
                _ => anyhow::bail!("--level requires arena, warehouse, freeway or city-park"),
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
    let stage = level.database(vehicle);
    let data = disc.read(&format!("UADMD/{stage}.DMD"))?;
    let textures = Vram::from_tms(&disc.read(&format!("UADMD/{stage}.TMS"))?)?;
    let dmd = Dmd::parse(&data)?;
    ensure!(dmd.version == textures.version, "DMD/TMS version mismatch");
    let mut scene = dmd.extract_kind(1)?;
    if level == Level::Arena {
        scene.polygons.extend(dmd.extract_kind(3)?.polygons);
    }
    for issue in &scene.omitted_nodes {
        eprintln!("Preview omission: {issue}");
    }
    let mut seed = 1;
    let opponent_kinds = level.opponents(
        vehicle,
        &mut tm1_core::drive::ArenaDeck::default(),
        &mut seed,
    );
    let player =
        tm1_core::drive::PlayerVehicle::seeded(&disc.read("SIPS_600.07")?, vehicle, level, seed)
            .ok_or_else(|| anyhow::anyhow!("missing JP SDK rotation table"))?;
    let player_mesh = dmd.vehicle_bodies(vehicle as u16)?;
    let opponent_mesh = opponent_kinds
        .iter()
        .map(|&kind| dmd.vehicle_bodies(kind as u16))
        .collect::<Result<Vec<_>>>()?;
    let contacts = tm1_core::drive::CollisionScene::for_opponents(
        &data,
        scene.root_id,
        player_mesh[0].root_id,
        &opponent_mesh
            .iter()
            .map(|variants| variants[0].root_id)
            .collect::<Vec<_>>(),
    )
    .ok_or_else(|| anyhow::anyhow!("unsupported arena collision graph"))?;
    let arena = render::Arena::new(
        scene,
        (level != Level::Arena)
            .then(|| dmd.extract_kind(2))
            .transpose()?,
        player_mesh,
        opponent_mesh,
        [
            Some(dmd.extract_kind(201)?),
            dmd.roots
                .iter()
                .any(|r| r.1 == 312)
                .then(|| dmd.extract_kind(312))
                .transpose()?,
        ],
        dmd.explosion_frames()?,
        dmd.contrail_frames()?,
        dmd.radar_arrows()?,
        textures,
    )?;
    eprintln!(
        "W/S drive/reverse, A/D steer, Shift hard turn, T turbo, Space machine gun, E cycle, F selected weapon, P pause, R restart, Tab overview, V radar."
    );
    let mut app = App::new();
    app.insert_resource(capture)
        .insert_resource(arena)
        .insert_resource(sfx::Sounds::load(&disc, vehicle, &opponent_kinds)?)
        .insert_resource(music::Music::level(&disc, level)?)
        .insert_resource(driving::Drive::new(player, contacts, &opponent_kinds))
        .insert_resource(ClearColor(match level {
            Level::Arena => Color::srgb(0.035, 0.045, 0.065),
            Level::CityPark => Color::srgb_u8(75, 60, 30),
            Level::Freeway => Color::srgb_u8(60, 45, 20),
            Level::Warehouse => Color::srgb_u8(75, 70, 55),
        }))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("TM1 • {level:?}"),
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
