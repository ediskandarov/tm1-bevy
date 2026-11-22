use crate::{driving, music, render, sfx};
use anyhow::{Result, ensure};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use tm1_assets::{disc::Disc, scene::Dmd, texture::Vram};
use tm1_core::drive::{Difficulty, Level, PlayerVehicle, Vehicle};

#[derive(Resource)]
pub struct Campaign(pub Disc);

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
    drive: driving::Drive,
    music: music::Music,
    sounds: sfx::Sounds,
    clear: ClearColor,
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
        vehicle == Vehicle::MrGrimm,
    )
}

impl Stage {
    pub fn load(disc: &Disc, level: Level, vehicle: Vehicle, entry: Entry<'_>) -> Result<Self> {
        let stage = level.database(vehicle);
        let data = disc.read(&format!("UADMD/{stage}.DMD"))?;
        let textures = Vram::from_tms(&disc.read(&format!("UADMD/{stage}.TMS"))?)?;
        let dmd = Dmd::parse(&data)?;
        ensure!(dmd.version == textures.version, "DMD/TMS version mismatch");
        let mut scene = dmd.extract_kind(1)?;
        if matches!(level, Level::Arena | Level::Arena5) {
            scene.polygons.extend(dmd.extract_kind(3)?.polygons);
        }
        for issue in &scene.omitted_nodes {
            eprintln!("Preview omission: {issue}");
        }
        let (mut seed, mut deck) = match entry {
            Entry::Continue(drive) => (drive.player.rng_state(), drive.deck),
            Entry::New(_) => (1, Default::default()),
        };
        let opponent_kinds = level.opponents(vehicle, &mut deck, &mut seed);
        let player = match entry {
            Entry::Continue(drive) => drive.player.for_next_stage_seeded(level, seed),
            Entry::New(difficulty) => {
                PlayerVehicle::contest(&disc.read("SIPS_600.07")?, vehicle, level, seed, difficulty)
                    .ok_or_else(|| anyhow::anyhow!("missing JP SDK rotation table"))?
            }
        };
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
            tm1_assets::screen::Fonts::load(disc)?,
            crate::pause::Art::load(disc)?,
            load_cockpit(disc, &dmd, vehicle)?,
            scene,
            (!matches!(level, Level::Arena | Level::Arena5))
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
            dmd.flame_frames()?,
            dmd.contrail_frames()?,
            dmd.dashboard_icons()?,
            dmd.radar_arrows()?,
            textures,
        )?;
        let mut drive = driving::Drive::new(player, contacts, &opponent_kinds);
        drive.deck = deck;
        if let Entry::Continue(previous) = entry {
            drive.camera = tm1_core::drive::VehicleCamera::new(&drive.player, previous.camera.mode);
            drive.mirror = previous.mirror;
            drive.helicopter_unlocked = previous.helicopter_unlocked;
            if previous.helicopter_pose().is_some() {
                drive.toggle_helicopter();
            }
        }
        Ok(Self {
            arena,
            drive,
            music: music::Music::level(disc, level)?,
            sounds: sfx::Sounds::load(disc, vehicle, &models)?,
            clear: ClearColor(match level {
                Level::Arena | Level::Arena5 => Color::srgb(0.035, 0.045, 0.065),
                Level::Rooftops => Color::srgb_u8(0, 20, 0),
                Level::Cyburbia => Color::srgb_u8(110, 110, 100),
                Level::CityPark => Color::srgb_u8(75, 60, 30),
                Level::Freeway => Color::srgb_u8(60, 45, 20),
                Level::Warehouse => Color::srgb_u8(75, 70, 55),
            }),
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
            self.drive.settings = prev.settings;
            self.drive.player.set_pad_layout(prev.settings.layout);
        }
        world.insert_resource(self.arena);
        world.insert_resource(self.drive);
        world.insert_resource(self.music);
        world.insert_resource(self.sounds);
        world.insert_resource(self.clear);
    }
}

#[derive(Component)]
struct LostGame;

pub fn advance(world: &mut World) {
    let drive = world.resource::<driving::Drive>();
    if drive.lives == 0 {
        let gain = drive.settings.effects_gain();
        if world
            .query_filtered::<Entity, With<LostGame>>()
            .iter(world)
            .next()
            .is_none()
        {
            let sounds = world.resource::<sfx::Sounds>();
            let engine = sounds.ending_engine(gain);
            let signature = sounds.lost_game_signature();
            world.spawn((StageEntity, LostGame, engine));
            world.spawn((
                StageEntity,
                signature,
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.44 * gain)),
            ));
        } else if let Err(err) = crate::contest::open(world) {
            error!("Opening title after defeat failed: {err:#}");
            world.write_message(AppExit::error());
        }
        return;
    }
    if drive.pause == Some(crate::pause::Page::Aborted) && drive.abort_delay.is_zero() {
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
    let disc = &world.resource::<Campaign>().0;
    let lives = drive.lives;
    let mut stage = match Stage::load(disc, level, drive.player.vehicle(), Entry::Continue(drive)) {
        Ok(stage) => stage,
        Err(err) => {
            error!("Loading {level:?} failed: {err:#}");
            world.write_message(AppExit::error());
            return;
        }
    };
    stage.drive.lives = lives;
    replace(world, stage);
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
    fn cockpit_art_and_steering_for_the_full_roster() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let dmd = Dmd::parse(&data).unwrap();
        for vehicle in Vehicle::ROSTER {
            let cockpit = load_cockpit(&disc, &dmd, vehicle).unwrap();
            let straight = cockpit.frame(0);
            assert_eq!(
                straight[4 * (100 * 320 + 160) + 3],
                0,
                "windshield {vehicle:?}"
            );
            assert!(straight.chunks_exact(4).any(|p| p[3] == 255));
            assert_ne!(straight, cockpit.frame(24), "steering {vehicle:?}");
        }
    }
}
