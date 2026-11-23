pub mod duel;
mod envelope;
mod interpolation;
pub mod pcm;
use anyhow::Result;
use bevy::{audio::AudioSinkPlayback, prelude::*};
use tm1_assets::{disc::Disc, sound::program};

#[derive(Component)]
pub(crate) struct MenuButtonVoice(pub bool);

pub(crate) fn play_menu_button(world: &mut World, select: bool, player: AudioPlayer) -> Entity {
    // UA_SOUND requests cycle and select with mode 5: each restarts its own ID.
    let previous = world
        .query::<(Entity, &MenuButtonVoice)>()
        .iter(world)
        .find_map(|(entity, voice)| (voice.0 == select).then_some(entity));
    if let Some(entity) = previous {
        world.despawn(entity);
    }
    world
        .spawn((MenuButtonVoice(select), player, PlaybackSettings::DESPAWN))
        .id()
}

#[derive(Resource)]
pub struct Sounds {
    menu: Vec<MenuSound>,
    turbo: Vec<EffectTone>,
    effects: Vec<Vec<EffectTone>>,
    rival_engines: Vec<Vec<EngineSample>>,
    player_engine: Vec<EngineSample>,
    screen_engine: Vec<Handle<pcm::StereoPcm>>,
    screen_levels: pcm::Levels,
    player_signature: Vec<Handle<pcm::StereoPcm>>,
    defeat_signature: Vec<Handle<pcm::StereoPcm>>,
}

struct EffectTone {
    clip: pcm::StereoPcm,
    priority: u8,
}

impl EffectTone {
    fn load(bank: &[u8], prog: usize) -> Result<Vec<Self>> {
        program(bank, prog)?
            .into_iter()
            .map(|tone| {
                let priority = tone.priority;
                let clip = if tone.loop_start.is_some() {
                    let pitch = sample_pitch(&tone, false) as u16;
                    pcm::LoopingTone::new(tone)?.voice(pitch).0
                } else {
                    pcm::StereoPcm::one_shot(&tone)
                };
                Ok(Self { clip, priority })
            })
            .collect()
    }
}

pub struct EngineSample {
    priority: u8,
    motor: pcm::LoopingTone,
    pitch: EnginePitch,
}
impl EngineSample {
    pub fn voice(&self, rev: i32) -> pcm::StereoPcm {
        self.motor.voice(self.pitch.register(rev)).0
    }
    pub fn load(bank: &[u8], car: tm1_core::drive::Vehicle) -> Result<Vec<Self>> {
        program(bank, car.sound_programs().0)?
            .into_iter()
            .map(|tone| {
                let priority = tone.priority;
                let pitch = EnginePitch::from_tone(&tone);
                Ok(Self {
                    priority,
                    pitch,
                    motor: pcm::LoopingTone::new(tone)?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
struct EnginePitch {
    base: i32,
    span: i32,
}
impl EnginePitch {
    fn from_tone(tone: &tm1_assets::sound::Tone) -> Self {
        Self {
            base: 60 + i32::from(tone.low) - i32::from(tone.center),
            span: i32::from(tone.high) - i32::from(tone.low),
        }
    }
    fn register(self, rev: i32) -> u16 {
        let steps = self.base * 128 + self.span * 128 * rev.clamp(0, 99) / 99;
        tm1_core::sound::spu_pitch((steps / 128) as u8, (steps % 128) as u8)
    }
}
fn sample_pitch(tone: &tm1_assets::sound::Tone, engine: bool) -> usize {
    usize::from(if engine {
        EnginePitch::from_tone(tone).register(0)
    } else {
        tm1_core::sound::spu_pitch(120u8.wrapping_sub(tone.center), tone.fine)
    })
    .clamp(1, usize::from(interpolation::MAX_STEP))
}
pub(crate) struct MenuSound {
    tones: Vec<tm1_assets::sound::Tone>,
    clips: std::collections::HashMap<u8, Handle<AudioSource>>,
}

impl MenuSound {
    pub fn load(bank: &[u8], program_id: usize) -> Result<Self> {
        Ok(Self {
            tones: program(bank, program_id)?,
            clips: Default::default(),
        })
    }

    pub fn player(&mut self, volume: u8, assets: &mut Assets<AudioSource>) -> AudioPlayer {
        let handle = self.clips.entry(volume).or_insert_with(|| {
            let sample = Sample::from_tones(&self.tones, volume);
            assets.add(
                sample
                    .wave()
                    .expect("decoded menu sample fits a stereo WAV"),
            )
        });
        AudioPlayer::new(handle.clone())
    }
}

pub struct Sample {
    mix: Vec<[f32; 2]>,
}
impl Sample {
    fn from_tones(tones: &[tm1_assets::sound::Tone], volume: u8) -> Self {
        let mut mix = Vec::<[f32; 2]>::new();
        for t in tones {
            let gain = pcm::VabVolume::from_tone(t).gain(volume, 99);
            Self::mix_tone(&mut mix, t, false, gain);
        }
        Self { mix }
    }
    fn mix_tone(mix: &mut Vec<[f32; 2]>, t: &tm1_assets::sound::Tone, engine: bool, gain: f32) {
        let increment = sample_pitch(t, engine);
        let start = interpolation::INITIAL_PHASE;
        let n = (t.pcm.len() * 4096)
            .saturating_sub(start)
            .div_ceil(increment);
        let end = interpolation::KEY_ON_DELAY + n;
        mix.resize(mix.len().max(end), [0.; 2]);
        let mut envelope = envelope::Envelope::new(t.adsr);
        for (i, out) in mix[interpolation::KEY_ON_DELAY..end].iter_mut().enumerate() {
            let v = envelope.apply(interpolation::interpolate(&t.pcm, start + i * increment))
                as f32
                * gain;
            out[0] += v;
            out[1] += v;
        }
    }
    pub fn wave(&self) -> Result<AudioSource> {
        let pcm: Vec<_> = self
            .mix
            .iter()
            .flatten()
            .flat_map(|v| (v.clamp(-32768., 32767.) as i16).to_le_bytes())
            .collect();
        crate::music::stereo_wave(&pcm)
    }
}

// Program 26 is the four-tone sound at ID 0x82 in the JP sound table.
const TURBO_PROGRAM: usize = 26;
// SOUND.c allocates tone records before it selects the 24 hardware voices.
const TONE_POOL_SIZE: usize = 35;

fn battle_programs(
    vehicle: tm1_core::drive::Vehicle,
    opponents: &[tm1_core::drive::Vehicle],
) -> impl Iterator<Item = usize> + '_ {
    use tm1_core::drive::Vehicle::SweetTooth;
    let (engine, special) = vehicle.sound_programs();
    [engine, 12, 13, 29, 19, special, 21, 16, 15, 18, 17, 21]
        .into_iter()
        .chain(opponents.iter().map(|&car| car.sound_programs().1))
        .chain([52, 53, 27, 38, 36])
        .chain(tm1_core::drive::ContactVoice::PROGRAMS)
        .chain([28, 54, 25, 24, 23, 22, SweetTooth.sound_programs().1])
}

impl Sounds {
    pub fn set_screen_volume(&self, volume: u8) {
        self.screen_levels.set([i32::from(volume); 2], 99);
    }
    pub fn ending_engine(&self) -> Vec<(AudioPlayer<pcm::StereoPcm>, PlaybackSettings)> {
        self.screen_engine
            .iter()
            .map(|source| (AudioPlayer(source.clone()), PlaybackSettings::ONCE))
            .collect()
    }

    pub fn signature(&self) -> &[Handle<pcm::StereoPcm>] {
        &self.player_signature
    }

    pub fn lost_game_signature(&self) -> &[Handle<pcm::StereoPcm>] {
        &self.defeat_signature
    }

    pub fn load(
        disc: &Disc,
        vehicle: tm1_core::drive::Vehicle,
        opponents: &[tm1_core::drive::Vehicle],
    ) -> Result<Self> {
        let bank = disc.read("SND/UACORE.VAB")?;
        let menu = [57, 58]
            .into_iter()
            .map(|prog| MenuSound::load(&bank, prog))
            .collect::<Result<_>>()?;
        let effects = battle_programs(vehicle, opponents)
            .map(|prog| EffectTone::load(&bank, prog))
            .collect::<Result<_>>()?;
        let player_engine = EngineSample::load(&bank, vehicle)?;
        let rival_engines = opponents
            .iter()
            .map(|&car| EngineSample::load(&bank, car))
            .collect::<Result<_>>()?;
        Ok(Self {
            menu,
            effects,
            turbo: EffectTone::load(&bank, TURBO_PROGRAM)?,
            rival_engines,
            player_engine,
            screen_engine: Vec::new(),
            screen_levels: pcm::Levels::new([33; 2], 99),
            player_signature: Vec::new(),
            defeat_signature: Vec::new(),
        })
    }
}
#[derive(Component)]
pub struct Engine;
#[derive(Component)]
pub struct ShellEngine;
#[derive(Component)]
pub struct RivalEngine {
    pub(super) slot: usize,
}
#[derive(Component)]
pub struct EngineTone {
    order: Option<u64>,
    priority: u8,
    source: Handle<pcm::StereoPcm>,
    levels: pcm::Levels,
    register: pcm::PitchRegister,
    pitch: EnginePitch,
}
impl EngineTone {
    pub(crate) fn pitch_register(&self) -> u16 {
        self.register.get()
    }
}
#[derive(Component)]
pub struct OneShot {
    range: u32,
    timed: Option<TimedVoice>,
    key: Option<crate::driving::ExclusiveVoice>,
    levels: pcm::Levels,
    order: u64,
    priority: u8,
}

// UA_SOUND requests mode 4 with a 960-update timeout for flamethrower ID 0x0f.
#[derive(Clone, Copy)]
struct TimedVoice {
    start: u32,
}
impl TimedVoice {
    fn from_request(request: &crate::driving::BattleRequest) -> Option<Self> {
        use crate::driving::{BattleRequest, BattleSound, ExclusiveVoice};
        match request {
            BattleRequest::Play(BattleSound {
                exclusive: Some(ExclusiveVoice::WeaponLaunch(7)),
                frame,
                ..
            }) => Some(Self { start: *frame }),
            _ => None,
        }
    }
    fn expired(self, frame: u32) -> bool {
        frame.wrapping_sub(self.start) >= 960
    }
}

#[derive(Resource, Default)]
pub struct VoiceSequence {
    serial: u64,
    engines_stopped: [bool; 9],
    engines_deferred: [bool; 9],
    engine_mix: [Option<crate::driving::EngineMix>; 9],
    last_frame: Option<u32>,
}
impl VoiceSequence {
    // A render update can contain a stop followed by a new engine request.
    fn update_engines(&mut self, requests: &[crate::driving::BattleRequest]) -> [bool; 9] {
        let mut restarted = [false; 9];
        for request in requests {
            use crate::driving::BattleRequest::*;
            match *request {
                EngineRev(slot, mix) => {
                    restarted[slot] |= self.engines_stopped[slot];
                    self.engines_stopped[slot] = false;
                    self.engines_deferred[slot] = false;
                    self.engine_mix[slot] = Some(mix);
                }
                EngineStop(slot) => self.engines_stopped[slot] = true,
                EngineDeferred(slot) => self.engines_deferred[slot] = true,
                _ => {}
            }
        }
        restarted
    }

    fn next(&mut self) -> u64 {
        self.serial += 1;
        self.serial
    }
}

struct Candidate {
    entity: Entity,
    order: u64,
    priority: u8,
    range: u32,
    finishing: bool,
    engine: bool,
}

pub fn start(
    mut commands: Commands,
    mut sounds: ResMut<Sounds>,
    drive: Res<crate::driving::Drive>,
    mut clips: ResMut<Assets<pcm::StereoPcm>>,
) {
    let screen_levels = sounds.screen_levels.clone();
    let mut register_signature = |tones: &[EffectTone]| {
        tones
            .iter()
            .map(|tone| clips.add(tone.clip.with_levels(screen_levels.clone())))
            .collect()
    };
    let (signature, _, _) = crate::driving::car_sample(
        tm1_core::drive::CarSound::Signature,
        drive.player.vehicle(),
        0,
        drive.recharge_sound(),
    );
    sounds.player_signature = register_signature(&sounds.effects[signature]);
    sounds.defeat_signature = register_signature(sounds.effects.last().unwrap());
    let mut sequence = VoiceSequence::default();
    let player = std::mem::take(&mut sounds.player_engine);
    let rivals = std::mem::take(&mut sounds.rival_engines);
    let engines = std::iter::once((None, player)).chain(
        rivals
            .into_iter()
            .enumerate()
            .map(|(slot, tones)| (Some(slot), tones)),
    );
    for (slot, tones) in engines {
        let levels = pcm::Levels::new(
            drive.listener.channels(
                drive.player.position(),
                drive.settings.effects_volume(),
                false,
            ),
            99,
        );
        for tone in tones {
            let pitch = tone.pitch;
            let priority = tone.priority;
            let (clip, register) = tone.motor.voice(pitch.register(drive.player.rev()));
            let source = clips.add(clip.with_levels(levels.clone()));
            let mut entity = commands.spawn((
                crate::campaign::StageEntity,
                EngineTone {
                    priority,
                    order: slot.is_none().then(|| sequence.next()),
                    source: source.clone(),
                    levels: levels.clone(),
                    register,
                    pitch,
                },
            ));
            if let Some(slot) = slot {
                entity.insert(RivalEngine { slot });
            } else {
                sounds
                    .screen_engine
                    .push(clips.add(tone.voice(30).with_levels(screen_levels.clone())));
                entity.insert((Engine, AudioPlayer(source), PlaybackSettings::ONCE));
            }
        }
    }
    commands.insert_resource(sequence);
}
pub fn update(
    mut commands: Commands,
    mut sounds: ResMut<Sounds>,
    mut assets: ResMut<Assets<AudioSource>>,
    mut clips: ResMut<Assets<pcm::StereoPcm>>,
    drive: Res<crate::driving::Drive>,
    window: Single<&Window>,
    mut shots: Query<(Entity, Option<&mut AudioSink>, &OneShot), Without<Engine>>,
    mut engines: Query<
        (
            Entity,
            &mut EngineTone,
            Option<&RivalEngine>,
            Has<AudioPlayer<pcm::StereoPcm>>,
            Option<&mut AudioSink>,
        ),
        Without<OneShot>,
    >,
    shell_engines: Query<
        (Entity, Option<&AudioSink>),
        (With<ShellEngine>, Without<EngineTone>, Without<OneShot>),
    >,
    mut restart: Local<u32>,
    mut sequence: ResMut<VoiceSequence>,
) {
    let reset = *restart != drive.audio_restart_count;
    let shell_rev = drive.lost_life_elapsed().is_some() || drive.showing_boss_intro();
    let stop_battle = drive.lives == 0 || shell_rev || drive.pause.is_some();
    let pause = !window.focused;
    if shell_rev && shell_engines.is_empty() {
        sounds.set_screen_volume(drive.settings.effects_volume());
        for (player, mut settings) in sounds.ending_engine() {
            settings.paused = pause;
            commands.spawn((crate::campaign::StageEntity, ShellEngine, player, settings));
        }
    }
    for (entity, sink) in &shell_engines {
        if !shell_rev {
            commands.entity(entity).despawn();
        } else if let Some(sink) = sink {
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
    let volume = drive.settings.effects_volume();
    let mut ranked = Vec::new();
    let frame = drive.player.frame();
    let new_frame = sequence.last_frame.replace(frame) != Some(frame);
    if reset {
        sequence.engines_stopped = [false; 9];
        sequence.engines_deferred = [false; 9];
        sequence.engine_mix = [None; 9];
    }
    let restarted = sequence.update_engines(&drive.sounds);
    for (entity, mut engine, rival, playing, sink) in &mut engines {
        let slot = rival.map_or(0, |r| r.slot + 1);
        let restart_engine = reset || restarted[slot];
        if restarted[slot] {
            engine.order = None;
        }
        let audible = if let Some(rival) = rival {
            drive.opponents.get(rival.slot).and_then(|ai| {
                ai.engine_rev()
                    .map(|rev| (rev, drive.listener.channels(ai.position(), volume, false)))
            })
        } else {
            Some((
                drive.player.rev(),
                drive
                    .listener
                    .channels(drive.player.position(), volume, false),
            ))
        };
        let Some((rev, channels)) =
            audible.filter(|_| !stop_battle && !sequence.engines_stopped[slot])
        else {
            engine.order = None;
            if playing {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer<pcm::StereoPcm>, AudioSink)>();
            }
            continue;
        };
        let deferred = sequence.engines_deferred[slot];
        if !playing && (deferred || (!new_frame && !reset)) {
            continue;
        }
        let position = rival
            .map(|r| drive.opponents[r.slot].position())
            .unwrap_or_else(|| drive.player.position());
        let mix = sequence.engine_mix[slot];
        ranked.push(Candidate {
            entity,
            order: *engine.order.get_or_insert_with(|| sequence.next()),
            priority: engine.priority,
            range: mix.map_or_else(|| drive.listener.sound_range(position), |m| m.range),
            finishing: deferred,
            engine: true,
        });
        if !deferred || mix.is_some() {
            engine.levels.set(mix.map_or(channels, |m| m.channels), 99);
            engine
                .register
                .set(engine.pitch.register(mix.map_or(rev, |m| m.rev)));
        }
        if restart_engine && playing {
            commands.entity(entity).remove::<AudioSink>();
        }
        if let Some(sink) = sink.filter(|_| !restart_engine) {
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        } else if !playing || restart_engine {
            let mut settings = PlaybackSettings::ONCE;
            settings.paused = pause;
            commands
                .entity(entity)
                .insert((AudioPlayer(engine.source.clone()), settings));
        }
    }
    *restart = drive.audio_restart_count;
    for (e, sink, shot) in &mut shots {
        if reset || stop_battle || shot.timed.is_some_and(|t| t.expired(frame)) {
            commands.entity(e).despawn();
            continue;
        }
        ranked.push(Candidate {
            entity: e,
            order: shot.order,
            priority: shot.priority,
            range: shot.range,
            finishing: shot.timed.is_none(),
            engine: false,
        });
        if let Some(sink) = sink {
            if pause {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
    if let Some(select) = drive.pause_sound {
        let player = sounds.menu[usize::from(select)].player(volume, &mut assets);
        commands.queue(move |world: &mut World| {
            let entity = play_menu_button(world, select, player);
            world
                .entity_mut(entity)
                .insert(crate::campaign::StageEntity);
        });
    }

    if stop_battle {
        return;
    }
    let mut active: Vec<_> = shots
        .iter()
        .filter_map(|(entity, _, shot)| {
            shot.key
                .filter(|_| !reset && !shot.timed.is_some_and(|t| t.expired(frame)))
                .map(|key| (key, entity, shot.levels.clone()))
        })
        .collect();
    use crate::driving::{BattleRequest, ExclusiveVoice};
    for request in &drive.sounds {
        let (tones, key, channels, percent, range) = match request {
            BattleRequest::EngineRev(..)
            | BattleRequest::EngineStop(_)
            | BattleRequest::EngineDeferred(_) => continue,
            BattleRequest::Play(sound) => (
                &sounds.effects[sound.id],
                sound.exclusive,
                sound.channels,
                sound.volume_percent,
                sound.range,
            ),
            BattleRequest::Turbo => (
                &sounds.turbo,
                Some(ExclusiveVoice::Turbo),
                [i32::from(volume); 2],
                99,
                0,
            ),
            BattleRequest::Stop(key) => {
                active.retain(|(group, entity, _)| {
                    if group == key {
                        commands.entity(*entity).despawn();
                        ranked.retain(|v| v.entity != *entity);
                    }
                    group != key
                });
                continue;
            }
        };
        if let Some(key) = key
            && let Some((_, _, control)) = active.iter().find(|(group, _, _)| *group == key)
        {
            refresh_requested(&mut ranked, &active, key, range);
            control.set(channels, percent);
            continue;
        }
        let control = pcm::Levels::new(channels, percent);
        for tone in tones
            .iter()
            .take(TONE_POOL_SIZE.saturating_sub(ranked.len()))
        {
            let clip = tone.clip.with_levels(control.clone());
            let order = sequence.next();
            let entity = commands
                .spawn((
                    crate::campaign::StageEntity,
                    OneShot {
                        timed: TimedVoice::from_request(request),
                        range,
                        key,
                        levels: control.clone(),
                        order,
                        priority: tone.priority,
                    },
                    AudioPlayer(clips.add(clip)),
                    PlaybackSettings::DESPAWN,
                ))
                .id();
            ranked.push(Candidate {
                entity,
                order,
                priority: tone.priority,
                range,
                finishing: false,
                engine: false,
            });
            if let Some(key) = key {
                active.push((key, entity, control.clone()));
            }
        }
    }
    if ranked.len() > 24 {
        rank_voices(&mut ranked);
        for voice in &ranked[24..] {
            if voice.engine {
                commands
                    .entity(voice.entity)
                    .remove::<(AudioPlayer<pcm::StereoPcm>, AudioSink)>();
            } else {
                commands.entity(voice.entity).despawn();
            }
        }
    }
}

fn rank_voices(ranked: &mut [Candidate]) {
    // Requested tones compete first. Finishing tones fill any unused voices in list order.
    ranked.sort_unstable_by_key(|v| (v.finishing, v.order));
    let requested = ranked.partition_point(|v| !v.finishing);
    tm1_core::sound::sort_requested(&mut ranked[..requested], |v| v.range, |v| v.priority);
}

fn refresh_requested(
    ranked: &mut [Candidate],
    active: &[(crate::driving::ExclusiveVoice, Entity, pcm::Levels)],
    key: crate::driving::ExclusiveVoice,
    range: u32,
) {
    for (_, entity, _) in active.iter().filter(|(group, _, _)| *group == key) {
        if let Some(voice) = ranked.iter_mut().find(|v| v.entity == *entity) {
            voice.finishing = false;
            voice.range = range;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{audio::Volume, ecs::system::RunSystemOnce};
    use tm1_core::drive::{Difficulty, Level, Vehicle};

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn menu_pcm_uses_the_selected_integer_voice_level() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let mut assets = Assets::default();
        // Each row gives the JP seven-bit tone level at channel volumes 0, 10, 33 and 57.
        for (id, levels) in [
            (57, [0, 10, 33, 57]),
            (58, [0, 8, 26, 46]),
            (36, [0, 3, 9, 16]),
            (41, [0, 7, 25, 43]),
        ] {
            let mut sound = MenuSound::load(&bank, id).unwrap();
            for (volume, level) in [0, 10, 33, 57].into_iter().zip(levels) {
                let handle = sound.player(volume, &mut assets).0;
                let mut expected = Vec::new();
                for tone in &sound.tones {
                    Sample::mix_tone(&mut expected, tone, false, level as f32 * 129. / 16384.);
                }
                let bytes = &assets.get(&handle).unwrap().bytes[44..];
                assert_eq!(bytes.len(), expected.len() * 4);
                for (frame, samples) in bytes.chunks_exact(4).zip(expected) {
                    assert_eq!(
                        frame,
                        samples
                            .map(|s| (s.clamp(-32768., 32767.) as i16).to_le_bytes())
                            .as_flattened()
                    );
                }
                assert_eq!(sound.player(volume, &mut assets).0, handle);
            }
        }
    }

    #[test]
    fn engine_stop_survives_render_updates_until_the_next_rev_request() {
        use crate::driving::BattleRequest::{EngineRev, EngineStop};
        let mut sequence = VoiceSequence::default();
        let rev = || EngineRev(1, Default::default());
        let stop = || EngineStop(1);
        sequence.update_engines(&[rev(), stop()]);
        sequence.update_engines(&[]);
        assert!(sequence.engines_stopped[1]);
        assert!(sequence.update_engines(&[rev()])[1]);
        assert!(!sequence.engines_stopped[1]);
        assert!(sequence.update_engines(&[stop(), rev(), rev()])[1]);
        assert!(!sequence.engines_stopped[1]);
        assert!(!sequence.update_engines(&[rev()])[1]);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn saturated_queue_holds_engine_pitch_and_releases_its_voice_under_pressure() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleRequest, BattleSound, Drive, EngineMix};
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let rev = EngineMix {
            rev: 80,
            channels: [15, 28],
            range: 120,
        };
        // Two simulation steps arrive together. Only the first rev reached the sound queue.
        world.resource_mut::<Drive>().sounds = vec![
            BattleRequest::EngineRev(0, rev),
            BattleRequest::EngineDeferred(0),
        ];
        world.run_system_once(update).unwrap();
        let mut motors = world
            .query_filtered::<&EngineTone, (With<Engine>, With<AudioPlayer<pcm::StereoPcm>>)>();
        assert_eq!(motors.iter(&world).count(), 2);
        for engine in motors.iter(&world) {
            assert_eq!(engine.pitch_register(), engine.pitch.register(80));
        }
        let mut requests: Vec<_> = (0..10)
            .map(|_| {
                BattleRequest::Play(BattleSound {
                    frame: 0,
                    id: 4,
                    exclusive: None,
                    volume_percent: 99,
                    range: 0,
                    channels: [33; 2],
                })
            })
            .collect();
        requests.push(BattleRequest::EngineDeferred(0));
        world.resource_mut::<Drive>().sounds = requests;
        world.run_system_once(update).unwrap();
        assert_eq!(motors.iter(&world).count(), 0);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn effect_loops_preserve_fractional_pitch_across_repeats() {
        use bevy::audio::Decodable;
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        // Both tones have extreme VAB pan settings. The game's final voice
        // volume write centers both the baked clip and the live looping voice.
        let hover = program(&bank, 38).unwrap();
        assert_eq!(
            hover.iter().map(|tone| tone.pan).collect::<Vec<_>>(),
            [0, 127]
        );
        for tone in hover {
            let sample = Sample::from_tones(&[tone], 33);
            assert!(sample.mix.iter().any(|pair| pair[0] != 0.));
            assert!(sample.mix.iter().all(|pair| pair[0] == pair[1]));
        }
        for prog in [12, TURBO_PROGRAM, 38] {
            let clips = EffectTone::load(&bank, prog).unwrap();
            for (mut tone, effect) in program(&bank, prog).unwrap().into_iter().zip(clips) {
                tone.prepare_loop();
                let Some(marker) = tone.loop_start else {
                    continue;
                };
                let pitch = sample_pitch(&tone, false);
                let end = tone.pcm.len();
                let frames = ((end + 3 * (end - marker)) * 4096).div_ceil(pitch);
                let mut voice = effect.clip.decoder();
                for frame in 0..frames {
                    let expected =
                        reference_loop_frame(&tone, frame.saturating_sub(4) * pitch, frame);
                    for channel in
                        expected.map(|v| v * pcm::VabVolume::from_tone(&tone).gain(33, 99))
                    {
                        assert_eq!(voice.next(), Some(channel), "program {prog}, frame {frame}");
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn effect_fine_tune_reaches_the_pcm_and_sample_length() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let tone = program(&bank, 44).unwrap().remove(0);
        assert_eq!((tone.center, tone.fine), (69, 18));
        // SOUND.c sets note 51, fine 18 on this voice (pitch register 2453).
        let increment = 2453;
        let frames = 4 + ((tone.pcm.len() - 3) * 4096).div_ceil(increment);
        assert_eq!(tone.program_volume, 75);
        let gain = f32::from(25 * u16::from(tone.volume) / 99) * (129. / 16384.);
        // Independent decoder and four-tap interpolation at pitch register 2453.
        let samples = [
            (0, 0),
            (62, 0),
            (111, 2),
            (127, -82),
            (128, -97),
            (129, -107),
            (137, -83),
            (255, -2),
            (400, 127),
            (511, 24),
            (666, -7),
            (777, 105),
            (1000, 235),
            (1023, -255),
        ];
        for (frame, value) in samples {
            assert_eq!(
                interpolation::interpolate(&tone.pcm, frame * increment),
                value
            );
        }
        let mut mix = Vec::new();
        Sample::mix_tone(&mut mix, &tone, false, gain);
        assert_eq!(mix.len(), frames);
        assert_eq!(mix[..4], [[0.; 2]; 4]);
        for (i, channels) in mix[4..].iter().enumerate() {
            let volume = match i {
                0 => 0,
                1 => 14336,
                2 => 28672,
                3 => 32256,
                4..20 => 32767,
                _ => 32759,
            };
            let v = interpolation::interpolate(&tone.pcm, 3 * 4096 + i * increment);
            assert_eq!(*channels, [((v * volume) >> 15) as f32 * gain; 2]);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn actor_signatures_and_wrecks_select_the_disc_programs() {
        use tm1_core::drive::CarSound;
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let table = 2048 + (0x8016fefc - base) as usize;
        let program_for = |id: usize| exe[table + 6 * id + 2] as usize;
        let bank = battle_programs(Vehicle::YellowJacket, &[]).collect::<Vec<_>>();
        for (weapon, id) in [0x15, 0x1b, 0x24, 0x1e, 0x21, 0x18].into_iter().enumerate() {
            assert_eq!(
                bank[crate::driving::missile_sample(weapon as u8)],
                program_for(id)
            );
        }
        // IDs follow the original car sound order, including Minion's separate signature.
        let cars = [
            Vehicle::SweetTooth,
            Vehicle::YellowJacket,
            Vehicle::Darkside,
            Vehicle::Hammerhead,
            Vehicle::Outlaw,
            Vehicle::CrimsonFury,
            Vehicle::Warthog,
            Vehicle::MrGrimm,
            Vehicle::PitViper,
            Vehicle::Thumper,
            Vehicle::Spectre,
            Vehicle::RoadKill,
            Vehicle::Minion,
        ];
        for (i, car) in cars.into_iter().enumerate() {
            for (slot, pair) in [
                (0, [car, Vehicle::YellowJacket]),
                (1, [Vehicle::YellowJacket, car]),
            ] {
                let bank = battle_programs(pair[0], &pair[1..]).collect::<Vec<_>>();
                let launch_ids = [
                    0x2a, 0x3c, 0x42, 0x36, 0x3f, 0x39, 0x33, 0x48, 0x27, 0x45, 0x30, 0x2d, 0x4b,
                ];
                assert_eq!(
                    bank[if slot == 0 { 5 } else { 12 }],
                    program_for(launch_ids[i])
                );
                assert_eq!(bank[6], program_for(0x0f));
                for (sound, id) in [(CarSound::Signature, 0x74 + i), (CarSound::Explosion, 0x73)] {
                    let (index, _, level) = crate::driving::car_sample(sound, car, slot, 13);
                    assert_eq!(bank[index], program_for(id), "{car:?} {sound:?}");
                    assert_eq!(level, 99);
                }
            }
        }
    }

    fn load_retail_program(
        cpu: &mut crate::reference_cpu::Cpu,
        disc: &Disc,
        id: u32,
    ) -> (usize, Vec<tm1_assets::sound::Tone>) {
        let at = (0x16fefc + 6 * id + 2) as usize;
        let program_id = usize::from(u16::from_le_bytes(cpu.ram[at..at + 2].try_into().unwrap()));
        let tones = program(&disc.read("SND/UACORE.VAB").unwrap(), program_id).unwrap();
        let single = 0x801963a8 + 28 * id;
        cpu.put(single, &id.to_le_bytes());
        cpu.put(single + 4, &[tones.len() as u8, tones[0].program_volume]);
        cpu.put(single + 20, &1u32.to_le_bytes());
        (program_id, tones)
    }

    fn retail_sound_cpu(disc: &Disc, id: u32) -> (crate::reference_cpu::Cpu, usize, usize) {
        let exe = disc.read("SIPS_600.07").unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        let (program_id, tones) = load_retail_program(&mut cpu, disc, id);
        cpu.put(0x8016fecc + 24, &(3357u32 * 3357).to_le_bytes());
        // Supply the disc tone attributes and stand in for the SDK audio device.
        for (index, tone) in tones.iter().enumerate() {
            cpu.put(
                0x801972dc + 0x84 * index as u32 + 6,
                &[
                    tone.priority,
                    0,
                    tone.volume,
                    tone.pan,
                    tone.center,
                    tone.fine,
                ],
            );
        }
        for endpoint in [0x801671c8, 0x80166fac, 0x8016efe4, 0x8016c494, 0x8016f674] {
            cpu.put(endpoint, &[8, 0, 224, 3, 0x21, 0x10, 0, 0]); // return zero
        }
        cpu.put(0x8016bbc8, &[8, 0, 224, 3, 0x21, 0x10, 0x80, 0]); // return chosen voice
        (cpu, program_id, tones.len())
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn catapult_launch_loops_all_seven_disc_tones_until_weapon_stop() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleRequest, BattleSound, Drive, ExclusiveVoice};
        use bevy::audio::Decodable;

        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let (mut cpu, prog, layers) = retail_sound_cpu(disc, 0x12);
        assert_eq!((prog, layers), (25, 7));
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let id = world.resource::<Drive>().recharge_sound() + 11;
        let key = ExclusiveVoice::WeaponLaunch(6);
        cpu.call(0x80107c6c, &[6, 0, 0]);
        let mut started = Vec::new();
        cpu.call_observed(0x80105ebc, &[], 0x80000080, |cpu| {
            if cpu.pc() == 0x8016bbc8 {
                assert_eq!(cpu.r[6] as usize, prog);
                started.push(cpu.r[4]);
            }
        });
        assert_eq!(started.len(), layers);
        world.resource_mut::<Drive>().sounds = vec![BattleRequest::Play(BattleSound {
            frame: 0,
            id,
            exclusive: Some(key),
            volume_percent: 99,
            range: 0,
            channels: [33; 2],
        })];
        world.run_system_once(update).unwrap();
        let mut voices: Vec<_> = world
            .query::<(&OneShot, &AudioPlayer<pcm::StereoPcm>)>()
            .iter(&world)
            .filter(|(voice, _)| voice.key == Some(key))
            .map(|(voice, player)| (voice.order, player.0.clone()))
            .collect();
        voices.sort_by_key(|(order, _)| *order);
        assert_eq!(voices.len(), layers);
        for ((_, handle), mut tone) in voices.into_iter().zip(program(&bank, prog).unwrap()) {
            let pitch = sample_pitch(&tone, false);
            tone.prepare_loop();
            let end = tone.pcm.len() * 4096;
            let start = tone.loop_start.unwrap() * 4096;
            let gain = pcm::VabVolume::from_tone(&tone).gain(33, 99);
            assert_eq!(tone.adsr, [0x80ff, 0x4e00]);
            let wrap = (end - interpolation::INITIAL_PHASE).div_ceil(pitch);
            let mut source = world
                .resource::<Assets<pcm::StereoPcm>>()
                .get(&handle)
                .unwrap()
                .decoder();
            let mut tail = source
                .by_ref()
                .skip(2 * (interpolation::KEY_ON_DELAY + wrap));
            for i in wrap..wrap + 256 {
                let phase =
                    start + (interpolation::INITIAL_PHASE + i * pitch - end) % (end - start);
                // Linear sustain subtracts eight every eight output frames after decay.
                let level = (32759 - 8 * ((i as i32 - 20) / 8)).max(0);
                let sample = (interpolation::interpolate(&tone.pcm, phase) * level >> 15) as f32
                    / 32768.
                    * gain;
                assert_eq!([tail.next(), tail.next()], [Some(sample); 2]);
            }
        }
        cpu.put(0x8016bfc0, &[8, 0, 224, 3, 0, 0, 0, 0]);
        cpu.call(0x80107d18, &[6]);
        let mut stopped = Vec::new();
        cpu.call_observed(0x80105ebc, &[], 0x80000080, |cpu| {
            if cpu.pc() == 0x8016bfc0 {
                stopped.push(cpu.r[4]);
            }
        });
        started.sort();
        stopped.sort();
        assert_eq!(stopped, started);
        world.resource_mut::<Drive>().sounds = vec![BattleRequest::Stop(key)];
        world.run_system_once(update).unwrap();
        assert!(
            !world
                .query::<&OneShot>()
                .iter(&world)
                .any(|voice| voice.key == Some(key))
        );
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn jp_stop_flush_mutes_unused_channels_only_while_tones_remain() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let mut tone_count = 0;
        for id in 0..128 {
            if bank[32 + id * 16] != 0 {
                for tone in program(&bank, id).unwrap() {
                    assert_eq!(tone.adsr[1] & 0x3f, 0, "release bits in VAB program {id}");
                    tone_count += 1;
                }
            }
        }
        assert_eq!(tone_count, 172);
        for engine_running in [false, true] {
            let (mut cpu, _, layers) = retail_sound_cpu(&disc, 0x12);
            cpu.call(0x80107c6c, &[6, 0, 0]); // Catapult launch.
            if engine_running {
                let (_, engine_tones) = load_retail_program(&mut cpu, &disc, 0x4e);
                assert_eq!(engine_tones.len(), 1);
                cpu.call(0x80106d60, &[0x4e, 0, 0, 1, 30, 0]);
            }
            cpu.call(0x80105ebc, &[]);
            let table = 0x196390;
            let occupied = cpu.ram[table..table + 24].to_vec();
            assert_eq!(
                occupied.iter().filter(|&&slot| slot != 0).count(),
                layers + usize::from(engine_running)
            );
            // The engine is still sounding at the second flush.
            cpu.put(0x8016f674, &[8, 0, 224, 3, 1, 0, 2, 36]);
            cpu.call(0x80107d18, &[6]);
            let mut muted = Vec::new();
            cpu.call_observed(0x80105ebc, &[], 0x80000080, |cpu| {
                if cpu.pc() == 0x8016c494 {
                    assert_eq!(&cpu.r[5..7], &[0, 0]);
                    muted.push(cpu.r[4] as usize);
                }
            });
            let slots = &cpu.ram[table..table + 24];
            let released_mask = occupied
                .iter()
                .zip(slots)
                .enumerate()
                .filter(|(_, (before, after))| **before != 0 && **after == 0)
                .fold(0u32, |mask, (slot, _)| mask | (1 << slot));
            assert_eq!(released_mask.count_ones() as usize, layers);
            // SsUtKeyOffV queues the bits in the SDK before the SPU transfer.
            let sdk_key_off = (cpu.word(0x801f8510) & 0xffff) | (cpu.word(0x801f8518) << 16);
            assert_eq!(sdk_key_off, released_mask);
            let expected: Vec<_> = (0..24)
                .filter(|&i| engine_running && slots[i] == 0)
                .collect();
            assert_eq!(muted, expected);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn timed_flame_voice_matches_jp_sound_flushes() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let (mut cpu, _, _) = retail_sound_cpu(&disc, 0xf);
        cpu.put(0x8016bfc0, &[8, 0, 224, 3, 0, 0, 0, 0]); // SDK key off
        cpu.call(0x80107c6c, &[7, 31, 0]);
        cpu.call(0x80105ebc, &[]);
        // Keep the hardware voice on to exercise the timer to its full duration.
        cpu.put(0x8016f674, &[8, 0, 224, 3, 1, 0, 2, 36]);
        let start = u32::MAX - 500;
        let timed = TimedVoice { start };
        for elapsed in 0..=961 {
            let tone = cpu.word(0x801963a8 + 28 * 0xf + 24);
            assert_eq!(
                timed.expired(start.wrapping_add(elapsed)),
                tone == 0,
                "flush {elapsed}"
            );
            if tone != 0 {
                assert_eq!(cpu.word(tone + 0x4c), 960 - elapsed);
                assert_eq!(cpu.word(tone + 0x74), 31);
            }
            cpu.call(0x80105ebc, &[]);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn simultaneous_pedestrian_screams_share_retail_voices() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleRequest, BattleSound, Drive, effect_sample, effect_voice};

        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let (mut cpu, program_id, count_tones) = retail_sound_cpu(disc, 0x89);
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let id = effect_sample(36, world.resource::<Drive>().recharge_sound()).unwrap();
        let mut previous = Vec::new();
        for count in [2, 3, 1] {
            let mut requests = Vec::new();
            for _ in 0..count {
                cpu.call(0x801085d0, &[]);
                requests.push(BattleRequest::Play(BattleSound {
                    frame: 0,
                    id,
                    exclusive: effect_voice(36),
                    volume_percent: 99,
                    range: 0,
                    channels: [33; 2],
                }));
            }
            let mut key_ons = 0;
            cpu.call_observed(0x80105ebc, &[], 0x80000080, |cpu| {
                if cpu.pc() == 0x8016bbc8 {
                    assert_eq!(cpu.r[6] as usize, program_id);
                    key_ons += 1;
                }
            });
            world.resource_mut::<Drive>().sounds = requests;
            world.run_system_once(update).unwrap();
            let mut voices: Vec<_> = world
                .query_filtered::<Entity, With<OneShot>>()
                .iter(&world)
                .collect();
            voices.sort();
            assert_eq!(voices.len(), count_tones);
            assert_eq!(
                voices.iter().filter(|v| !previous.contains(*v)).count(),
                key_ons
            );
            previous = voices;
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn hover_guns_and_car_guns_stop_independently_in_queue_order() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleRequest, BattleSound, Drive, ExclusiveVoice};
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Rooftops,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let car = ExclusiveVoice::MachineGuns(Vehicle::RoadKill);
        let hover = ExclusiveVoice::HoverGuns;
        let play = |key| {
            BattleRequest::Play(BattleSound {
                frame: 0,
                id: 1,
                exclusive: Some(key),
                volume_percent: 99,
                range: 0,
                channels: [33; 2],
            })
        };
        let voices = |world: &mut World, key| {
            let mut entities = world
                .query::<(Entity, &OneShot, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(world)
                .filter(|(_, v, _)| v.key == Some(key))
                .map(|(e, _, _)| e)
                .collect::<Vec<_>>();
            entities.sort();
            entities
        };
        world.resource_mut::<Drive>().sounds =
            vec![play(car), play(hover), BattleRequest::Stop(hover)];
        world.run_system_once(update).unwrap();
        let player = voices(&mut world, car);
        assert_eq!(player.len(), 2);
        assert!(voices(&mut world, hover).is_empty());
        world.resource_mut::<Drive>().sounds = vec![BattleRequest::Stop(hover), play(hover)];
        world.run_system_once(update).unwrap();
        let merc = voices(&mut world, hover);
        assert_eq!(merc.len(), 2);
        assert_eq!(voices(&mut world, car), player);
        world.resource_mut::<Drive>().sounds = vec![play(hover), BattleRequest::Stop(car)];
        world.run_system_once(update).unwrap();
        assert_eq!(voices(&mut world, hover), merc);
        assert!(voices(&mut world, car).is_empty());
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_explosions_preempt_engines_then_yield_finishing_voices() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::{BattleSound, Drive};
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.resource_mut::<Drive>().sounds = (0..10)
            .map(|i| BattleSound {
                frame: 0,
                id: 4,
                exclusive: None,
                volume_percent: 99,
                channels: [33; 2],
                range: i * 10,
            })
            .map(crate::driving::BattleRequest::Play)
            .collect();
        world.run_system_once(update).unwrap();
        let shots = |world: &mut World| {
            let mut shots: Vec<_> = world
                .query::<&OneShot>()
                .iter(world)
                .map(|v| (v.order, v.priority))
                .collect();
            shots.sort();
            shots
        };
        let first = shots(&mut world);
        assert_eq!(first.len(), 24);
        assert!(first.iter().all(|v| v.1 == 21));
        let engines = |world: &mut World| {
            world
                .query_filtered::<Entity, (With<Engine>, With<AudioPlayer<pcm::StereoPcm>>)>()
                .iter(world)
                .count()
        };
        assert_eq!(engines(&mut world), 0);
        world.resource_mut::<Drive>().sounds.clear();
        world.run_system_once(update).unwrap();
        assert_eq!(shots(&mut world), first);
        world.insert_resource(crate::smoke::Capture::new(None, true, false));
        world.run_system_once(crate::driving::advance).unwrap();
        world.resource_mut::<Drive>().sounds.clear();
        world.run_system_once(update).unwrap();
        assert_eq!(engines(&mut world), 2);
        let finishing = shots(&mut world);
        assert!(finishing.len() <= 22);
        assert_eq!(finishing, first[..finishing.len()]);
        world.resource_mut::<Drive>().pause = Some(crate::pause::Page::Choices(0));
        world.run_system_once(update).unwrap();
        assert_eq!(engines(&mut world), 0);
        assert!(shots(&mut world).is_empty());
        world.resource_mut::<Drive>().pause = None;
        world.run_system_once(update).unwrap();
        assert_eq!(engines(&mut world), 0);
        world.run_system_once(crate::driving::advance).unwrap();
        world.run_system_once(update).unwrap();
        assert_eq!(engines(&mut world), 2);
    }

    fn reference_loop_frame(
        tone: &tm1_assets::sound::Tone,
        phase: usize,
        frame: usize,
    ) -> [f32; 2] {
        if frame < 4 {
            return [0.; 2];
        }
        let phase = (3 << 12) + phase;
        let end = tone.pcm.len() * 4096;
        let period = end - tone.loop_start.unwrap() * 4096;
        let phase = if phase >= end {
            end - period + (phase - end) % period
        } else {
            phase
        };
        assert_eq!(tone.adsr, [0x80ff, 0x1fc0]);
        let level = match frame - 4 {
            0 => 0,
            1 => 14336,
            2 => 28672,
            3 => 32256,
            4..20 => 32767,
            _ => 32759,
        };
        let mono = ((interpolation::interpolate(&tone.pcm, phase) * level) >> 15) as f32;
        [mono.clamp(-32768., 32767.) / 32768.; 2]
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn engine_phase_survives_rev_changes_and_loop_wraps() {
        use bevy::audio::Decodable;
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        for car in Vehicle::ROSTER.into_iter().chain([Vehicle::Minion]) {
            let tones = program(&bank, car.sound_programs().0).unwrap();
            let motors = EngineSample::load(&bank, car).unwrap();
            for (mut tone, motor) in tones.into_iter().zip(motors) {
                tone.prepare_loop();
                let (clip, register) = motor.motor.voice(motor.pitch.register(0));
                let mut playback = clip.decoder();
                let mut phase = 0;
                let mut frame = 0;
                while phase < tone.pcm.len() * 4096 * 3 || frame < 13100 {
                    let rev = (frame / 131) % 100;
                    let pitch = motor.pitch.register(rev);
                    register.set(pitch);
                    let [left, right] = reference_loop_frame(&tone, phase, frame as usize);
                    assert_eq!(
                        playback.next(),
                        Some(left * pcm::VabVolume::from_tone(&tone).gain(33, 99))
                    );
                    register.set(0);
                    assert_eq!(
                        playback.next(),
                        Some(right * pcm::VabVolume::from_tone(&tone).gain(33, 99))
                    );
                    if frame >= 4 {
                        phase += usize::from(pitch.min(0x3fff));
                    }
                    frame += 1;
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn road_kill_uses_both_engine_tones_in_battle_and_at_the_reward() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::Drive;
        use bevy::audio::Decodable;

        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let car = Vehicle::RoadKill;
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let tones = program(&bank, car.sound_programs().0).unwrap();
        assert_eq!(tones.len(), 2);
        let stage =
            Stage::load(disc, Level::Rooftops, car, Entry::New(Difficulty::Medium)).unwrap();
        crate::campaign::replace(&mut world, stage);
        let mut drive = world.resource_mut::<Drive>();
        crate::pause::navigate(
            crate::pause::Page::Audio(1),
            crate::pause::Action::Right,
            &mut drive.settings,
        );
        let volume = drive.settings.effects_volume();
        assert_ne!(volume, 33);
        let [x, y, z] = drive.player.position();
        drive.listener = tm1_core::drive::SoundListener::new(
            &drive.player,
            &drive.camera,
            Some([x + 500, y, z, 0, 0]),
        );
        let channels = drive
            .listener
            .channels([x, y, z], drive.settings.effects_volume(), false);
        assert_ne!(channels[0], channels[1]);
        world.run_system_once(update).unwrap();
        let mut battle = world
            .query_filtered::<(&EngineTone, &AudioPlayer<pcm::StereoPcm>), With<Engine>>()
            .iter(&world)
            .map(|(tone, player)| (tone.pitch, player.0.clone()))
            .collect::<Vec<_>>();
        battle.sort_by_key(|(pitch, _)| pitch.span);
        crate::ending::begin(&mut world).unwrap();
        let reward_handles = &world.resource::<Sounds>().screen_engine.clone();
        let reward = world.query_filtered::<(&PlaybackSettings,&AudioPlayer<pcm::StereoPcm>), With<crate::ending::EndingEngine>>()
            .iter(&world).map(|(setting,player)| { assert_eq!(setting.speed,1.); assert_eq!(setting.volume, Volume::Linear(1.)); player.0.clone() }).collect::<Vec<_>>();
        assert_eq!(battle.len(), 2);
        assert_eq!(reward.len(), 2);
        let clips = world.resource::<Assets<pcm::StereoPcm>>();
        for (mut tone, ending) in tones.into_iter().zip(reward_handles) {
            let pitch = EnginePitch::from_tone(&tone);
            let (_, driving) = battle.iter().find(|(p, _)| *p == pitch).unwrap();
            assert!(reward.contains(ending));
            let mut drive = clips.get(driving).unwrap().decoder();
            let mut screen = clips.get(ending).unwrap().decoder();
            tone.prepare_loop();
            let step = usize::from(pitch.register(0));
            for i in 0..tone.pcm.len() * 4096 / step + 1024 {
                let dry = reference_loop_frame(&tone, i.saturating_sub(4) * step, i);
                for (sample, level) in dry.into_iter().zip(channels) {
                    assert_eq!(
                        drive.next(),
                        Some(sample * pcm::VabVolume::from_tone(&tone).gain(level as u8, 99))
                    );
                }
                for sample in reference_loop_frame(
                    &tone,
                    i.saturating_sub(4) * usize::from(pitch.register(30)),
                    i,
                ) {
                    assert_eq!(
                        screen.next(),
                        Some(sample * pcm::VabVolume::from_tone(&tone).gain(volume, 99))
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn road_kill_credits_and_defeat_use_their_disc_signatures() {
        use crate::campaign::{Campaign, Entry, Stage};
        use bevy::audio::Decodable;
        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let stage = Stage::load(
            disc,
            Level::Arena,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let sounds = world.resource::<Sounds>();
        let volume = 57;
        sounds.set_screen_volume(volume);
        let clips = world.resource::<Assets<pcm::StereoPcm>>();
        let table =
            2048 + 0x8016fefc - u32::from_le_bytes(exe[24..28].try_into().unwrap()) as usize;
        for (id, handles) in [
            (0x7f, sounds.signature()),
            (0x74, sounds.lost_game_signature()),
        ] {
            let prog = usize::from(exe[table + id * 6 + 2]);
            let expected = EffectTone::load(&bank, prog).unwrap();
            assert_eq!(handles.len(), expected.len());
            for (handle, tone) in handles.iter().zip(expected) {
                let gain = tone.clip.volume.gain(volume, 99);
                let mut actual = clips.get(handle).unwrap().decoder();
                for raw in tone.clip.unscaled_decoder() {
                    assert_eq!(actual.next(), Some(raw * gain));
                }
                assert_eq!(actual.next(), None);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_loads_engine_layers_for_the_full_roster() {
        let world = crate::contest::menu_world();
        let disc = &world.resource::<crate::campaign::Campaign>().0;
        let cars = [Vehicle::CARD_ORDER.as_slice(), &[Vehicle::Minion]].concat();
        let sounds = Sounds::load(disc, Vehicle::RoadKill, &cars).unwrap();
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        assert_eq!(sounds.rival_engines.len(), cars.len());
        for (tones, car) in sounds.rival_engines.iter().zip(cars) {
            assert_eq!(
                tones.len(),
                program(&bank, car.sound_programs().0).unwrap().len()
            );
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn rival_engine_tones_follow_the_camera_through_their_loop() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::Drive;
        use bevy::audio::Decodable;
        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let bank = disc.read("SND/UACORE.VAB").unwrap();
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            Vehicle::RoadKill,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.insert_resource(crate::smoke::Capture::new(None, true, false));
        for _ in 0..110 {
            world.run_system_once(crate::driving::advance).unwrap();
        }
        let drive = world.resource::<Drive>();
        let (slot, rival) = drive
            .opponents
            .iter()
            .enumerate()
            .filter(|(_, rival)| rival.engine_rev().is_some())
            .max_by_key(|(_, rival)| {
                program(&bank, rival.vehicle().sound_programs().0)
                    .unwrap()
                    .len()
            })
            .unwrap();
        let rev = rival.engine_rev().unwrap();
        let position = rival.position();
        let mut tones = program(&bank, rival.vehicle().sound_programs().0).unwrap();
        let count = tones.len();
        let mut playbacks = Vec::new();
        let mut previous = None;
        for dx in [-500, 500] {
            let mut drive = world.resource_mut::<Drive>();
            let listener = tm1_core::drive::SoundListener::new(
                &drive.player,
                &drive.camera,
                Some([position[0] + dx, position[1], position[2], 0, 0]),
            );
            let channels = listener.channels(position, drive.settings.effects_volume(), false);
            assert_ne!(channels[0], channels[1]);
            if let Some(old) = previous {
                assert_eq!(channels[0], old);
            }
            previous = Some(channels[1]);
            let range = listener.sound_range(position);
            drive.sounds.push(crate::driving::BattleRequest::EngineRev(
                slot + 1,
                crate::driving::EngineMix {
                    rev,
                    channels,
                    range,
                },
            ));
            world.run_system_once(update).unwrap();
            let mut voices = world
                .query::<(Entity, &RivalEngine, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(&world)
                .filter(|(_, e, _)| e.slot == slot)
                .map(|(e, _, p)| (e, p.0.clone()))
                .collect::<Vec<_>>();
            voices.sort_by_key(|(e, _)| e.index());
            assert_eq!(voices.len(), count);
            if playbacks.is_empty() {
                for ((entity, handle), mut tone) in voices.iter().zip(tones.drain(..)) {
                    let step = usize::from(EnginePitch::from_tone(&tone).register(rev));
                    tone.prepare_loop();
                    let played = world
                        .resource::<Assets<pcm::StereoPcm>>()
                        .get(handle)
                        .unwrap()
                        .decoder();
                    playbacks.push((*entity, tone, step, played, 0usize));
                }
            }
            for ((entity, tone, step, played, phase), (current, _)) in
                playbacks.iter_mut().zip(voices)
            {
                assert_eq!(*entity, current);
                let mut audible = false;
                for _ in 0..tone.pcm.len() * 4096 / *step + 32 {
                    for (sample, level) in reference_loop_frame(
                        tone,
                        (*phase).saturating_sub(4 * *step),
                        *phase / *step,
                    )
                    .into_iter()
                    .zip(channels)
                    {
                        let expected =
                            sample * pcm::VabVolume::from_tone(tone).gain(level as u8, 99);
                        assert_eq!(played.next(), Some(expected));
                        audible |= expected != 0.;
                    }
                    *phase += *step;
                }
                assert!(audible);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn repeated_impact_updates_both_channels_without_restarting() {
        use crate::{
            campaign::{Campaign, Entry, Stage},
            driving::{BattleSound, Drive, ExclusiveVoice},
        };
        use bevy::audio::Decodable;
        let mut world = crate::contest::menu_world();
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            Vehicle::Outlaw,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        let impact = |channels| {
            crate::driving::BattleRequest::Play(BattleSound {
                frame: 0,
                range: 0,
                channels,
                exclusive: Some(ExclusiveVoice::ScaledEffect(103)),
                id: 3,
                volume_percent: 84,
            })
        };
        let mut references: Vec<_> = world.resource::<Sounds>().effects[3]
            .iter()
            .map(|tone| (tone.clip.volume, tone.clip.unscaled_decoder()))
            .collect();
        world.resource_mut::<Drive>().sounds = vec![impact([0, 0])];
        world.run_system_once(update).unwrap();
        let voices = |world: &mut World| {
            let mut voices: Vec<_> = world
                .query::<(&OneShot, Entity, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(world)
                .map(|(voice, e, p)| (voice.order, e, p.0.clone()))
                .collect();
            voices.sort_by_key(|v| v.0);
            voices
        };
        let first = voices(&mut world);
        assert_eq!(first.len(), references.len());
        let clips = world.resource::<Assets<pcm::StereoPcm>>();
        let mut played: Vec<_> = first
            .iter()
            .map(|(_, _, h)| clips.get(h).unwrap().decoder())
            .collect();
        for channels in [[0, 0], [47, 19], [25, 51]] {
            world.resource_mut::<Drive>().sounds = vec![impact(channels)];
            if channels != [0; 2] {
                world.run_system_once(update).unwrap();
                assert_eq!(voices(&mut world), first);
            }
            let mut audible = false;
            for ((volume, reference), played) in references.iter_mut().zip(&mut played) {
                for _ in 0..1024 {
                    for level in channels {
                        let dry = reference.next().unwrap();
                        audible |= dry != 0.;
                        assert_eq!(played.next().unwrap(), dry * volume.gain(level as u8, 84));
                    }
                }
            }
            assert!(audible);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_machine_guns_release_after_cooldown_and_overheat() {
        use crate::campaign::{Campaign, Entry, Stage};
        let mut world = crate::contest::menu_world();
        let car = Vehicle::RoadKill;
        let stage = Stage::load(
            &world.resource::<Campaign>().0,
            Level::Warehouse,
            car,
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(50));
        let step = |world: &mut World, held| {
            let mut keys = ButtonInput::default();
            if held {
                keys.press(KeyCode::Space);
            }
            world.insert_resource(keys);
            world.run_system_once(crate::driving::advance).unwrap();
            world.run_system_once(update).unwrap();
            let mut voices = world
                .query::<(Entity, &OneShot)>()
                .iter(world)
                .filter(|(_, shot)| {
                    shot.key == Some(crate::driving::ExclusiveVoice::MachineGuns(car))
                })
                .map(|(entity, _)| entity)
                .collect::<Vec<_>>();
            voices.sort();
            voices
        };
        step(&mut world, false);
        let first = step(&mut world, true);
        assert_eq!(first.len(), 2);
        for _ in 0..3 {
            assert_eq!(step(&mut world, false), first);
        }
        assert!(step(&mut world, false).is_empty());
        let second = step(&mut world, true);
        assert_eq!(second.len(), 2);
        for _ in 0..10 {
            assert_eq!(step(&mut world, true), second);
        }
        for _ in 0..200 {
            if step(&mut world, true).is_empty()
                && world
                    .resource::<crate::driving::Drive>()
                    .player
                    .gun_overheated()
            {
                return;
            }
        }
        panic!("expected the machine gun voices to stop during overheat");
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn turbo_tones_follow_fuel_and_duel_slot_order() {
        use crate::campaign::{Campaign, Entry, Stage};
        use crate::driving::ExclusiveVoice::Turbo;
        use KeyCode::*;
        let mut world = crate::contest::menu_world();
        let disc = &world.resource::<Campaign>().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let record = 2048 + (0x8016fefc - base) as usize + 0x82 * 6;
        assert_eq!(&exe[record..record + 4], &[0, 0, TURBO_PROGRAM as u8, 0]);
        let cars = [Vehicle::YellowJacket, Vehicle::Outlaw];
        let stage = Stage::load(
            disc,
            Level::Warehouse,
            cars[0],
            Entry::New(Difficulty::Medium),
        )
        .unwrap();
        crate::campaign::replace(&mut world, stage);
        world.insert_resource(crate::smoke::Capture::new(None, false, false));
        world
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(50));
        let solo = |world: &mut World, held| {
            let mut keys = ButtonInput::default();
            if held {
                keys.press(KeyT);
            }
            world.insert_resource(keys);
            world.run_system_once(crate::driving::advance).unwrap();
            world.run_system_once(update).unwrap();
            let mut voices = world
                .query::<(Entity, &OneShot, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(world)
                .filter(|(_, voice, _)| voice.key == Some(Turbo))
                .map(|(e, _, source)| {
                    use bevy::audio::{Decodable, Source};
                    let clip = world
                        .resource::<Assets<pcm::StereoPcm>>()
                        .get(&source.0)
                        .unwrap();
                    (e, clip.decoder().total_duration().is_none())
                })
                .collect::<Vec<_>>();
            voices.sort();
            voices
        };
        solo(&mut world, false); // Acquire window focus before the first driving step.
        let voices = solo(&mut world, true);
        assert_eq!(voices.len(), 4);
        assert_eq!(voices.iter().filter(|v| v.1).count(), 3);
        // Keep the four live decoders across mix changes to catch both a
        // restarted attack and scaling after the original integer reductions.
        use bevy::audio::Decodable;
        let mut query =
            world.query::<(&OneShot, &AudioPlayer<pcm::StereoPcm>, &PlaybackSettings)>();
        let mut playing: Vec<_> = query
            .iter(&world)
            .filter(|(voice, _, _)| voice.key == Some(Turbo))
            .map(|(voice, player, settings)| {
                assert_eq!(settings.volume, Volume::Linear(1.));
                (
                    voice.order,
                    world
                        .resource::<Assets<pcm::StereoPcm>>()
                        .get(&player.0)
                        .unwrap()
                        .decoder(),
                )
            })
            .collect();
        playing.sort_by_key(|(order, _)| *order);
        let mut reference: Vec<_> = world
            .resource::<Sounds>()
            .turbo
            .iter()
            .map(|tone| (tone.clip.volume, tone.clip.unscaled_decoder()))
            .collect();
        for (direction, steps) in [
            (crate::pause::Action::Right, 7),
            (crate::pause::Action::Left, 14),
            (crate::pause::Action::Right, 4),
        ] {
            let settings = &mut world.resource_mut::<crate::driving::Drive>().settings;
            for _ in 0..steps {
                crate::pause::navigate(crate::pause::Page::Audio(1), direction, settings);
            }
            let volume = settings.effects_volume();
            assert_eq!(solo(&mut world, true), voices);
            for ((_, actual), (attr, dry)) in playing.iter_mut().zip(&mut reference) {
                let gain = attr.gain(volume, 99);
                for _ in 0..512 {
                    assert_eq!(actual.next(), dry.next().map(|v| v * gain));
                }
            }
        }
        assert_eq!(solo(&mut world, true), voices);
        assert!(solo(&mut world, false).is_empty());
        assert_eq!(solo(&mut world, true).len(), 4);
        for _ in 0..130 {
            solo(&mut world, true);
        }
        assert!(solo(&mut world, true).is_empty());

        crate::duel::start(
            &mut world,
            Level::Warehouse,
            cars,
            Difficulty::Medium,
            [0; 2],
        )
        .unwrap();
        let duel = |world: &mut World, keys: &[KeyCode]| {
            let mut input = ButtonInput::default();
            for key in keys {
                input.press(*key);
            }
            world.insert_resource(input);
            world.run_system_once(crate::duel::advance).unwrap();
            world.run_system_once(duel::mix).unwrap();
            world
                .query::<(Entity, &duel::Voice, &AudioPlayer<pcm::StereoPcm>)>()
                .iter(world)
                .filter(|(_, voice, _)| voice.is_turbo())
                .map(|(entity, _, source)| {
                    use bevy::audio::{Decodable, Source};
                    (
                        entity,
                        world
                            .resource::<Assets<pcm::StereoPcm>>()
                            .get(&source.0)
                            .unwrap()
                            .decoder()
                            .total_duration()
                            .is_none(),
                    )
                })
                .collect::<Vec<_>>()
        };
        // Both requests address sound 0x82; the second slot can stop the first slot.
        assert!(duel(&mut world, &[KeyT]).is_empty());
        let voices = duel(&mut world, &[KeyT, Slash]);
        assert_eq!(voices.len(), 4);
        assert_eq!(voices.iter().filter(|v| v.1).count(), 3);
        assert_eq!(duel(&mut world, &[KeyT, Slash]), voices);
        assert_eq!(duel(&mut world, &[Slash]).len(), 4);
        assert!(duel(&mut world, &[]).is_empty());
    }
}
