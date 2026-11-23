use crate::{driving, music, render, sfx};
use anyhow::{Result, ensure};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use std::time::Duration;
use tm1_assets::{disc::Disc, scene::Dmd, texture::Vram};
use tm1_core::drive::{Difficulty, Level, PlayerVehicle, Vehicle};

#[derive(Resource)]
pub struct Campaign(pub Disc);

pub(crate) fn round_ticks(world: &World) -> u32 {
    timer_ticks(
        world.resource::<Time<Real>>(),
        world.get_resource::<bevy::time::TimeUpdateStrategy>(),
        world
            .get_resource::<crate::smoke::Capture>()
            .is_some_and(|capture| capture.demo),
    )
}

pub(crate) fn timer_ticks(
    time: &Time<Real>,
    strategy: Option<&bevy::time::TimeUpdateStrategy>,
    demo: bool,
) -> u32 {
    if demo {
        return 1;
    }
    let elapsed = if matches!(strategy, Some(bevy::time::TimeUpdateStrategy::Automatic)) {
        time.startup().elapsed()
    } else {
        time.elapsed()
    };
    // PS1 root counter 2 runs at the system clock divided by eight.
    (elapsed.as_nanos() * 4_233_600 / 1_000_000_000) as u32
}

#[derive(Component)]
pub struct StageEntity;

#[derive(Default, Clone, Copy)]
pub struct PasswordModes {
    pub infinite_weapons: bool,
    pub god_mode: bool,
    pub minion: bool,
    pub helicopter: bool,
}

pub enum Entry<'a> {
    New(Difficulty),
    Continue(&'a driving::Drive),
}
pub struct Stage {
    arena: render::Arena,
    pub(crate) drive: driving::Drive,
    pub(crate) card_angles: Vec<i32>,
    music: music::Music,
    sounds: sfx::Sounds,
    clear: ClearColor,
}
pub(crate) fn stage_clear_color(level: Level) -> ClearColor {
    let command = match level {
        Level::Arena | Level::Arena5 => [40, 40, 40],
        Level::Rooftops => [0, 20, 0],
        Level::Cyburbia => [110, 110, 100],
        Level::CityPark => [75, 60, 30],
        Level::Freeway => [60, 45, 20],
        Level::Warehouse => [75, 70, 55],
    };
    ClearColor(render::flat_color(command, 255))
}

fn load_cockpit(
    disc: &Disc,
    dmd: &Dmd<'_>,
    vehicle: Vehicle,
) -> Result<tm1_assets::screen::Cockpit> {
    let dash = match vehicle {
        Vehicle::SweetTooth => "ICE_DASH",
        Vehicle::YellowJacket => "CAB_DASH",
        Vehicle::Darkside => "SEMI_DAS",
        Vehicle::Outlaw => "COP_DASH",
        Vehicle::Thumper => "LOWRIDER",
        Vehicle::CrimsonFury => "LAMBO_DA",
        Vehicle::PitViper => "DUNEB_DA",
        Vehicle::Warthog => "HUMVEE_D",
        Vehicle::MrGrimm => "HARLEY_D",
        Vehicle::Spectre => "VETTE_DA",
        Vehicle::Hammerhead => "MONSTERT",
        Vehicle::RoadKill => "MADMAX_D",
        Vehicle::Minion => anyhow::bail!("Minion has no player cockpit"),
    };
    tm1_assets::screen::Cockpit::load(
        &disc.read(&format!("UADASH/{dash}.TMS"))?,
        dmd.cockpit_rectangles()?,
        dmd.steering_wheel()?,
        vehicle == Vehicle::MrGrimm,
    )
}

pub(crate) fn arena_art(
    disc: &Disc,
    level: Level,
    vehicle: Vehicle,
    dmd: &Dmd<'_>,
    textures: Vram,
    scene: tm1_assets::scene::Scene,
    player_mesh: [tm1_assets::scene::Scene; 3],
    opponent_mesh: Vec<[tm1_assets::scene::Scene; 3]>,
    contacts: &tm1_core::drive::CollisionScene,
) -> Result<render::Arena> {
    let pedestrians = contacts
        .pedestrian_models()
        .into_iter()
        .map(|root| dmd.actor_frames(root))
        .collect::<Result<Vec<_>>>()?;
    render::Arena::new(
        dmd,
        tm1_assets::screen::Fonts::load(disc)?,
        crate::pause::Art::load(disc)?,
        load_cockpit(disc, dmd, vehicle)?,
        scene,
        (!matches!(level, Level::Arena | Level::Arena5))
            .then(|| dmd.battle_sky())
            .transpose()?,
        player_mesh,
        opponent_mesh,
        [
            Some(dmd.battle_model(201)?),
            dmd.roots
                .iter()
                .any(|r| r.1 == 312)
                .then(|| dmd.battle_model(312))
                .transpose()?,
        ],
        [dmd.merc_frames(750)?, dmd.merc_frames(752)?],
        pedestrians,
        dmd.explosion_frames()?,
        dmd.spark_frames()?,
        dmd.flame_frames()?,
        dmd.burn_frames()?,
        [dmd.smoke_frames(false)?, dmd.smoke_frames(true)?],
        dmd.special_bullet_frames()?,
        dmd.fragment_frames()?,
        dmd.drop_sprites()?,
        dmd.contrail_frames()?,
        dmd.dashboard_icons()?,
        dmd.radar_arrows()?,
        textures,
    )
}

impl Stage {
    #[cfg(test)]
    pub fn load(disc: &Disc, level: Level, vehicle: Vehicle, entry: Entry<'_>) -> Result<Self> {
        Self::load_with_clock(disc, level, vehicle, entry, || 1)
    }

    pub fn from_world(
        world: &World,
        level: Level,
        vehicle: Vehicle,
        entry: Entry<'_>,
    ) -> Result<Self> {
        Self::load_with_clock(
            &world.resource::<Campaign>().0,
            level,
            vehicle,
            entry,
            || round_ticks(world),
        )
    }

    fn load_with_clock(
        disc: &Disc,
        level: Level,
        vehicle: Vehicle,
        entry: Entry<'_>,
        ticks: impl Fn() -> u32,
    ) -> Result<Self> {
        let stage = level.database(vehicle);
        let data = disc.read(&format!("UADMD/{stage}.DMD"))?;
        let textures = Vram::from_tms(&disc.read(&format!("UADMD/{stage}.TMS"))?)?;
        let dmd = Dmd::parse(&data)?;
        ensure!(dmd.version == textures.version, "DMD/TMS version mismatch");
        let scene = dmd.battle_geometry(matches!(level, Level::Arena | Level::Arena5))?;
        for issue in &scene.omitted_nodes {
            eprintln!("Preview omission: {issue}");
        }
        let (mut seed, mut deck) = match entry {
            Entry::Continue(drive) => (drive.player.rng_state(), drive.deck),
            Entry::New(_) => (ticks(), Default::default()),
        };
        let opponent_kinds = level.opponents(vehicle, &mut deck, &mut seed);
        // screenDrawCard takes one BIOS random value per settled opponent card,
        // after shellSelectCards and before the new stage initializes its cars.
        let card_angles = opponent_kinds
            .iter()
            .map(|_| {
                let random = tm1_core::drive::bios_rand(&mut seed) as i32;
                if random & 8 == 0 {
                    random & 6
                } else {
                    -(random & 6)
                }
            })
            .collect();
        let headlights = !matches!(level, Level::Cyburbia | Level::Rooftops);
        let player_mesh = dmd.vehicle_bodies_with_lights(vehicle as u16, headlights)?;
        let mut models = opponent_kinds.clone();
        if level == Level::Rooftops {
            models.push(Vehicle::Minion);
        }
        let opponent_mesh = models
            .iter()
            .map(|&kind| dmd.vehicle_bodies_with_lights(kind as u16, headlights))
            .collect::<Result<Vec<_>>>()?;
        let mut contacts = tm1_core::drive::CollisionScene::for_opponents(
            &data,
            scene.root_id,
            player_mesh[0].root_id,
            &opponent_mesh
                .iter()
                .map(|variants| variants[0].root_id)
                .collect::<Vec<_>>(),
        )
        .ok_or_else(|| anyhow::anyhow!("unsupported arena collision graph"))?;
        contacts.start_pedestrians(level, &mut seed);
        let player = match entry {
            Entry::Continue(drive) => drive.player.for_next_stage_seeded(level, seed),
            Entry::New(difficulty) => PlayerVehicle::contest(vehicle, level, seed, difficulty),
        };
        let arena = arena_art(
            disc,
            level,
            vehicle,
            &dmd,
            textures,
            scene,
            player_mesh,
            opponent_mesh,
            &contacts,
        )?;
        let helicopter_route = match entry {
            Entry::Continue(previous) => previous.helicopter_route(),
            Entry::New(_) => Default::default(),
        };
        let mut drive = driving::Drive::new(
            player,
            contacts,
            &opponent_kinds,
            helicopter_route,
            ticks,
            matches!(entry, Entry::New(_)),
        );
        drive.deck = deck;
        if let Entry::Continue(previous) = entry {
            drive.camera = tm1_core::drive::VehicleCamera::new(&drive.player, previous.camera.mode);
            drive.mirror = previous.mirror;
            drive.radar = previous.radar;
            drive.list_cars = previous.list_cars;
            drive.helicopter_unlocked = previous.helicopter_unlocked;
            if previous.helicopter_pose().is_some() {
                drive.toggle_helicopter();
            }
        }
        Ok(Self {
            card_angles,
            arena,
            drive,
            music: music::Music::level(disc, level)?,
            sounds: sfx::Sounds::load(disc, vehicle, &models)?,
            clear: stage_clear_color(level),
        })
    }
    pub fn apply_passwords(&mut self, modes: PasswordModes) {
        self.drive.helicopter_unlocked = modes.helicopter;
        self.drive
            .player
            .set_weapon_and_damage_modes(modes.infinite_weapons, modes.god_mode);
        if modes.minion {
            self.drive.start_with_minion();
        }
    }
    pub fn install(mut self, world: &mut World) {
        if let Some(prev) = world.get_resource::<driving::Drive>() {
            self.drive.contacts.continue_stage_state(&prev.contacts);
            self.drive.settings = prev.settings;
            self.drive.player.set_pad_layout(prev.settings.layout);
        }
        self.drive.begin_battle(round_ticks(world));
        crate::contest::remember_battle_cars(world, &[self.drive.player.vehicle()]);
        world.insert_resource(self.arena);
        world.insert_resource(self.drive);
        world.init_resource::<Assets<music::CdTrack>>();
        world.insert_resource(self.music);
        world.init_resource::<Assets<AudioSource>>();
        world.init_resource::<Assets<sfx::pcm::StereoPcm>>();
        world.insert_resource(self.sounds);
        world.insert_resource(self.clear);
    }
}

#[derive(Component)]
struct LostGame {
    load_remaining: Duration,
}

fn defeat(world: &mut World, volume: u8) -> Result<()> {
    let delta = world.resource::<Time<Real>>().delta();
    if let Ok(mut screen) = world.query::<&mut LostGame>().single_mut(world) {
        screen.load_remaining = screen.load_remaining.saturating_sub(delta);
        if screen.load_remaining.is_zero() {
            crate::contest::open(world)?;
        }
        return Ok(());
    }

    // shellLoadCarsDatabase runs before screenPlayCinema stops the defeat voices.
    let disc = &world.resource::<Campaign>().0;
    let mut sectors = 0u32;
    for file in [
        "UADMD/CARS.TMS",
        "UADMD/CARS.DMD",
        "UASCREEN/SCRNC.TIM",
        "UASCREEN/SCRNF.TIM",
        "UASCREEN/SCRND.TIM",
        "UASCREEN/SCRNFC1.TIM",
    ] {
        let entry = disc
            .files
            .get(file)
            .ok_or_else(|| anyhow::anyhow!("missing title asset {file}"))?;
        sectors += entry.size.div_ceil(2048);
    }
    let load_remaining = Duration::from_secs_f64(f64::from(sectors) / 150.);
    let sounds = world.resource::<sfx::Sounds>();
    sounds.set_screen_volume(volume);
    let engine = sounds.ending_engine();
    let signature = sounds.lost_game_signature().to_vec();
    for tone in engine {
        world.spawn((StageEntity, tone));
    }
    world.spawn((StageEntity, LostGame { load_remaining }));
    for tone in signature {
        world.spawn((StageEntity, AudioPlayer(tone), PlaybackSettings::DESPAWN));
    }
    Ok(())
}

pub fn advance(world: &mut World) {
    let drive = world.resource::<driving::Drive>();
    if drive.lives == 0 {
        let volume = drive.settings.effects_volume();
        if let Err(err) = defeat(world, volume) {
            error!("Returning to the title after defeat failed: {err:#}");
            world.write_message(AppExit::error());
        }
        return;
    }
    if drive.pause == Some(crate::pause::Page::Aborted) && drive.abort_wait.finished {
        if let Err(err) = crate::contest::open(world) {
            error!("Opening title after abort failed: {err:#}");
            world.write_message(AppExit::error());
        }
        return;
    }
    if drive.pause.is_some() || !drive.stage_finished() {
        return;
    }
    if drive.player.level() == Level::Arena5 {
        // This contest ends at the title, without playing the Rooftops ending.
        if let Err(e) = crate::contest::open(world) {
            error!("Opening contest screen failed: {e:#}");
            world.write_message(AppExit::error());
        }
        return;
    }
    let Some(level) = drive.player.level().next() else {
        if let Err(e) = crate::ending::begin(world) {
            error!("Loading ending failed: {e:#}");
            world.write_message(AppExit::error());
        }
        return;
    };
    advance_to(world, level);
}

pub fn advance_to(world: &mut World, level: Level) {
    let drive = world.resource::<driving::Drive>();
    let lives = drive.lives;
    let mut stage =
        match Stage::from_world(world, level, drive.player.vehicle(), Entry::Continue(drive)) {
            Ok(stage) => stage,
            Err(err) => {
                error!("Loading {level:?} failed: {err:#}");
                world.write_message(AppExit::error());
                return;
            }
        };
    stage.drive.lives = lives;
    let from_ending = level == Level::Arena5;
    if let Err(err) = crate::transition::begin(world, stage, !from_ending, from_ending, None) {
        error!("Loading stage introduction failed: {err:#}");
        world.write_message(AppExit::error());
    }
}

pub fn clear_stage(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<StageEntity>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
}

pub fn replace(world: &mut World, stage: Stage) {
    clear_stage(world);
    stage.install(world);
    world
        .run_system_once(render::setup)
        .expect("stage render setup");
    world
        .run_system_once(music::start)
        .expect("stage music setup");
    world
        .run_system_once(sfx::start)
        .expect("stage sound setup");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn stage_clear_matches_both_retail_draw_environments() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        for (level, entry, stop) in [
            (Level::Arena, 0x8014b7a4, 0x8014b8ec),
            (Level::Arena5, 0x8014b7a4, 0x8014b8ec),
            (Level::Warehouse, 0x8014b934, 0x8014ba84),
            (Level::Freeway, 0x8014bacc, 0x8014bc1c),
            (Level::CityPark, 0x8014bc68, 0x8014bdb0),
            (Level::Cyburbia, 0x8014bdfc, 0x8014bf28),
            (Level::Rooftops, 0x8014bf6c, 0x8014c0b4),
        ] {
            let mut cpu = crate::reference_cpu::Cpu::new(&exe);
            // Isolate the stage's background setup from camera and light creation.
            // Those helpers return writable records. Execute viewSetBgColor in full.
            for at in (entry..stop).step_by(4) {
                let op = cpu.word(at);
                if op >> 26 == 3 {
                    let target = 0x80000000 | ((op & 0x03ffffff) << 2);
                    if target != 0x8010ae48 {
                        cpu.put(target, &0x03e00008u32.to_le_bytes());
                        cpu.put(target + 4, &0x3c028001u32.to_le_bytes());
                    }
                }
            }
            let views = [0x80020000u32, 0x80021000];
            cpu.put(0x8018bf98, &2u32.to_le_bytes());
            for (i, db) in views.iter().enumerate() {
                cpu.put(0x8018c514 + i as u32 * 4, &db.to_le_bytes());
                cpu.put(*db, &[0xff; 0xb0]);
            }
            cpu.call_until(entry, &[], stop);
            for db in views {
                for offset in [0x28, 0xa8] {
                    let [_, r, g, b] = cpu.word(db + offset).to_le_bytes();
                    let expand = |c: u8| {
                        let v = c / 8;
                        v * 8 + v / 4
                    };
                    assert_eq!(
                        stage_clear_color(level).0,
                        Color::srgb_u8(expand(r), expand(g), expand(b)),
                        "{level:?}"
                    );
                }
            }
        }
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn arena_five_registration_precedes_the_level_counter_reseed() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let level = Level::Arena5;
        let vehicle = Vehicle::YellowJacket;
        for (deal_seed, level_seed) in [(1, 73), (0x54321, 905)] {
            let ticks = std::cell::RefCell::new([deal_seed, level_seed].into_iter());
            let stage = Stage::load_with_clock(
                &disc,
                level,
                vehicle,
                Entry::New(Difficulty::Medium),
                || ticks.borrow_mut().next().unwrap(),
            )
            .unwrap();
            let mut seed = deal_seed;
            let rivals = level.opponents(vehicle, &mut Default::default(), &mut seed);
            for _ in &rivals {
                tm1_core::drive::bios_rand(&mut seed);
            }
            let mut cpu = crate::reference_cpu::Cpu::new(&exe);
            for (i, kind) in std::iter::once(&vehicle).chain(&rivals).enumerate() {
                let cs = 0x80008000 + 0x200 * i as u32;
                cpu.put(0x8019edd8 + 4 * i as u32, &(*kind as u32).to_le_bytes());
                cpu.put(0x8019ee0c + 4 * i as u32, &cs.to_le_bytes());
                if i > 0 {
                    cpu.put(
                        0x801f7264 + 4 * (i as u32 - 1),
                        &(*kind as u32).to_le_bytes(),
                    );
                }
            }
            cpu.put(0x8018c336, &6u16.to_le_bytes());
            for (at, value) in [
                (0x8018c648, vehicle as u32),
                (0x8018c66c, 5),
                (0x8018c3b4, level as u32),
                (0x8018bf9c, 70),
                (0x8018c640, 1),
            ] {
                cpu.put(at, &value.to_le_bytes());
            }
            cpu.put(0x8018c624, &[1]);
            cpu.put(0x8019ee40 + 0x184, &1i32.to_le_bytes()); // uaInit enables damage.
            cpu.put(0x80170238, &3i16.to_le_bytes());
            cpu.put(0x8017023a, &20i16.to_le_bytes());
            cpu.call(0x80150224, &[Difficulty::Medium as u32]);
            // Keep the database and level schedulers. Skip graphics, reverb and view parenting.
            for at in [0x800ff38c, 0x8010abc8, 0x801414e0, 0x80141c64, 0x801055dc] {
                cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
            }
            // Arena DB specifics set up rendering and the arena's trigger-point table.
            cpu.put(
                0x8014b7a4,
                &(0x08000000u32 | ((0x80125bc0u32 >> 2) & 0x03ffffff)).to_le_bytes(),
            );
            cpu.put(0x8014b7a8, &[0; 4]);
            cpu.random_state = seed;
            cpu.call_until(0x8014b498, &[level as u32], 0x80109bdc);
            for (slot, ai) in stage.drive.opponents.iter().enumerate() {
                let base = 0x8019f7a8 + slot as u32 * 0x560;
                assert_eq!(ai.vehicle() as u32, cpu.word(base + 0x3c));
                assert_eq!(
                    ai.position(),
                    [0, 4, 8].map(|offset| cpu.word(base + 0x444 + offset) as i32)
                );
            }
            cpu.random_state = level_seed;
            cpu.call_until(0x8014c100, &[level as u32], 0x80109bdc);
            for (offset, byte) in stage.drive.player.state().iter().enumerate() {
                assert_eq!(
                    *byte,
                    cpu.ram[0x19ee40 + offset],
                    "battle player offset {offset:x}"
                );
            }
            let mut expected = cpu.random_state;
            // Drive construction prepares the first set of windshield marks.
            for _ in 0..12 {
                tm1_core::drive::bios_rand(&mut expected);
            }
            assert_eq!(stage.drive.player.rng_state(), expected);
        }
    }
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn cockpit_art_and_steering_for_the_full_roster() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let dmd = Dmd::parse(&data).unwrap();
        for vehicle in Vehicle::ROSTER {
            let cockpit = load_cockpit(&disc, &dmd, vehicle).unwrap();
            let straight = cockpit.frame([0, 4096]);
            assert_eq!(
                straight[4 * (100 * 320 + 160) + 3],
                0,
                "windshield {vehicle:?}"
            );
            assert!(straight.chunks_exact(4).any(|p| p[3] == 255));
            assert_ne!(
                straight,
                cockpit.frame([1662, 3744]),
                "steering {vehicle:?}"
            );
        }
    }
}
