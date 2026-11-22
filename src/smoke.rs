use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use std::path::PathBuf;

#[derive(Resource)]
pub struct Capture {
    path: Option<PathBuf>,
    frames: u32,
    requested: Option<u32>,
    pub demo: bool,
    pub realtime: bool,
    pub elapsed: std::time::Duration,
    pub steps: u64,
}
impl Capture {
    pub fn new(path: Option<PathBuf>, demo: bool, realtime: bool) -> Self {
        Self {
            path,
            demo,
            realtime,
            elapsed: Default::default(),
            steps: 0,
            frames: 0,
            requested: None,
        }
    }
}

pub fn capture(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    sinks: Query<&AudioSink>,
    pads: Query<&Gamepad>,
    engine: Query<&AudioSink, With<crate::sfx::Engine>>,
    music: Query<&AudioSink, With<crate::music::ArenaMusic>>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = capture.path.clone() else {
        return;
    };
    capture.frames += 1;
    let ready = if capture.realtime {
        capture.elapsed.as_secs() >= 15
    } else {
        capture.frames == 120
    };
    if ready && capture.requested.is_none() {
        capture.requested = Some(capture.frames);
        if capture.demo {
            eprintln!(
                "Active audio sinks: {}, connected gamepads: {}",
                sinks.iter().count(),
                pads.iter().count()
            );
            if let (Ok(engine), Ok(music)) = (engine.single(), music.single()) {
                eprintln!(
                    "Engine playback {:?}, music playback {:?}",
                    engine.position(),
                    music.position()
                );
            }
            if capture.realtime {
                eprintln!(
                    "Wall-clock playback: {:.4}s, {} simulation steps, remainder {:.4}ms",
                    capture.elapsed.as_secs_f64(),
                    capture.steps,
                    capture.elapsed.as_secs_f64() * 1000. - capture.steps as f64 * 50.
                );
            }
            for d in diagnostics.iter() {
                if let Some(ms) = d.average() {
                    eprintln!("Render diagnostic {}: {ms:.4} {}", d.path(), d.suffix);
                }
            }
            if let Ok(usage) = std::process::Command::new("/bin/ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
            {
                if usage.status.success() {
                    eprintln!("RSS KiB: {}", String::from_utf8_lossy(&usage.stdout).trim());
                }
            }
        }
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let result = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|error| format!("{error:?}"))
                    .and_then(|image| image.save(&path).map_err(|error| error.to_string()));
                match result {
                    Ok(()) => {
                        eprintln!("Saved GPU screenshot: {}", path.display());
                        exit.write(AppExit::Success);
                    }
                    Err(error) => {
                        eprintln!("Screenshot failed: {error}");
                        exit.write(AppExit::error());
                    }
                }
            },
        );
    } else if capture
        .requested
        .is_some_and(|frame| capture.frames - frame > 1080)
    {
        eprintln!("Screenshot readback timed out");
        exit.write(AppExit::error());
    }
}

pub fn sample(name: &str, samples: &mut Vec<f64>, start: std::time::Instant) {
    if samples.len() >= 100 {
        return;
    }
    samples.push(start.elapsed().as_secs_f64() * 1000.);
    if samples.len() == 100 {
        let mut values = samples[20..].to_vec();
        values.sort_by(f64::total_cmp);
        eprintln!(
            "{name}, 80 samples after warmup: median {:.4} ms, p95 {:.4} ms",
            values[40], values[75]
        );
    }
}
