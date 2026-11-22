use anyhow::Result;
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
};
use tm1_assets::{disc::Disc, sound::program};

#[derive(Resource)]
pub struct Sounds {
    sources: Vec<AudioSource>,
    voices: Vec<Handle<AudioSource>>,
    engine_range: f32,
    engine_loop: std::time::Duration,
}

impl Sounds {
    pub fn load(disc: &Disc) -> Result<Self> {
        let bank = disc.read("SND/UACORE.VAB")?;
        let mut sources = Vec::new();
        let mut engine_range = 0.;
        let mut engine_loop = std::time::Duration::ZERO;
        // SOUND.c program map for taxi engine, gun, impact, explosion and fire missile.
        for (id, prog) in [0, 12, 13, 29, 19].into_iter().enumerate() {
            let tones = program(&bank, prog)?;
            let mut mix = Vec::<[f32; 2]>::new();
            for t in tones {
                let note = if id == 0 {
                    engine_range = f32::from(t.high) - f32::from(t.low);
                    t.low
                } else {
                    60
                };
                let rate = 2f32.powf((f32::from(note) - f32::from(t.center)) / 12.);
                if id == 0 {
                    engine_loop = std::time::Duration::from_secs_f64(
                        t.loop_start.unwrap_or(0) as f64 / (44100. * f64::from(rate)),
                    );
                }
                let n = (t.pcm.len() as f32 / rate).ceil() as usize;
                mix.resize(mix.len().max(n), [0.; 2]);
                let pan = f32::from(t.pan) / 64.;
                let gain = f32::from(t.volume) / 127.;
                for (i, out) in mix[..n].iter_mut().enumerate() {
                    let at = (i as f32 * rate) as usize;
                    let v = t.pcm[at.min(t.pcm.len() - 1)] as f32 * gain;
                    out[0] += v * (2. - pan).min(1.);
                    out[1] += v * pan.min(1.);
                }
            }
            // Keep VAB stereo and quantize the listener-relative direction to five pan positions.
            for pan in [-1_f32, -0.5, 0., 0.5, 1.] {
                let pcm: Vec<_> = mix
                    .iter()
                    .flat_map(|[l, r]| [l * (1. - pan.max(0.)), r * (1. + pan.min(0.))])
                    .flat_map(|v| (v.clamp(-32768., 32767.) as i16).to_le_bytes())
                    .collect();
                sources.push(crate::music::stereo_wave(&pcm)?);
            }
        }
        Ok(Self {
            sources,
            voices: Vec::new(),
            engine_range,
            engine_loop,
        })
    }
}
#[derive(Component)]
pub struct Engine;
#[derive(Component)]
pub struct OneShot;
pub fn start(
    mut commands: Commands,
    mut sounds: ResMut<Sounds>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    sounds.voices = std::mem::take(&mut sounds.sources)
        .into_iter()
        .map(|s| assets.add(s))
        .collect();
    commands.spawn((
        Engine,
        AudioPlayer::new(sounds.voices[2].clone()),
        PlaybackSettings::LOOP
            .with_start_position(sounds.engine_loop)
            .with_volume(Volume::Linear(0.28)),
    ));
}
pub fn update(
    mut commands: Commands,
    sounds: Res<Sounds>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    engine: Query<&AudioSink, With<Engine>>,
    shots: Query<(Entity, Option<&AudioSink>), With<OneShot>>,
    mut restart: Local<u32>,
) {
    let reset = *restart != drive.restart_count;
    let pause = drive.paused || !window.focused;
    if let Ok(sink) = engine.single() {
        if reset {
            let _ = sink.try_seek(sounds.engine_loop);
        }
        sink.set_speed(2f32.powf(sounds.engine_range * drive.taxi.rev() as f32 / (99. * 12.)));
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
    *restart = drive.restart_count;
    for (e, sink) in &shots {
        if reset {
            commands.entity(e).despawn();
        } else if let Some(sink) = sink {
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
    let mut remaining = 22usize.saturating_sub(shots.iter().count());
    let heading = drive.taxi.heading() as f32 * std::f32::consts::TAU / 4096.;
    for &(id, point) in &drive.sounds {
        let car = drive.taxi.position();
        let offset = Vec3::from_array(std::array::from_fn(|i| (point[i] - car[i]) as f32));
        let range = offset.length();
        let pan = ((offset.x * heading.cos() - offset.y * heading.sin()) / range.max(100.))
            .clamp(-1., 1.);
        let bin = (pan * 2.).round() as i32 + 2;
        let volume = (1. - range / 6000.).clamp(0., 1.) * if id == 2 { 0.16 } else { 0.44 };
        if volume > 0. && remaining > 0 {
            remaining -= 1;
            commands.spawn((
                OneShot,
                AudioPlayer::new(sounds.voices[id * 5 + bin as usize].clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
            ));
        }
    }
}
