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
    attempts: u8,
    pub demo: bool,
    pub special_demo: bool,
    pub demo_frame: u32,
    pub realtime: bool,
    pub elapsed: std::time::Duration,
    pub steps: u64,
}
impl Capture {
    pub fn holding_battle_frame(&self) -> bool {
        self.demo && self.attempts > 0
    }
    pub fn new(path: Option<PathBuf>, demo: bool, realtime: bool) -> Self {
        Self {
            path,
            demo,
            special_demo: false,
            demo_frame: 120,
            realtime,
            elapsed: Default::default(),
            steps: 0,
            frames: 0,
            requested: None,
            attempts: 0,
        }
    }
}

pub fn capture(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    window: Single<&Window>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<&mut bevy::camera::RenderTarget, With<Camera>>,
    mut offscreen: Local<Option<bevy::camera::RenderTarget>>,
    sinks: Query<&AudioSink>,
    rival_engines: Query<(&AudioSink, &crate::sfx::EngineTone), With<crate::sfx::RivalEngine>>,
    pads: Query<&Gamepad>,
    engine: Query<(&AudioSink, &crate::sfx::EngineTone), With<crate::sfx::Engine>>,
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
        capture.frames >= capture.demo_frame
    };
    if ready && capture.requested.is_none() {
        capture.requested = Some(capture.frames);
        capture.attempts += 1;
        if capture.demo && capture.attempts == 1 {
            eprintln!(
                "Active audio sinks: {}, connected gamepads: {}",
                sinks.iter().count(),
                pads.iter().count()
            );
            for (sink, tone) in &rival_engines {
                eprintln!(
                    "Rival engine playback {:?}, SPU pitch {:#06x}",
                    sink.position(),
                    tone.pitch_register()
                );
            }
            for (engine, tone) in &engine {
                eprintln!(
                    "Player engine playback {:?}, SPU pitch {:#06x}",
                    engine.position(),
                    tone.pitch_register()
                );
            }
            if let Ok(music) = music.single() {
                eprintln!("Music playback {:?}", music.position());
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
        // A window drawable can be unavailable on macOS. Replay the same
        // camera stack into a texture at the window's resolution and scale.
        if capture.attempts == 2 {
            let image = Image::new_target_texture(
                window.physical_width(),
                window.physical_height(),
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            );
            let target = bevy::camera::RenderTarget::Image(bevy::camera::ImageRenderTarget {
                handle: images.add(image),
                scale_factor: window.scale_factor(),
            });
            for mut camera in &mut cameras {
                if matches!(*camera, bevy::camera::RenderTarget::Window(_)) {
                    *camera = target.clone();
                }
            }
            *offscreen = Some(target);
        }
        let shot = offscreen
            .as_ref()
            .map_or_else(Screenshot::primary_window, |target| {
                Screenshot(target.clone())
            });
        commands.spawn(shot).observe(
            move |event: On<ScreenshotCaptured>,
                  mut capture: ResMut<Capture>,
                  mut exit: MessageWriter<AppExit>| {
                if event
                    .image
                    .data
                    .as_ref()
                    .is_some_and(|bytes| bytes.iter().all(|&b| b == 0))
                    && capture.attempts < 3
                {
                    warn!(
                        "Empty window readback on attempt {}, requesting another frame",
                        capture.attempts
                    );
                    capture.requested = None;
                    return;
                }
                let result = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|error| format!("{error:?}"))
                    .and_then(|image| {
                        if image.as_bytes().iter().all(|&b| b == 0) {
                            return Err("Metal returned an empty frame".into());
                        }
                        image.save(&path).map_err(|error| error.to_string())
                    });
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
