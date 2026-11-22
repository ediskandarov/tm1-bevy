//! Arena driving frontend.
mod additive;
mod campaign;
mod cinema;
mod contest;
mod driving;
mod duel;
mod ending;
mod garage;
mod gpu_timing;
mod music;
mod pause;
mod render;
mod scroll;
mod sfx;
mod smoke;
mod transition;

use anyhow::{Result, ensure};
use bevy::prelude::*;
use tm1_assets::disc::Disc;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let cue = anyhow::Context::context(
        args.next(),
        "usage: tm1-bevy <path-to-disc.cue> [--level arena|warehouse|freeway|city-park|cyburbia|rooftops|arena-5] [--difficulty easy|medium|hard] [--car <vehicle>] [--duel <second-vehicle>] [--screenshot <path.png>]",
    )?;
    let mut flag = args.next();
    let mut menu = flag.is_none();
    let mut garage = false;
    let mut duel_car = None;
    let mut explicit_car = false;
    use tm1_core::drive::{Difficulty, Level, Vehicle};
    let mut difficulty = Difficulty::Medium;
    let mut level = Level::Arena;
    let mut vehicle = Vehicle::YellowJacket;
    while matches!(
        flag.as_ref().and_then(|s| s.to_str()),
        Some("--duel" | "--car" | "--level" | "--menu" | "--garage" | "--difficulty")
    ) {
        if matches!(
            flag.as_ref().and_then(|f| f.to_str()),
            Some("--menu" | "--garage")
        ) {
            garage = flag.as_deref() == Some(std::ffi::OsStr::new("--garage"));
            menu = true;
            flag = args.next();
            continue;
        }
        if flag.as_deref() == Some(std::ffi::OsStr::new("--difficulty")) {
            difficulty = match args.next().as_ref().and_then(|s| s.to_str()) {
                Some("easy") => Difficulty::Easy,
                Some("medium") => Difficulty::Medium,
                Some("hard") => Difficulty::Hard,
                _ => anyhow::bail!("--difficulty requires easy, medium or hard"),
            };
            flag = args.next();
            continue;
        }
        if flag.as_deref() == Some(std::ffi::OsStr::new("--level")) {
            level = match args.next().as_ref().and_then(|s| s.to_str()) {
                Some("warehouse") => Level::Warehouse,
                Some("rooftops") => Level::Rooftops,
                Some("cyburbia") => Level::Cyburbia,
                Some("city-park") => Level::CityPark,
                Some("freeway") => Level::Freeway,
                Some("arena") => Level::Arena,
                Some("arena-5") => Level::Arena5,
                _ => anyhow::bail!(
                    "--level requires arena, warehouse, freeway, city-park, cyburbia, rooftops or arena-5"
                ),
            };
            flag = args.next();
            continue;
        }
        let name = args.next();
        let selected = Vehicle::ROSTER
            .into_iter()
            .find(|v| name.as_deref() == Some(std::ffi::OsStr::new(v.slug())))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--car requires one of {}",
                    Vehicle::ROSTER.map(Vehicle::slug).join(", ")
                )
            })?;
        if flag.as_deref() == Some(std::ffi::OsStr::new("--duel")) {
            duel_car = Some(selected);
        } else {
            explicit_car = true;
            vehicle = selected;
        }
        flag = args.next();
    }
    if flag.as_deref() == Some(std::ffi::OsStr::new("--object-mesh")) {
        flag = args.next();
    }
    if menu && !explicit_car {
        vehicle = Vehicle::SweetTooth;
    }
    let realtime = flag.as_deref() == Some(std::ffi::OsStr::new("--realtime-screenshot"));
    let special_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-special-screenshot"));
    let helicopter_demo =
        flag.as_deref() == Some(std::ffi::OsStr::new("--demo-helicopter-screenshot"));
    ensure!(
        !helicopter_demo
            || (!menu && matches!(level, Level::Arena | Level::Arena5 | Level::Rooftops)),
        "helicopter capture requires Arena, Rooftops or Arena 5"
    );
    let cockpit_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-cockpit-screenshot"));
    ensure!(
        !cockpit_demo || !menu,
        "cockpit capture requires a running encounter"
    );
    let mirror_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-mirror-screenshot"));
    ensure!(
        !mirror_demo || !menu,
        "mirror capture requires a running encounter"
    );
    let pause_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-pause-screenshot"));
    ensure!(
        !pause_demo || !menu,
        "pause capture requires a running encounter"
    );
    let boss_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-boss-screenshot"));
    ensure!(
        !boss_demo || (!menu && level == Level::Rooftops),
        "boss capture requires Rooftops"
    );
    let battle_demo = flag.as_deref() == Some(std::ffi::OsStr::new("--demo-battle-screenshot"));
    ensure!(
        !battle_demo || garage,
        "battle card capture requires --garage"
    );
    let transition_demo =
        flag.as_deref() == Some(std::ffi::OsStr::new("--demo-transition-screenshot"));
    ensure!(
        !transition_demo || garage,
        "transition capture requires --garage"
    );
    let demo = transition_demo
        || battle_demo
        || boss_demo
        || pause_demo
        || mirror_demo
        || cockpit_demo
        || helicopter_demo
        || realtime
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
    if duel_car.is_some() {
        eprintln!(
            "P1: WASD, Space guns, F weapon, E cycle. P2: arrows, Comma guns, Period weapon, L cycle. P or keypad Enter pauses. Enter skips the score, Esc returns to title."
        );
    } else {
        eprintln!(
            "W/S drive/reverse, A/D steer, Shift hard turn, T turbo, Space machine gun, E cycle, F selected weapon, P pause, R restart, C camera, B mirror, Tab overview, V radar, L rivals."
        );
    }
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
    use bevy::audio::AddAudioSource;
    app.add_audio_source::<sfx::pcm::StereoPcm>();
    if let Some(second) = duel_car {
        app.add_systems(Startup, move |world: &mut World| {
            if let Err(e) = duel::start(world, level, [vehicle, second], difficulty, [0; 2]) {
                error!("Loading duel failed: {e:#}");
                world.write_message(AppExit::error());
            } else if pause_demo {
                world.resource_mut::<duel::Duel>().pause = Some(pause::Page::Choices(0));
            }
        });
    } else if !menu {
        let stage = campaign::Stage::from_world(
            app.world(),
            level,
            vehicle,
            campaign::Entry::New(difficulty),
        )?;
        stage.install(app.world_mut());
        if boss_demo {
            app.world_mut()
                .resource_mut::<driving::Drive>()
                .start_with_minion();
        }
        if pause_demo {
            app.world_mut().resource_mut::<driving::Drive>().pause = Some(pause::Page::Choices(0));
        }
        if mirror_demo {
            app.world_mut().resource_mut::<driving::Drive>().mirror = true;
        }
        if cockpit_demo {
            let mut drive = app.world_mut().resource_mut::<driving::Drive>();
            drive.camera.mode = tm1_core::drive::CarView::Cockpit;
        }
        if helicopter_demo {
            let mut drive = app.world_mut().resource_mut::<driving::Drive>();
            drive.helicopter_unlocked = true;
            drive.toggle_helicopter();
        }
        app.add_systems(Startup, (render::setup, music::start, sfx::start));
    } else {
        contest::show(
            app.world_mut(),
            vehicle,
            level,
            difficulty,
            (!garage).then_some(cinema::Clip::Presents),
        )?;
        if garage {
            contest::open_garage(app.world_mut(), battle_demo || transition_demo);
            if transition_demo {
                contest::start_battle(app.world_mut())?;
            }
        }
    }
    app.add_plugins(additive::AdditivePlugin);
    if demo {
        app.add_plugins((
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            gpu_timing::GpuTiming,
        ));
    }
    app.edit_schedule(Update, duel::configure);
    let exit = app
        .add_systems(
            Update,
            contest::update.run_if(resource_exists::<contest::Contest>),
        )
        .add_systems(
            Update,
            (
                driving::advance,
                render::controls,
                render::update,
                render::cockpit,
                render::hit_sheet,
                render::mirror_display,
                render::hud_letters,
                render::hud,
                render::battle_notice,
                pause::draw,
            )
                .chain()
                .run_if(not(resource_exists::<duel::Duel>))
                .run_if(not(resource_exists::<ending::Ending>))
                .run_if(not(resource_exists::<contest::Contest>))
                .run_if(not(resource_exists::<transition::Transition>)),
        )
        .add_systems(
            Update,
            (
                smoke::capture.after(pause::draw),
                sfx::update
                    .after(driving::advance)
                    .run_if(not(resource_exists::<duel::Duel>))
                    .run_if(not(resource_exists::<ending::Ending>))
                    .run_if(not(resource_exists::<contest::Contest>))
                    .run_if(not(resource_exists::<transition::Transition>)),
                music::control
                    .after(driving::advance)
                    .run_if(not(resource_exists::<duel::Duel>))
                    .run_if(not(resource_exists::<ending::Ending>))
                    .run_if(not(resource_exists::<contest::Contest>))
                    .run_if(not(resource_exists::<transition::Transition>)),
            ),
        )
        .add_systems(
            Update,
            campaign::advance
                .run_if(not(resource_exists::<duel::Duel>))
                .run_if(not(resource_exists::<ending::Ending>))
                .run_if(not(resource_exists::<contest::Contest>))
                .run_if(not(resource_exists::<transition::Transition>))
                .after(pause::draw)
                .after(sfx::update)
                .after(music::control)
                .after(smoke::capture),
        )
        .add_systems(
            Update,
            (ending::update, ending::finish)
                .chain()
                .after(campaign::advance)
                .run_if(resource_exists::<ending::Ending>),
        )
        .add_systems(
            Update,
            transition::update
                .after(campaign::advance)
                .after(ending::finish)
                .after(contest::update)
                .run_if(resource_exists::<transition::Transition>),
        )
        .run();
    ensure!(exit.is_success(), "arena exited with {exit:?}");
    Ok(())
}
