use crate::{driving, music, render, sfx};
use anyhow::{Result, ensure};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use tm1_assets::{disc::Disc, scene::Dmd, texture::Vram};
use tm1_core::drive::{Difficulty, Level, PlayerVehicle, Vehicle};

#[derive(Resource)]
pub struct Campaign(pub Disc);

#[derive(Component)]
pub struct StageEntity;

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
            dmd.radar_arrows()?,
            textures,
        )?;
        let mut drive = driving::Drive::new(player, contacts, &opponent_kinds);
        drive.deck = deck;
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
    pub fn install(self, world: &mut World) {
        world.insert_resource(self.arena);
        world.insert_resource(self.drive);
        world.insert_resource(self.music);
        world.insert_resource(self.sounds);
        world.insert_resource(self.clear);
    }
}

pub fn advance(world: &mut World) {
    let drive = world.resource::<driving::Drive>();
    if !drive.stage_finished() {
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
