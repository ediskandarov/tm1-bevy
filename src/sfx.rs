use anyhow::Result;
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
};
use tm1_assets::{disc::Disc, sound::program};

#[derive(Resource)]
pub struct Sounds {
    sources: Vec<AudioSource>,
    rival_engines: Vec<(AudioSource, f32, std::time::Duration)>,
    voices: Vec<Handle<AudioSource>>,
    engine_range: f32,
    engine_loop: std::time::Duration,
}

pub struct Sample {
    mix: Vec<[f32; 2]>,
    pub range: f32,
    loop_start: std::time::Duration,
}
impl Sample {
    pub fn load(bank: &[u8], prog: usize, engine: bool) -> Result<Self> {
        let mut engine_range = 0.;
        let mut engine_loop = std::time::Duration::ZERO;
        let tones = program(bank, prog)?;
        let mut mix = Vec::<[f32; 2]>::new();
        for t in tones {
            let note = if engine {
                engine_range = f32::from(t.high) - f32::from(t.low);
                t.low
            } else {
                60
            };
            let rate = 2f32.powf((f32::from(note) - f32::from(t.center)) / 12.);
            if engine {
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
        Ok(Self {
            mix,
            range: engine_range,
            loop_start: engine_loop,
        })
    }
    pub fn wave(&self, pan: f32) -> Result<AudioSource> {
        let pcm: Vec<_> = self
            .mix
            .iter()
            .flat_map(|[l, r]| [l * (1. - pan.max(0.)), r * (1. + pan.min(0.))])
            .flat_map(|v| (v.clamp(-32768., 32767.) as i16).to_le_bytes())
            .collect();
        crate::music::stereo_wave(&pcm)
    }
}

pub(crate) fn engine_speed(range: f32, rev: i32) -> f32 {
    let pitch_steps = (range as i32 * 128 * rev.clamp(0, 99)) / 99;
    2f32.powf(pitch_steps as f32 / (128. * 12.))
}

impl Sounds {
    pub fn ending_engine(&self, gain: f32) -> (AudioPlayer, PlaybackSettings) {
        (
            AudioPlayer::new(self.voices[2].clone()),
            PlaybackSettings::LOOP
                .with_speed(engine_speed(self.engine_range, 30))
                .with_volume(Volume::Linear(0.28 * gain)),
        )
    }

    pub fn signature(&self) -> AudioPlayer {
        AudioPlayer::new(self.voices[5 * 5 + 2].clone())
    }

    pub fn lost_game_signature(&self) -> AudioPlayer {
        let program = self.voices.len() / 5 - 3;
        AudioPlayer::new(self.voices[program * 5 + 2].clone())
    }

    pub fn load(
        disc: &Disc,
        vehicle: tm1_core::drive::Vehicle,
        opponents: &[tm1_core::drive::Vehicle],
    ) -> Result<Self> {
        let bank = disc.read("SND/UACORE.VAB")?;
        let mut sources = Vec::new();
        let mut engine_range = 0.;
        let mut engine_loop = std::time::Duration::ZERO;
        use tm1_core::drive::Vehicle::SweetTooth;
        let (engine, special) = vehicle.sound_programs();
        for (id, prog) in [engine, 12, 13, 29, 19, special, 21, 16, 15, 18, 17, 21]
            .into_iter()
            .chain(opponents.iter().map(|&car| car.sound_programs().1))
            .chain([52, 53, 27, 38, 36])
            .chain(tm1_core::drive::ContactVoice::PROGRAMS)
            .chain([28, SweetTooth.sound_programs().1, 57, 58])
            .enumerate()
        {
            let sample = Sample::load(&bank, prog, id == 0)?;
            if id == 0 {
                engine_range = sample.range;
                engine_loop = sample.loop_start;
            }
            // Keep VAB stereo and quantize the listener-relative direction to five pan positions.
            for pan in [-1_f32, -0.5, 0., 0.5, 1.] {
                sources.push(sample.wave(pan)?);
            }
        }
        let mut rival_engines = Vec::with_capacity(opponents.len());
        for car in opponents {
            let sample = Sample::load(&bank, car.sound_programs().0, true)?;
            rival_engines.push((sample.wave(0.)?, sample.range, sample.loop_start));
        }
        Ok(Self {
            sources,
            rival_engines,
            voices: Vec::new(),
            engine_range,
            engine_loop,
        })
    }
}
#[derive(Component)]
pub struct Engine;
#[derive(Component)]
pub struct RivalEngine {
    slot: usize,
    source: Handle<AudioSource>,
    range: f32,
    loop_start: std::time::Duration,
}
#[derive(Component)]
pub struct OneShot(Option<crate::driving::ExclusiveVoice>);
pub fn start(
    mut commands: Commands,
    mut sounds: ResMut<Sounds>,
    drive: Res<crate::driving::Drive>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    sounds.voices = std::mem::take(&mut sounds.sources)
        .into_iter()
        .map(|s| assets.add(s))
        .collect();
    commands.spawn((crate::campaign::StageEntity, SpatialListener::new(1.0)));
    for (slot, (source, range, loop_start)) in sounds.rival_engines.drain(..).enumerate() {
        commands.spawn((
            crate::campaign::StageEntity,
            Transform::default(),
            RivalEngine {
                slot,
                source: assets.add(source),
                range,
                loop_start,
            },
        ));
    }
    commands.spawn((
        crate::campaign::StageEntity,
        Engine,
        AudioPlayer::new(sounds.voices[2].clone()),
        PlaybackSettings::LOOP
            .with_start_position(sounds.engine_loop)
            .with_volume(Volume::Linear(0.28 * drive.settings.effects_gain())),
    ));
}
pub fn update(
    mut commands: Commands,
    sounds: Res<Sounds>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    mut engine: Query<&mut AudioSink, With<Engine>>,
    mut shots: Query<(Entity, Option<&mut AudioSink>, &OneShot), Without<Engine>>,
    mut rivals: Query<(
        Entity,
        &RivalEngine,
        &mut Transform,
        Has<AudioPlayer>,
        Option<&mut SpatialAudioSink>,
    )>,
    mut restart: Local<u32>,
) {
    let reset = *restart != drive.restart_count;
    let pause = drive.lives == 0 || drive.pause.is_some() || !window.focused;
    let gain = drive.settings.effects_gain();
    if let Ok(mut sink) = engine.single_mut() {
        sink.set_volume(Volume::Linear(0.28 * gain));
        if reset {
            let _ = sink.try_seek(sounds.engine_loop);
        }
        sink.set_speed(engine_speed(sounds.engine_range, drive.player.rev()));
        if pause {
            sink.pause();
        } else {
            sink.play();
        }
    }
    let heading = drive.player.heading() as f32 * std::f32::consts::TAU / 4096.;
    let listener = drive.player.position();
    let right = Vec3::new(heading.cos(), -heading.sin(), 0.);
    let mix_position = |point: [i32; 3]| {
        let offset = Vec3::from_array(std::array::from_fn(|i| (point[i] - listener[i]) as f32));
        let distance = offset.length();
        let pan = (offset.dot(right) / distance.max(100.)).clamp(-1., 1.);
        (pan, (1. - distance / 6000.).clamp(0., 1.))
    };
    let occupied = shots.iter().count()
        + rivals
            .iter()
            .filter(|(_, _, _, playing, _)| *playing)
            .count();
    let mut free_engine_slots = 22usize.saturating_sub(occupied);
    let mut active_engines = 0;
    for (entity, engine, mut transform, playing, sink) in &mut rivals {
        let audible = drive.opponents.get(engine.slot).and_then(|ai| {
            let rev = ai.engine_rev()?;
            let (pan, attenuation) = mix_position(ai.position());
            (attenuation > 0.).then_some((rev, 0.28 * gain * attenuation, pan))
        });
        let Some((rev, volume, pan)) = audible else {
            if playing {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer, SpatialAudioSink)>();
            }
            continue;
        };
        if !playing {
            if free_engine_slots == 0 {
                continue;
            }
            free_engine_slots -= 1;
        }
        active_engines += 1;
        transform.translation = Vec3::X * pan;
        let speed = engine_speed(engine.range, rev);
        if let Some(mut sink) = sink {
            sink.set_speed(speed);
            sink.set_volume(Volume::Linear(volume));
            if reset {
                let _ = sink.try_seek(std::time::Duration::ZERO);
            }
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        } else if !playing {
            let mut settings = PlaybackSettings::LOOP
                .with_spatial(true)
                .with_start_position(engine.loop_start)
                .with_speed(speed)
                .with_volume(Volume::Linear(volume));
            settings.paused = pause;
            commands
                .entity(entity)
                .insert((AudioPlayer::new(engine.source.clone()), settings));
        }
    }
    *restart = drive.restart_count;
    for (e, sink, _) in &mut shots {
        if reset || drive.pause.is_some() {
            commands.entity(e).despawn();
        } else if let Some(sink) = sink {
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
    if let Some(select) = drive.pause_sound {
        let program = sounds.voices.len() / 5 - 2 + usize::from(select);
        commands.spawn((
            crate::campaign::StageEntity,
            AudioPlayer::new(sounds.voices[program * 5 + 2].clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.44 * gain)),
        ));
    }
    if drive.sounds.is_empty() {
        return;
    }
    let mut remaining = 22usize.saturating_sub(shots.iter().count() + active_engines);
    let mut active: Vec<_> = shots
        .iter()
        .filter_map(|(entity, _, shot)| shot.0.filter(|_| !reset).map(|key| (key, entity)))
        .collect();
    for sound in &drive.sounds {
        let id = sound.id;
        let (pan, attenuation) = mix_position(sound.position);
        let bin = (pan * 2.).round() as i32 + 2;
        let attenuation = if sound.full_range { 1. } else { attenuation };
        let volume = attenuation * 0.44 * sound.volume_percent.max(0) as f32 / 99.;
        let level = Volume::Linear(volume * gain);
        let playback = || {
            (
                AudioPlayer::new(sounds.voices[id * 5 + bin as usize].clone()),
                PlaybackSettings::DESPAWN.with_volume(level),
            )
        };
        if let Some(key) = sound.exclusive
            && let Some(&(_, entity)) = active.iter().find(|(active_key, _)| *active_key == key)
        {
            if matches!(
                key,
                crate::driving::ExclusiveVoice::Contact(_)
                    | crate::driving::ExclusiveVoice::ScaledEffect(_)
            ) {
                if let Ok((_, Some(mut sink), _)) = shots.get_mut(entity) {
                    sink.set_volume(level);
                } else {
                    commands.entity(entity).insert(playback());
                }
            }
            continue;
        }
        if volume > 0. && remaining > 0 {
            remaining -= 1;
            let entity = commands
                .spawn((
                    crate::campaign::StageEntity,
                    OneShot(sound.exclusive),
                    playback(),
                ))
                .id();
            if let Some(key) = sound.exclusive {
                active.push((key, entity));
            }
        }
    }
}
