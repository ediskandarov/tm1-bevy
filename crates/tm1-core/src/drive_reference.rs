use super::*;
#[path = "../tests/support/mips.rs"]
mod mips;
use mips::Cpu;
#[path = "../tests/support/targets.rs"]
mod targets;
use targets::register_retail_targets;

const PLAYER: u32 = 0x8019ee40;
const POLICE: u32 = 0x8019f7a8;
// Fixture objects and traversal stacks must stay below the DMD load address
// (0x800188b8). The larger Cyburbia databases extend past 0x800e9000.
const MISSILE_CS: u32 = 0x80002000;
const MISSILE_AUX: u32 = 0x80003000;
const VIEW_OUTPUT: u32 = 0x80005000;
const RENDER_SCRATCH: u32 = 0x80006000;
const HD_STACK: u32 = 0x8000a000;
const HDP_STACK: u32 = 0x8000c000;
const EXTRA_MODELS: u32 = 0x80010000;
const TAXI_CS: u32 = 0x80009000;
const POLICE_CS: u32 = 0x80009200;
const WORLD: u32 = 0x80009400;
fn put(cpu: &mut Cpu, at: u32, v: u32) {
    cpu.put(at, &v.to_le_bytes());
}

struct Reference {
    pedestrian_nodes: Vec<(u32, u32)>,
    cpu: Cpu,
    player: Taxi,
    police: PoliceCar,
    scene: CollisionScene,
}
impl Reference {
    fn new() -> Self {
        Self::with_vehicle(Vehicle::YellowJacket)
    }
    fn with_vehicle(vehicle: Vehicle) -> Self {
        Self::build(vehicle, false)
    }
    fn build(vehicle: Vehicle, retail_particles: bool) -> Self {
        Self::on_level(vehicle, retail_particles, Level::Arena)
    }
    fn on_level(vehicle: Vehicle, retail_particles: bool, level: Level) -> Self {
        Self::on_difficulty(vehicle, retail_particles, level, Difficulty::Medium)
    }
    fn on_difficulty(
        vehicle: Vehicle,
        retail_particles: bool,
        level: Level,
        difficulty: Difficulty,
    ) -> Self {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc
            .read(&format!("UADMD/{}.DMD", level.database(vehicle)))
            .unwrap();
        let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
        let opponent = match level {
            Level::Arena => Vehicle::Outlaw,
            Level::Warehouse => vehicle.warehouse_opponents()[0],
            Level::Freeway
            | Level::CityPark
            | Level::Cyburbia
            | Level::Rooftops
            | Level::Arena5 => level.opponents(vehicle, &mut ArenaDeck::default(), &mut 1)[0],
        };
        let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let mut cpu = Cpu::new(&exe);
        cpu.call(0x80150224, &[difficulty as u32]);
        let scene =
            CollisionScene::from_dmd(&data, root(1), root(vehicle as u16), root(opponent as u16))
                .unwrap();
        let mut player = PlayerVehicle::contest(&exe, vehicle, level, 1, difficulty).unwrap();
        let mut police = PoliceCar::for_vehicle(&mut player, opponent);
        police.state[0x40] = 1;
        cpu.put(0x800188b8, &data);
        let pedestrian_nodes = register_retail_targets(&mut cpu, &roots);
        for (at, v) in [
            (0x8018c4b0, WORLD),
            (0x8018bf14, POLICE_CS),
            (0x8018c634, TAXI_CS),
            (0x8019f788, POLICE_CS),
            (0x8019ee0c, TAXI_CS),
            (0x8019ee10, POLICE_CS),
            (0x8019ed8c, 1),
            (POLICE_CS + 0x108, TAXI_CS),
            (WORLD + 0x104, root(1)),
            (WORLD + 0xc0, 8),
            (TAXI_CS + 0x104, root(vehicle as u16)),
            (POLICE_CS + 0x104, root(opponent as u16)),
        ] {
            put(&mut cpu, at, v);
        }
        for (at, v) in [
            (0x8018c334, 1u16),
            (0x8018c336, 2),
            (0x8018c338, 1),
            (0x801a22a8, 1),
            (0x80170238, 3),
            (0x8017023a, 20),
        ] {
            cpu.put(at, &v.to_le_bytes());
        }
        unsafe extern "C" {
            fn tm1_pickup_address(scene: *mut std::ffi::c_void, i: i32) -> u32;
        }
        for i in 0..50 {
            put(&mut cpu, 0x8019ce2c + 4 * i, unsafe {
                tm1_pickup_address(scene.native.as_ptr(), i as i32)
            });
        }
        for &(root, kind) in &roots {
            if matches!(kind, 750 | 752) {
                cpu.call(0x80148fb8, &[kind as u32, root]);
            }
        }
        cpu.put(0x80148380, &[8, 0, 224, 3, 0, 0, 0, 0]); // hover audio
        cpu.call(0x80102b48, &[HD_STACK]);
        cpu.call(0x80103a4c, &[HDP_STACK]);
        // Isolated physics runs skip display/audio request endpoints on both sides.
        for at in [
            0x801369b0, 0x8013721c, 0x8010461c, 0x8010767c, 0x80107524, 0x80106d60,
        ] {
            cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        if !retail_particles {
            // An exhausted explosion pool returns NULL to its callers.
            cpu.put(0x80135f90, &[8, 0, 224, 3, 0x21, 0x10, 0, 0]);
        }
        Self {
            pedestrian_nodes,
            cpu,
            scene,
            player,
            police,
        }
    }
    fn start_pedestrians(&mut self, level: Level) {
        put(&mut self.cpu, 0x8018c3b4, level as u32);
        assert_eq!(
            self.scene.pedestrian_models(),
            self.pedestrian_nodes
                .iter()
                .map(|&(_, ptr)| ptr)
                .collect::<Vec<_>>()
        );
        self.cpu.random_state = self.player.seed;
        for &(kind, addr) in &self.pedestrian_nodes {
            self.cpu.call(0x8014945c, &[kind, addr]);
        }
        self.scene.start_pedestrians(level, &mut self.player.seed);
        assert_eq!(self.player.seed, self.cpu.random_state);
    }
    fn sync(&mut self) {
        let cpu = &mut self.cpu;
        cpu.put(PLAYER, &self.player.state);
        cpu.put(POLICE, &self.police.state);
        cpu.random_state = self.player.seed;
        put(cpu, 0x8017023c, self.player.frame);
        for (cs, state, m, st, id) in [
            (TAXI_CS, &self.player.state[..], 0x380, 0x138, 1),
            (POLICE_CS, &self.police.state[..], 0x43c, 0x20c, 50),
        ] {
            Self::sync_model(cpu, cs, state, m, st, id);
        }
    }
    fn sync_model(cpu: &mut Cpu, cs: u32, state: &[u8], m: usize, st: usize, id: u32) {
        let word = |at: usize| i32::from_le_bytes(state[at..at + 4].try_into().unwrap());
        cpu.put(cs + 0x14, &state[m + 0x50..m + 0x70]);
        cpu.put(cs + 0x5c, &state[m + 8..m + 20]);
        for (offset, v) in [(8, 1), (12, 1), (0xc0, id), (0x74, 5)] {
            put(cpu, cs + offset, v);
        }
        for i in 0..5 {
            let x = if i == 4 {
                0
            } else if i == 1 || i == 2 {
                word(st + 0x74) / 2
            } else {
                -word(st + 0x74) / 2
            };
            let y = if i == 2 || i == 3 {
                word(st + 0x84)
            } else {
                word(st + 0x80)
            };
            for (j, v) in [x, y, 24 - word(st + 0x6c)].into_iter().enumerate() {
                cpu.put(cs + 0x78 + 8 * i + 2 * j as u32, &(v as i16).to_le_bytes());
            }
        }
    }
    fn compare(&self) {
        for (addr, state) in [
            (PLAYER, &self.player.state[..]),
            (POLICE, &self.police.state[..]),
        ] {
            let off = (addr & 0x1fffff) as usize;
            for (i, &v) in state.iter().enumerate() {
                assert_eq!(v, self.cpu.ram[off + i], "state {addr:x}+{i:x}");
            }
        }
        assert_eq!(self.player.seed, self.cpu.random_state);
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn engine_requests_match_disc() {
    let mut r = Reference::new();
    let table = r.cpu.call(0x801073bc, &[]);
    for car in Vehicle::ROSTER.into_iter().chain([Vehicle::Minion]) {
        for player in [false, true] {
            let (address, state, flag, velocity, gear_limit, name) = if player {
                (PLAYER, &mut r.player.state[..], 0xa0, 0x398, 0x1e0, 0x94)
            } else {
                (POLICE, &mut r.police.state[..], 0x66, 0x454, 0x2b4, 0x3c)
            };
            state[name..name + 4].copy_from_slice(&(car as i32).to_le_bytes());
            state[gear_limit..gear_limit + 4].copy_from_slice(&120i32.to_le_bytes());
            r.cpu.put(address, state);
            for speed in [-200i32, -75, 0, 61, 120, 300] {
                for (gear, turbo) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                    let state = if player {
                        &mut r.player.state[..]
                    } else {
                        &mut r.police.state[..]
                    };
                    state[velocity..velocity + 4].copy_from_slice(&speed.to_le_bytes());
                    state[flag + 4] = gear;
                    state[flag + 13] = turbo;
                    r.cpu.put(address, state);
                    let mut rev = None;
                    r.cpu.call_observed(
                        0x80150790,
                        &[0, address, player as u32],
                        0x80000080,
                        |cpu| {
                            if cpu.pc() == 0x80106d60 && cpu.r[7] == 1 {
                                let record = (table + 6 * cpu.r[4]) as usize & 0x1fffff;
                                assert_eq!(
                                    cpu.ram[record + 2] as usize,
                                    car.sound_programs().0,
                                    "{car:?}"
                                );
                                rev = Some((cpu.word(cpu.r[29] + 16) as i32).clamp(0, 99));
                            }
                        },
                    );
                    r.police.state[0x40] = 1;
                    let port = if player {
                        Some(r.player.rev())
                    } else {
                        r.police.engine_rev()
                    };
                    assert_eq!(port, rev, "{car:?} speed {speed} gear {gear} turbo {turbo}");
                }
            }
        }
    }
    r.police.state[0x40] = 0;
    assert_eq!(r.police.engine_rev(), None);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn actor_explosion_and_signature_are_consumed_once() {
    let mut r = Reference::new();
    for (state, base) in [
        (&mut r.player.state[..], 0xa0),
        (&mut r.police.state[..], 0x66),
    ] {
        state[base + 26] = 1;
        state[base + 24] = 1;
    }
    r.police.state[0x40] = 1;
    for tick in 0..2 {
        r.scene
            .step(&mut r.player, &mut r.police, Input::default(), 3);
        for slot in 0..2 {
            let events = r
                .scene
                .car_sound_requests(slot)
                .map(|(s, _)| s)
                .filter(|s| matches!(s, CarSound::Explosion | CarSound::Signature))
                .collect::<Vec<_>>();
            assert_eq!(
                events,
                if tick == 0 {
                    vec![CarSound::Explosion, CarSound::Signature]
                } else {
                    vec![]
                }
            );
        }
        for (state, base) in [(&r.player.state[..], 0xa0), (&r.police.state[..], 0x66)] {
            assert_eq!((state[base + 26], state[base + 24]), (0, 0));
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn skid_and_crash_requests_match_disc() {
    let mut r = Reference::new();
    let table = r.cpu.call(0x801073bc, &[]);
    for car in Vehicle::ROSTER.into_iter().chain([Vehicle::Minion]) {
        for player in [false, true] {
            let (address, flag, collision, name) = if player {
                (PLAYER, 0xa2, 0x26a, 0x94)
            } else {
                (POLICE, 0x68, 0x15a, 0x3c)
            };
            for impact in [-1i16, 0, 1, 14, 33, 34, 39, 50, 51, 66, 67, 99, 120] {
                for skid in [false, true] {
                    let state = if player {
                        &mut r.player.state[..]
                    } else {
                        &mut r.police.state[..]
                    };
                    state[name..name + 4].copy_from_slice(&(car as i32).to_le_bytes());
                    state[flag] = u8::from(skid);
                    state[collision..collision + 2].copy_from_slice(&impact.to_le_bytes());
                    if !player {
                        state[0x40] = 1;
                    }
                    r.cpu.put(address, state);
                    let mut original = Vec::new();
                    r.cpu.call_observed(
                        0x80150790,
                        &[0, address, player as u32],
                        0x80000080,
                        |cpu| {
                            if cpu.pc() == 0x80106d60 && (106..=111).contains(&cpu.r[4]) {
                                assert_eq!(cpu.r[7], 8);
                                let rec = (table + 6 * cpu.r[4]) as usize & 0x1fffff;
                                original.push((
                                    cpu.ram[rec + 2] as usize,
                                    cpu.word(cpu.r[29] + 16) as i32,
                                ));
                            }
                        },
                    );
                    let port = if player {
                        r.player.contact_sounds()
                    } else {
                        r.police.contact_sounds()
                    };
                    let sounds: Vec<_> = port
                        .into_iter()
                        .flatten()
                        .map(|(kind, v)| (ContactVoice::PROGRAMS[kind as usize], v))
                        .collect();
                    assert_eq!(
                        sounds, original,
                        "{car:?} player {player} skid {skid} impact {impact}"
                    );
                }
            }
        }
    }
    r.police.state[0x40] = 0;
    assert_eq!(r.police.contact_sounds(), [None; 2]);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn hud_opponent_matches_disc() {
    let mut r = Reference::new();
    let mut opponents = Opponent::spawn_bout(&mut r.player, &[Vehicle::Outlaw, Vehicle::Darkside]);
    vec3(&mut r.player.state, 0x388, [0; 3]);
    r.cpu.put(PLAYER, &r.player.state);
    r.cpu.put(0x8018c338, &2u16.to_le_bytes());
    for distance in [0, 12, 13, 0x7ffe, 0x7fff, 0x8000] {
        for flags in 0..16 {
            for (i, ai) in opponents.iter_mut().enumerate() {
                let pos = if i == 0 {
                    [distance, 0, 500]
                } else {
                    [5, -7, 100]
                };
                vec3(&mut ai.state, 0x444, pos);
                ai.state[0x41] = u8::from(flags & (1 << i) != 0);
                ai.state[0x24c..0x250]
                    .copy_from_slice(&i32::from(flags & (4 << i) != 0).to_le_bytes());
                r.cpu.put(POLICE + i as u32 * 0x560, &ai.state);
            }
            let address = r.cpu.call(0x8011c174, &[0, RENDER_SCRATCH]);
            let target = r.player.hud_opponent(&opponents);
            let index = opponents
                .iter()
                .position(|ai| target.is_some_and(|t| std::ptr::eq(t, ai)));
            assert_eq!(
                address,
                index.map_or(0, |i| POLICE + i as u32 * 0x560),
                "distance {distance} flags {flags}"
            );
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn carrying_capacity_matches_disc() {
    unsafe extern "C" {
        fn carGetPickup(car: *mut u8, player: u8, tag: i32, slot: i32);
        fn tm1_tick_carry_notice();
    }
    let mut r = Reference::new();
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let fonts = tm1_assets::screen::Fonts::load(&disc).unwrap();
    for (card, car) in Vehicle::CARD_ORDER
        .into_iter()
        .chain([Vehicle::Minion])
        .enumerate()
    {
        let address = r.cpu.call(0x80145d70, &[car as u32]);
        let name =
            std::ffi::CStr::from_bytes_until_nul(&r.cpu.ram[(address & 0x1fffff) as usize..])
                .unwrap();
        assert_eq!(fonts.car_sign(card).as_bytes(), name.to_bytes());
    }
    for address in [0x801419c8, 0x8010b4f4] {
        // scene switch and sprite submission
        r.cpu.put(address, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    for (player, tag, before, expected) in [
        (1, 401, 0, 0),
        (1, 401, 9, 30),
        (1, 401, 10, 30),
        (1, 430, 10, 0),
        (0, 401, 9, 0),
    ] {
        let mut state = if player == 1 {
            r.player.state.to_vec()
        } else {
            r.police.state.to_vec()
        };
        let weapon = if player == 1 { 0x458 } else { 0x514 };
        state[weapon + 12..weapon + 40].fill(0);
        for (offset, v) in [(12, before), (40, 10u16), (52, before)] {
            state[weapon + offset..weapon + offset + 2].copy_from_slice(&v.to_le_bytes());
        }
        r.cpu.put(PLAYER, &state);
        r.cpu.call(0x80142820, &[]);
        r.cpu.call(0x80135840, &[PLAYER, player as u32, tag, 1]);
        unsafe {
            tm1_use_scene(r.scene.native.as_ptr());
            carGetPickup(state.as_mut_ptr(), player, tag as i32, 1);
            tm1_use_scene(std::ptr::null_mut());
        }
        assert_eq!(expected, r.scene.carry_notice_frames());
        for _ in 0..32 {
            assert_eq!(r.scene.carry_notice_frames() as u32, r.cpu.word(0x8018c5ec));
            r.cpu.call_until(0x80142b2c, &[], 0x80142b9c);
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                tm1_tick_carry_notice();
                tm1_use_scene(std::ptr::null_mut());
            }
        }
        let start = (PLAYER & 0x1fffff) as usize + weapon;
        assert_eq!(
            &state[weapon..weapon + 0x48],
            &r.cpu.ram[start..start + 0x48]
        );
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn pickup_sounds_match_disc() {
    unsafe extern "C" {
        fn carGetPickup(car: *mut u8, player: u8, tag: i32, slot: i32);
        fn tm1_bind_step_audio(audio: *mut std::ffi::c_void);
    }
    let mut r = Reference::new();
    r.cpu.put(0x801419c8, &[8, 0, 224, 3, 0, 0, 0, 0]); // pickup mesh switch
    let table = r.cpu.call(0x801073bc, &[]);
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    let mut voice_ids = std::collections::BTreeMap::new();
    for player in [0, 1] {
        for tag in [
            401, 402, 403, 404, 405, 406, 410, 411, 412, 413, 414, 430, 431, 450,
        ] {
            for total in [0u16, 10] {
                let (mut car, weapon, position) = if player == 1 {
                    (r.player.state.to_vec(), 0x458, 0x388)
                } else {
                    (r.police.state.to_vec(), 0x514, 0x444)
                };
                vec3(&mut car, position, [131, -279, 405]);
                car[weapon + 0x28..weapon + 0x2a].copy_from_slice(&10u16.to_le_bytes());
                car[weapon + 0x34..weapon + 0x36].copy_from_slice(&total.to_le_bytes());
                r.cpu.put(PLAYER, &car);
                let mut origin = [0; 3];
                let mut expected = Vec::new();
                r.cpu.call_observed(
                    0x80135840,
                    &[PLAYER, player as u32, tag, 1],
                    0x80000080,
                    |cpu| {
                        if cpu.pc() == 0x8010461c {
                            origin =
                                std::array::from_fn(|i| cpu.word(cpu.r[4] + 4 * i as u32) as i32);
                        }
                        if cpu.pc() == 0x80106d60 {
                            assert_eq!(cpu.r[7], 7);
                            if let Some(previous_tag) = voice_ids.insert(cpu.r[4], tag) {
                                assert_eq!(previous_tag, tag);
                            }
                            let rec = (table + 6 * cpu.r[4]) as usize & 0x1fffff;
                            assert_eq!(&cpu.ram[rec..rec + 4], &[0, 0, 36, 0]);
                            expected.push([origin[0], origin[1], origin[2], tag as i32]);
                        }
                    },
                );
                r.scene.audio.clear();
                unsafe {
                    tm1_use_scene(r.scene.native.as_ptr());
                    tm1_bind_step_audio(std::ptr::from_mut(&mut r.scene.audio).cast());
                    carGetPickup(car.as_mut_ptr(), player, tag as i32, 1);
                    tm1_bind_step_audio(std::ptr::null_mut());
                    tm1_use_scene(std::ptr::null_mut());
                }
                let sounds: Vec<_> = r
                    .scene
                    .audio
                    .iter()
                    .filter_map(|event| match *event {
                        BattleEvent::Effect {
                            kind,
                            position: [x, y, z],
                        } => Some([x, y, z, kind]),
                        _ => None,
                    })
                    .collect();
                assert_eq!(sounds, expected, "player {player} tag {tag} total {total}");
            }
        }
    }
    assert_eq!(voice_ids.len(), 12);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn warehouse_lineups_and_ai_profiles_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    let mut player = PlayerVehicle::from_executable(&exe).unwrap();
    put(&mut cpu, 0x8018c36c, 1);
    let mut deck = ArenaDeck::default();
    let mut seed = 1;
    cpu.random_state = seed;
    for vehicle in [Vehicle::YellowJacket; 24]
        .into_iter()
        .chain(Vehicle::ROSTER)
    {
        cpu.call(0x80150a40, &[vehicle as u32, 0, 0x80030000, 0x80030004]);
        assert_eq!(cpu.word(0x80030000), 1);
        assert_eq!(cpu.word(0x80030004), deck.draw(vehicle, &mut seed) as u32);
        assert_eq!(cpu.random_state, seed);
    }
    for vehicle in Vehicle::ROSTER {
        let count = 0x80030000;
        let cards = count + 4;
        cpu.call(0x80150a40, &[vehicle as u32, 1, count, cards]);
        let lineup = vehicle.warehouse_opponents();
        assert_eq!(cpu.word(count) as usize, lineup.len());
        for (i, opponent) in lineup.iter().enumerate() {
            assert_eq!(cpu.word(cards + 4 * i as u32), *opponent as u32);
        }
        let seed = player.seed;
        let ai = PoliceCar::for_vehicle(&mut player, vehicle);
        cpu.put(POLICE, &[0; 0x560]);
        cpu.put(0x80170238, &3u16.to_le_bytes());
        cpu.put(0x8017023a, &20u16.to_le_bytes());
        put(&mut cpu, 0x8019f788, POLICE_CS);
        cpu.call(0x80125bc0, &[]);
        cpu.random_state = seed;
        cpu.call(0x80124d34, &[POLICE, vehicle as u32, 1]);
        cpu.call(0x80142064, &[POLICE]);
        let base = (POLICE & 0x1fffff) as usize;
        for (i, &v) in ai.state.iter().enumerate() {
            assert_eq!(v, cpu.ram[base + i], "{vehicle:?} AI spawn at {i:#x}");
        }
        assert_eq!(cpu.random_state, player.seed);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn group_spawn_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    put(&mut cpu, 0x8018c36c, 1);

    cpu.put(0x80170238, &3u16.to_le_bytes());
    cpu.put(0x8017023a, &20u16.to_le_bytes());
    for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        cpu.call(0x80150224, &[difficulty as u32]);
        for (level, init_points) in [
            (Level::Warehouse, 0x801262ec),
            (Level::Freeway, 0x801268f8),
            (Level::CityPark, 0x80127694),
            (Level::Cyburbia, 0x801281b8),
            (Level::Rooftops, 0x8012b5a4),
        ] {
            put(&mut cpu, 0x8018c3b4, level as u32);
            for vehicle in Vehicle::ROSTER {
                let mut player =
                    PlayerVehicle::contest(&exe, vehicle, level, 1, difficulty).unwrap();
                let cards = level.opponents(vehicle, &mut ArenaDeck::default(), &mut 1);
                cpu.call(init_points, &[]);
                cpu.random_state = player.seed;
                let opponents = Opponent::spawn_bout(&mut player, &cards);
                for (slot, opponent) in opponents.iter().enumerate() {
                    let addr = POLICE + 0x560 * slot as u32;
                    put(
                        &mut cpu,
                        0x8019f788 + 4 * slot as u32,
                        POLICE_CS + 0x200 * slot as u32,
                    );
                    cpu.put(addr, &[0; 0x560]);
                    cpu.put(addr + 6, &(slot as u16).to_le_bytes());
                    cpu.call(0x80124d34, &[addr, opponent.vehicle as u32, 1]);
                    cpu.call(0x80142064, &[addr]);
                    let base = (addr & 0x1fffff) as usize;
                    for (i, &v) in opponent.state.iter().enumerate() {
                        assert_eq!(v, cpu.ram[base + i], "{vehicle:?} slot {slot} byte {i:#x}");
                    }
                }
                assert_eq!(cpu.random_state, player.seed);
            }
        }
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn group_battle_selection_original() {
    unsafe extern "C" {
        fn tm1_schedule_battle(
            waited: &mut i32,
            cars: *mut u8,
            count: i16,
            engaged: i16,
            fields: i16,
        );
    }
    // The C routine accesses these PS1-layout records as an array of AICar.
    #[repr(align(4))]
    struct Records([[u8; 0x560]; 3]);
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    let mut player =
        PlayerVehicle::on_level(&exe, Vehicle::YellowJacket, Level::Warehouse).unwrap();
    let lineup = Opponent::spawn_lineup(
        &mut player,
        [Vehicle::Darkside, Vehicle::Thumper, Vehicle::CrimsonFury],
    );
    for count in 1..=3 {
        for engaged in 0..=count.min(2) {
            for live in 1..(1 << count) {
                for wait in [0, 177, 178, 180] {
                    let mut native = Records(std::array::from_fn(|i| lineup[i].state));
                    for (i, car) in native.0.iter_mut().enumerate().take(count) {
                        car[0x24c..0x250].copy_from_slice(
                            &(if live & (1 << i) != 0 { 50i32 } else { 0 }).to_le_bytes(),
                        );
                        car[0x4a] = u8::from(i == 1);
                        car[1] = u8::from(i != 1);
                        car[2] = (i + 1) as u8;
                        cpu.put(POLICE + 0x560 * i as u32, car);
                    }
                    let mut waited = wait;
                    put(&mut cpu, 0x8018c370, wait as u32);
                    cpu.put(0x8018c338, &(count as i16).to_le_bytes());
                    cpu.put(0x8018bfd0, &(engaged as i16).to_le_bytes());
                    cpu.put(0x80170238, &3u16.to_le_bytes());
                    cpu.call_until(0x8014dcf0, &[], 0x8014df5c);
                    unsafe {
                        tm1_schedule_battle(
                            &mut waited,
                            native.0[0].as_mut_ptr(),
                            count as i16,
                            engaged as i16,
                            3,
                        );
                    }
                    assert_eq!(cpu.word(0x8018c370), waited as u32);
                    for (i, car) in native.0.iter().enumerate().take(count) {
                        let base = (POLICE as usize & 0x1fffff) + 0x560 * i;
                        assert_eq!(&cpu.ram[base..base + 0x560], car);
                    }
                }
            }
        }
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn warehouse_group_hit_ids_original() {
    compare_car_hits(false);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_hit_ids_original() {
    compare_car_hits(true);
}

fn compare_car_hits(duel: bool) {
    use std::ffi::c_void;
    unsafe extern "C" {
        fn CAR_HD(point: *const i32, owner: i32, radius: i32) -> i32;
        fn GetPlayerCs3D(i: i16) -> *mut c_void;
        fn GetAICs3D(i: i16) -> *mut c_void;
        fn tm1_contact_setup(cs: *mut c_void, motion: *const i32, stats: *const i32);
        fn HdPntTest(skip0: i32, skip1: i32, pt: *const i32, hit: &mut i32) -> *mut c_void;
        fn uaIsCarMatID(id: i16, player: &mut i8, slot: &mut u16) -> i32;
    }
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc
        .read(if duel {
            "UA2PLAY/2PWH.DMD"
        } else {
            "UADMD/WH1.DMD"
        })
        .unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    cpu.put(0x800188b8, &data);
    let root = |kind| roots.iter().find(|r| r.1 == kind).unwrap().0;
    let (kinds, ids): (&[u16], &[u32]) = if duel {
        (&[20, 30], &[1, 2])
    } else {
        (&[20, 30, 100, 60], &[1, 50, 51, 52])
    };
    let scene = if duel {
        CollisionScene::for_duel(&data, root(1), [root(20), root(30)])
    } else {
        CollisionScene::for_opponents(&data, root(1), root(20), &[root(30), root(100), root(60)])
    }
    .unwrap();
    put(&mut cpu, 0x8018c4b0, WORLD);
    put(&mut cpu, WORLD + 0x104, root(1));
    put(&mut cpu, WORLD + 0xc0, 8);
    cpu.call(0x80103a4c, &[HDP_STACK]);
    let model = |i: usize| EXTRA_MODELS + i as u32 * 0x200;
    if duel {
        cpu.put(0x8018c334, &2u16.to_le_bytes());
        cpu.put(0x8018c338, &0u16.to_le_bytes());
        for slot in 0..2 {
            put(&mut cpu, 0x8018c634 + slot * 4, model(slot as usize));
        }
    }
    let mut head = 0;
    for &(node, kind) in &roots {
        if let Some(i) = kinds.iter().position(|v| *v == kind) {
            let cs = model(i);
            for (off, value) in [
                (0x104, node),
                (0x108, head),
                (0xc0, ids[i]),
                (8, 1),
                (12, 1),
            ] {
                put(&mut cpu, cs + off, value);
            }
            head = cs;
        }
    }
    put(&mut cpu, 0x8018bf14, head);
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        tm1_use_scene(scene.native.as_ptr());
    }
    struct Unbind;
    impl Drop for Unbind {
        fn drop(&mut self) {
            unsafe {
                tm1_use_scene(std::ptr::null_mut());
            }
        }
    }
    let _unbind = Unbind;
    for overlap in [false, true] {
        for (i, &id) in ids.iter().enumerate() {
            let x = if overlap { 0 } else { i as i32 * 2500 };
            let mut motion = [0i32; 80];
            motion[2] = x;
            motion[4] = 12000;
            put(&mut cpu, model(i) + 0x5c, x as u32);
            put(&mut cpu, model(i) + 0x64, 12000);
            let mut player = -1;
            let mut slot = 99;
            unsafe {
                let cs = if duel || i == 0 {
                    GetPlayerCs3D(i as i16)
                } else {
                    GetAICs3D((i - 1) as i16)
                };
                tm1_contact_setup(cs, motion.as_ptr(), [0i32; 72].as_ptr());
                assert_eq!(uaIsCarMatID(id as i16, &mut player, &mut slot), 1);
            }
            assert_eq!(
                (player, slot),
                (
                    i8::from(duel || i == 0),
                    if duel {
                        i as u16
                    } else {
                        i.saturating_sub(1) as u16
                    }
                )
            );
        }
        for x in [0, 2500, 5000, 7500, 10000] {
            let p = [x, 0, 12000];
            for (j, v) in p.into_iter().enumerate() {
                put(&mut cpu, 0x80011200 + 4 * j as u32, v as u32);
            }
            if duel {
                for owner in [1, 2] {
                    let original = cpu.call(0x8010c1d0, &[0x80011200, owner, 0]);
                    assert_eq!(
                        unsafe { CAR_HD(p.as_ptr(), owner as i32, 0) },
                        original as i32
                    );
                }
            }
            for skip0 in std::iter::once(0).chain(ids.iter().copied()) {
                for skip1 in std::iter::once(0).chain(ids.iter().copied()) {
                    cpu.call(0x801038b0, &[skip0, skip1, 0x80011200, 0x80011240]);
                    let mut id = -1;
                    unsafe {
                        HdPntTest(skip0 as i32, skip1 as i32, p.as_ptr(), &mut id);
                    }
                    assert_eq!(id, cpu.word(0x80011240) as i32);
                    if !overlap && x < 2500 * ids.len() as i32 {
                        let car = ids[(x / 2500) as usize];
                        assert_eq!(
                            id,
                            if [skip0, skip1].contains(&car) {
                                0
                            } else {
                                car as i32
                            }
                        );
                    }
                }
            }
        }
    }
}
fn vec3(state: &mut [u8], at: usize, xyz: [i32; 3]) {
    for (i, v) in xyz.into_iter().enumerate() {
        state[at + 4 * i..at + 4 * i + 4].copy_from_slice(&v.to_le_bytes());
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn bullet_impact_original() {
    let mut r = Reference::new();
    vec3(&mut r.player.state, 0x388, [2700, 2000, 400]);
    vec3(&mut r.police.state, 0x444, [2700, 3050, 400]);
    r.sync();
    let pos = [2700i32, 2250, 400];
    let dir = [0i32, 4096, 0];
    for (at, v) in [(0x800be000, dir), (0x800be020, pos)] {
        for (i, x) in v.iter().enumerate() {
            put(&mut r.cpu, at + i as u32 * 4, *x as u32);
        }
    }
    r.cpu.call(0x8010c930, &[1, 0x800be000, 0x800be020, 1]);
    {
        let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe extern "C" {
            fn tm1_use_scene(scene: *mut std::ffi::c_void);
            fn create_bullet(owner: i32, dir: *const i32, pos: *const i32, damage: i32);
        }
        unsafe {
            tm1_use_scene(r.scene.native.as_ptr());
            create_bullet(1, dir.as_ptr(), pos.as_ptr(), 1);
            tm1_use_scene(std::ptr::null_mut());
        }
    }
    let initial = r.police.health().0;
    for _ in 0..5 {
        r.cpu.call(0x8010d03c, &[]);
        r.cpu.call(0x80137d2c, &[]);
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        r.compare();
        let mut segments = [[[0; 3]; 2]; 40];
        let active = r.scene.bullet_segments(&mut segments);
        assert_eq!(active, usize::from(r.cpu.word(0x80198c28 + 0x28) > 0));
        if active > 0 {
            for (j, at) in [12, 0].into_iter().enumerate() {
                for i in 0..3 {
                    assert_eq!(
                        segments[0][j][i],
                        r.cpu.word(0x80198c28 + at + 4 * i as u32) as i32
                    );
                }
            }
        }
    }
    assert_eq!(r.police.health().0, initial - 1);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn special_bullet_effects_original() {
    let car = [2700, 2250, 400];
    let forward = [0, 4096, 0];
    for (kind, pos, dir, life, expected) in [
        (3, car, forward, 12, &[1, 54, 55][..]),
        (3, [2700, 2250, 20], [0, 4096, -4096], 12, &[1, 54]),
        (7, car, forward, 12, &[1, 54, 37, 8]),
        (7, [2700, 2250, 1500], forward, 1, &[37, 8]),
        (19, car, forward, 12, &[1, 28, 35, 54]),
    ] {
        let mut r = Reference::new();
        vec3(&mut r.player.state, 0x388, [2700, 2000, 400]);
        vec3(&mut r.police.state, 0x444, [2700, 3050, 400]);
        r.sync();
        for (at, values) in [(0x800be000, dir), (0x800be020, pos)] {
            for (i, x) in values.into_iter().enumerate() {
                put(&mut r.cpu, at + 4 * i as u32, x as u32);
            }
        }
        r.cpu
            .call(0x8010cd08, &[1, 0x800be000, 0x800be020, kind, 1, life, 0]);
        {
            let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            unsafe extern "C" {
                fn s_create_bullet(
                    owner: i32,
                    dir: *const i32,
                    pos: *const i32,
                    kind: i32,
                    damage: i32,
                    life: i32,
                    arg6: i32,
                );
            }
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                s_create_bullet(
                    1,
                    dir.as_ptr(),
                    pos.as_ptr(),
                    kind as i32,
                    1,
                    life as i32,
                    0,
                );
                tm1_use_scene(std::ptr::null_mut());
            }
        }
        let mut seen = Vec::new();
        for tick in 0..life {
            let mut original = Vec::new();
            let mut sound_pos = [0; 3];
            r.cpu.call_observed(0x8010d03c, &[], 0x80000080, |cpu| {
                let effect = match cpu.pc() {
                    0x8010461c => {
                        sound_pos = [0, 4, 8].map(|d| cpu.word(cpu.r[4] + d) as i32);
                        return;
                    }
                    0x8010767c => {
                        assert_eq!(cpu.r[6], 25);
                        original.push((54, sound_pos));
                        return;
                    }
                    0x801369b0 => 1,
                    0x80136a28 => 28,
                    0x80136aa0 => 55,
                    0x80136c78 => 37,
                    0x80136e18 => 35,
                    0x80137140 => 8,
                    _ => return,
                };
                original.push((effect, [0, 4, 8].map(|d| cpu.word(cpu.r[4] + d) as i32)));
            });
            r.scene.phase(&mut r.player, &mut r.police, 2, 3);
            let native: Vec<_> = r
                .scene
                .audio
                .iter()
                .filter_map(|event| match *event {
                    BattleEvent::Effect { kind, position }
                        if matches!(kind, 1 | 54 | 28 | 37 | 55 | 35 | 8) =>
                    {
                        Some((kind, position))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(native, original, "kind {kind}, tick {tick}");
            let mut effects = [[0; 5]; 122];
            let count = r.scene.effects(&mut effects);
            for &(effect, [x, y, z]) in &original {
                let (ttl, rise) = match effect {
                    1 => (3, 0),
                    28 | 37 | 55 => (31, 0),
                    35 => (103, 0),
                    8 => (87, 10),
                    54 => continue,
                    _ => unreachable!(),
                };
                assert!(effects[..count].contains(&[x, y, z + rise, effect, ttl]));
            }
            seen.extend(original.iter().map(|e| e.0));
            if !seen.is_empty() {
                break;
            }
        }
        assert_eq!(seen, expected, "kind {kind}, pos {pos:?}");
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn password_weapon_and_damage_modes_original() {
    for god_mode in [false, true] {
        let mut r = Reference::new();
        vec3(&mut r.player.state, 0x388, [2700, 3050, 400]);
        vec3(&mut r.police.state, 0x444, [2700, 2000, 400]);
        r.sync();
        r.cpu.call(0x8014d418, &[0, 1]);
        if god_mode {
            r.cpu.call(0x8014d354, &[0, 1]);
        }
        r.player.set_weapon_and_damage_modes(true, god_mode);
        r.compare();
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        r.cpu.call(0x8013148c, &[PLAYER, 1]);
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        let hp = r.player.health().0;
        fire_at_player(&mut r);
        assert_eq!(hp - r.player.health().0, i32::from(!god_mode));
        r.player.reset();
        assert_eq!(r.player.state[0x4a1], 1);
        assert_eq!(r.player.state[0x184], u8::from(!god_mode));
    }
}

fn fire_at_player(r: &mut Reference) -> bool {
    let pos = [2700i32, 2250, 400];
    let dir = [0i32, 4096, 0];
    for (at, v) in [(0x800be000, dir), (0x800be020, pos)] {
        for (i, x) in v.into_iter().enumerate() {
            put(&mut r.cpu, at + 4 * i as u32, x as u32);
        }
    }
    r.cpu.call(0x8010c930, &[50, 0x800be000, 0x800be020, 1]);
    {
        let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe extern "C" {
            fn create_bullet(owner: i32, dir: *const i32, pos: *const i32, damage: i32);
        }
        unsafe {
            tm1_use_scene(r.scene.native.as_ptr());
            create_bullet(50, dir.as_ptr(), pos.as_ptr(), 1);
            tm1_use_scene(std::ptr::null_mut());
        }
    }
    let mut flashed = false;
    for _ in 0..5 {
        let sheet = retail_hit_sheet(&mut r.cpu);
        r.cpu.call(0x8010e3d8, &[]);
        r.cpu.call(0x80137d2c, &[]);
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        r.compare();
        if let Some(rgba) = sheet {
            flashed = true;
            assert_eq!(r.scene.hit_sheet(), rgba);
        } else {
            assert_eq!(r.scene.hit_sheet()[3], 0);
        }
    }
    flashed
}

fn retail_hit_sheet(cpu: &mut Cpu) -> Option<[u8; 4]> {
    if cpu.word(0x8018bfcc) == 0 {
        return None;
    }
    cpu.call(0x80110ea8, &[RENDER_SCRATCH]);
    let packet = 0x80000000 | (cpu.word(RENDER_SCRATCH) & 0x00ffffff);
    let [r, g, b, opcode] = cpu.word(packet + 4).to_le_bytes();
    Some([r, g, b, if opcode & 2 != 0 { 128 } else { 255 }])
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn cockpit_damage_sheet_original() {
    unsafe extern "C" {
        fn SetFXSheet(count: i32, r: i32, g: i32, b: i32);
        fn tm1_display_hit_sheet();
    }
    for mode in [CarView::Cockpit, CarView::Chase] {
        let mut r = Reference::new();
        r.player.set_battle_view(PlayerView::Car(mode), false);
        vec3(&mut r.player.state, 0x388, [2700, 3050, 400]);
        vec3(&mut r.police.state, 0x444, [2700, 2000, 400]);
        r.sync();
        assert_eq!(fire_at_player(&mut r), mode == CarView::Cockpit);
        for frame in 1..=14 {
            let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                if frame == 1 || frame == 2 {
                    let color = if frame == 1 { 0 } else { 255 };
                    r.cpu.call(0x80110d90, &[10, 255, color, color]);
                    SetFXSheet(10, 255, color as i32, color as i32);
                }
                tm1_display_hit_sheet();
                tm1_use_scene(std::ptr::null_mut());
            }
            if let Some(rgba) = retail_hit_sheet(&mut r.cpu) {
                assert_eq!(r.scene.hit_sheet(), rgba);
            } else {
                assert_eq!(r.scene.hit_sheet()[3], 0);
            }
            r.cpu.call(0x8010e3d8, &[]);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn exterior_damage_lights_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |kind| roots.iter().find(|r| r.1 == kind).unwrap().0;
    let mut scene = CollisionScene::for_opponents(
        &data,
        root(1),
        root(10),
        &(2..=9).map(|i| root(i * 10)).collect::<Vec<_>>(),
    )
    .unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    let envs = [0, 1, 2].map(|i| cpu.call(0x801043b4, &[i]));
    let nodes = std::array::from_fn::<_, 9, _>(|i| EXTRA_MODELS + 0x200 * i as u32);
    for cs in nodes {
        put(&mut cpu, cs + 4, envs[0]);
    }
    unsafe extern "C" {
        fn GetPlayerCs3D(index: i16) -> *mut std::ffi::c_void;
        fn GetAICs3D(index: i16) -> *mut std::ffi::c_void;
        fn UAeffectActHitLights(cs: *mut std::ffi::c_void, count: i32, rgb: *const u8);
        fn UAeffectSetFreezeLight(cs: *mut std::ffi::c_void);
        fn tm1_clear_freeze_light(player: u8, index: i16);
        fn tm1_update_hit_lights();
        fn tm1_capture_car_lights();
    }
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        tm1_use_scene(scene.native.as_ptr());
        for tick in 0..12 {
            for (slot, &cs) in nodes.iter().enumerate() {
                let index = slot.saturating_sub(1) as i16;
                let native = if slot == 0 {
                    GetPlayerCs3D(0)
                } else {
                    GetAICs3D(index)
                };
                let count = match (tick, slot) {
                    (0, _) => 4,
                    (2, 1) => 5,
                    (4, 6..=8) => 4,
                    _ => 0,
                };
                if count > 0 {
                    cpu.call(0x80146754, &[cs, count, 0]);
                    UAeffectActHitLights(native, count as i32, std::ptr::null());
                }
                if tick == 1 && slot >= 5 {
                    cpu.call(0x80146900, &[cs]);
                    UAeffectSetFreezeLight(native);
                }
                if tick == 8 && slot >= 5 {
                    cpu.call(0x80146930, &[cs]);
                    tm1_clear_freeze_light(u8::from(slot == 0), index);
                }
            }
            cpu.call(0x80146800, &[]);
            tm1_update_hit_lights();
            tm1_capture_car_lights();
            let expected = nodes.map(|cs| {
                envs.iter()
                    .position(|&env| env == cpu.word(cs + 4))
                    .unwrap() as u8
            });
            assert_eq!(scene.car_lights(), expected, "tick {tick}");
        }
        tm1_use_scene(std::ptr::null_mut());
    }
    scene.reset_projectiles();
    assert_eq!(scene.car_lights(), [0; 9]);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn wheel_motion_original() {
    let mut r = Reference::new();
    let tire = |slot: u32, stage: u32| RENDER_SCRATCH + slot * 128 + stage * 32;
    for slot in 1..=13 {
        for stage in 0..3 {
            let node = tire(slot, stage);
            put(&mut r.cpu, 0x8019cd78 + (slot * 3 + stage) * 4, node);
            r.cpu.put(node + 6, &[2 + stage as u8, stage as u8]);
        }
    }
    let wheel = RENDER_SCRATCH + 0x1000;
    for i in 0..4 {
        put(&mut r.cpu, 0x8019e5bc + 4 * i, wheel + 64 * i);
    }
    unsafe extern "C" {
        fn tm1_update_car_speed(car_id: i32, speed: i32);
    }
    for speed in [
        0, 160, 160, 1, 650, 320, -1, -321, -645, 6, -6, 7, -7, 250, 250,
    ] {
        for slot in 1..=13 {
            r.cpu.call(0x801466b0, &[slot * 10, speed as u32]);
            {
                let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
                unsafe {
                    tm1_use_scene(r.scene.native.as_ptr());
                    tm1_update_car_speed(slot as i32 * 10, speed);
                    tm1_use_scene(std::ptr::null_mut());
                }
            }
            let step = r.scene.wheel_steps()[slot as usize];
            if slot == 4 {
                assert_eq!(step, i64::from(r.cpu.word(0x8018c2f8) as i32));
                let [sin, cos] = r.player.wheel_sin_cos(step);
                for i in 0..4 {
                    for (j, expected) in [4096, 0, 0, 0, cos, sin, 0, -sin, cos]
                        .into_iter()
                        .enumerate()
                    {
                        let at = ((wheel + 64 * i + 4) as usize & 0x1fffff) + 2 * j;
                        assert_eq!(
                            i16::from_le_bytes(r.cpu.ram[at..at + 2].try_into().unwrap()),
                            expected,
                            "speed {speed}, matrix element {j}"
                        );
                    }
                }
            } else {
                for stage in 0..3 {
                    let actual = r.cpu.ram[((tire(slot, stage) + 7) & 0x1fffff) as usize];
                    assert_eq!(
                        i64::from(actual),
                        (step + i64::from(stage)).rem_euclid(2 + i64::from(stage))
                    );
                }
            }
        }
    }
    r.scene.reset_projectiles();
    assert_eq!(r.scene.wheel_steps(), [0; 14]);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn victory_blocks_late_bullet_damage_original() {
    for simultaneous_death in [false, true] {
        let mut r = Reference::new();
        r.police.state[0x24c..0x250].fill(0);
        if simultaneous_death {
            r.player.state[0x178..0x17c].fill(0);
        }
        r.scene
            .step(&mut r.player, &mut r.police, Input::default(), 3);
        r.sync();
        // Observe the retail victory branch twice to check that it queues one return.
        for _ in 0..2 {
            r.cpu.call_until(0x8014e0b4, &[], 0x8014e10c);
            assert_eq!(r.scene.level_won(), r.cpu.word(0x8018c360) != 0);
            assert_eq!(r.cpu.word(0x8018bf18), u32::from(!simultaneous_death));
        }
        assert_eq!(r.scene.level_won(), !simultaneous_death);
        if simultaneous_death {
            continue;
        }
        assert_eq!(r.cpu.word(0x80194bf0), 180);
        assert_eq!(r.cpu.word(0x80194bf4), 3);

        vec3(&mut r.player.state, 0x388, [2700, 3050, 400]);
        vec3(&mut r.police.state, 0x444, [2700, 2000, 400]);
        r.sync();
        let hp = r.player.health().0;
        fire_at_player(&mut r);
        assert_eq!(r.player.health().0, hp);
        r.scene.reset_projectiles();
        put(&mut r.cpu, 0x8018c360, 0);
        assert!(!r.scene.level_won());
        fire_at_player(&mut r);
        assert_eq!(r.player.health().0, hp - 1);
    }
}

fn setup_chase_camera(r: &mut Reference) -> VehicleCamera {
    // Supply the one-player FOV and observe the last body draw request.
    r.cpu.put(0x8010a690, &[8, 0, 224, 3, 60, 0, 2, 36]);
    let flag = RENDER_SCRATCH;
    for (entry, visible) in [(0x8015060c, 1), (0x801505c4, 0)] {
        let code = [
            0x3c080000 | ((flag + 0x8000) >> 16),
            0x34090000 | visible,
            0x03e00008,
            0xad090000 | (flag & 0xffff),
        ];
        // The store sign-extends its 16-bit offset.
        r.cpu.put(
            entry,
            &code
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
    }
    for entry in [0x8010ad84, 0x8010add4, 0x80144be8] {
        r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    r.cpu.put(PLAYER, &r.player.state);
    r.cpu.call(0x80124474, &[PLAYER]);
    let cam = VehicleCamera::new(&r.player, CarView::Chase);
    put(&mut r.cpu, PLAYER + 0xf4, cam.mode as u32);
    r.cpu.put(0x800ffd10, &[8, 0, 224, 3, 0, 0, 2, 36]); // No cheat input.
    vec3_cpu(&mut r.cpu, PLAYER + 0xc, cam.offset);
    put(&mut r.cpu, PLAYER + 4, cam.pitch as u32);
    cam
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn ordinary_camera_original() {
    for vehicle in Vehicle::ROSTER {
        let mut r = Reference::on_level(vehicle, false, Level::Arena);
        let mut cam = setup_chase_camera(&mut r);
        for tick in 0..180 {
            let forward = matches!(tick, 12 | 48 | 75);
            let backward = matches!(tick, 95 | 110 | 143);
            r.cpu.put(PLAYER + 0xd9, &[backward.into(), forward.into()]);
            r.cpu.call(0x8014eff4, &[PLAYER]); // CheckVRMode
            if forward || backward {
                cam.cycle(forward);
            }
            assert_eq!(cam.mode as u32, r.cpu.word(PLAYER + 0xf4));
            r.player.step(
                Input {
                    accelerate: true,
                    left: tick < 75,
                    right: tick > 90,
                    ..Default::default()
                },
                3,
            );
            r.cpu.put(PLAYER + 0x138, &r.player.state[0x138..]);
            let fields = 1 + (tick % 5) as u8;
            r.cpu.put(0x80170238, &u16::from(fields).to_le_bytes());
            assert_eq!(
                r.player.speed_mph(fields),
                r.cpu.call(0x8014f794, &[0]) as i32
            );
            r.cpu.call(0x80116bcc, &[PLAYER]);
            cam.update(&r.player, false);
            let offset = std::array::from_fn(|i| r.cpu.word(PLAYER + 0xc + 4 * i as u32) as i32);
            assert_eq!(cam.offset, offset, "{vehicle:?} tick {tick}");
            assert_eq!(cam.pitch, r.cpu.word(PLAYER + 4) as i16 as i32);
            assert_eq!(cam.show_body, r.cpu.word(RENDER_SCRATCH) != 0);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn special_view_wrap_and_timeout_original() {
    let mut r = Reference::new();
    for layout in 0..4 {
        r.player.set_pad_layout(layout);
        r.cpu.put(PLAYER, &r.player.state);
        for bit in 0..16 {
            for companion in 0..16 {
                let pads = (1 << bit) | (1 << companion);
                r.cpu.call(0x800ffb98, &[2, pads]);
                r.cpu.call(0x801125ac, &[PLAYER]);
                assert_eq!(
                    r.player.view_buttons(pads),
                    [18, 17, 20, 23, 19]
                        .map(|i| r.cpu.ram[(PLAYER as usize & 0x1fffff) + 0xc8 + i] != 0)
                );
            }
        }
        let fire = u32::from(r.player.keyboard_buttons(Input {
            missile: true,
            ..Default::default()
        }));
        for (forward, weapon, buttons, armed) in [
            (true, 11, fire, 0),
            (true, 11, 0, 0),
            (true, 0, fire, 0),
            (false, 11, fire, 0),
            (false, 11, fire, 1),
        ] {
            r.player.cheat_controls = false;
            r.player.state[0x460..0x464].copy_from_slice(&i32::to_le_bytes(weapon));
            r.player.state[0x9a] = u8::from(!forward);
            r.player.state[0x9b] = armed;
            r.player.state[0x144..0x146].copy_from_slice(&19i16.to_le_bytes());
            r.cpu.put(PLAYER, &r.player.state);
            put(&mut r.cpu, PLAYER + 0xf4, if forward { 3 } else { 1 });
            r.cpu.put(0x8018c33a, &[0]); // gStandMode
            r.cpu.put(0x8018c3ac, &1u16.to_le_bytes());
            r.cpu.call(0x800ffb98, &[2, buttons]);
            r.cpu.call(0x801125ac, &[PLAYER]); // CarUpdateControlPad
            r.cpu
                .put(PLAYER + 0xd9, &[u8::from(!forward), u8::from(forward)]);
            r.cpu.call_until(0x8014eff4, &[PLAYER], 0x8014f278);
            r.player.wrap_special_view(forward, buttons);
            assert_eq!(
                r.player.word(0x98) as u32 >> 16,
                r.cpu.word(PLAYER + 0x98) >> 16
            );
            assert_eq!(
                r.player.word(0x144) as u16,
                r.cpu.word(PLAYER + 0x144) as u16
            );
            assert_eq!(r.player.cheat_controls, r.cpu.ram[0x18c33a] != 0);
        }
    }
    r.player.cheat_controls = false;
    r.player.state[0x9a..0x9c].copy_from_slice(&[1, 1]);
    r.player.state[0x144..0x146].fill(0);
    r.cpu.put(PLAYER, &r.player.state);
    // Enter CarUpdate's non-cheating branch with s1 pointing to the car.
    put(&mut r.cpu, RENDER_SCRATCH, 0x00808821); // move s1,a0
    put(
        &mut r.cpu,
        RENDER_SCRATCH + 4,
        0x08000000 | ((0x801119ecu32 >> 2) & 0x03ffffff),
    );
    put(&mut r.cpu, RENDER_SCRATCH + 8, 0);
    for tick in 1..=33 {
        let pads = match tick {
            4 => 0x1c035,
            5 => 0xc035,
            _ => 0,
        };
        r.cpu.call(0x800ffb98, &[2, pads]);
        r.cpu.call_until(RENDER_SCRATCH, &[PLAYER], 0x80111a5c);
        r.scene
            .step_bout_with_pads(&mut r.player, std::slice::from_mut(&mut r.police), pads, 3);
        assert_eq!(
            r.player.word(0x98) as u32 >> 16,
            r.cpu.word(PLAYER + 0x98) >> 16,
            "tick {tick}"
        );
        assert_eq!(
            r.player.word(0x144) as u16,
            r.cpu.word(PLAYER + 0x144) as u16
        );
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn split_screen_chase_cameras_match_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    put(&mut cpu, 0x8018bf28, 1); // split_screen
    put(&mut cpu, 0x8018c334, 2); // numPlayers
    // Run the slot loop from InitUALevel1, retaining the camera setup part of CarInit.
    put(
        &mut cpu,
        0x8012287c,
        0x08000000 | ((0x80124474_u32 >> 2) & 0x03ffffff),
    );
    put(&mut cpu, 0x80122880, 0);
    cpu.put(0x8010a690, &[8, 0, 224, 3, 70, 0, 2, 36]); // split FOV in both views
    for entry in [0x8015060c, 0x801505c4, 0x8010ad84, 0x8010add4, 0x80144be8] {
        cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    for i in 0..Vehicle::ROSTER.len() {
        let kinds = [
            Vehicle::ROSTER[i],
            Vehicle::ROSTER[(i + 1) % Vehicle::ROSTER.len()],
        ];
        let mut cars =
            PlayerVehicle::duel_players(&exe, kinds, Level::Arena, 1, Difficulty::Medium).unwrap();
        for (slot, car) in cars.iter().enumerate() {
            let addr = PLAYER + (slot * car.state.len()) as u32;
            cpu.put(addr, &car.state);
            put(&mut cpu, addr + 0xf4, CarView::Chase as u32);
        }
        cpu.call_until(0x8014c384, &[], 0x8014c4d4);
        for (slot, car) in cars.iter_mut().enumerate() {
            let addr = PLAYER + (slot * car.state.len()) as u32;
            let mut camera = VehicleCamera::duel(car);
            for axis in 0..3 {
                assert_eq!(
                    camera.offset[axis],
                    cpu.word(addr + 0x54 + axis as u32 * 4) as i32
                );
            }
            assert_eq!(camera.pitch, cpu.word(addr + 0x20) as i16 as i32);
            put(&mut cpu, 0x8018c354, slot as u32); // let both pads reach the split-screen gate
            camera.offset = [0; 3];
            camera.pitch = 0;
            for tick in 0..120 {
                let turn = [0, -32, 190, -240, 48][(tick / 8) % 5];
                vec3(&mut car.state, 0x380 + 0x2c, [0, 0, turn]);
                put(&mut cpu, addr + 0x3b4, turn as u32);
                let forward = tick % 3 == 0;
                cpu.put(addr + 0xd9, &[(!forward).into(), forward.into()]);
                cpu.call(0x8014eff4, &[addr]);
                camera.cycle(forward);
                cpu.call(0x80116bcc, &[addr]);
                camera.update(car, false);
                assert_eq!(camera.mode as u32, cpu.word(addr + 0xf4));
                assert_eq!(
                    camera.offset,
                    std::array::from_fn(|j| cpu.word(addr + 12 + 4 * j as u32) as i32),
                    "{kinds:?} slot {slot} tick {tick}"
                );
                assert_eq!(camera.pitch, cpu.word(addr + 4) as i16 as i32);
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn helicopter_camera_original() {
    const HELI: u32 = 0x801a22b8;
    for (level, points) in [
        (Level::Arena, 0x80125bc0),
        (Level::Rooftops, 0x8012b5a4),
        (Level::Arena5, 0x80125bc0),
    ] {
        let mut r = Reference::on_level(Vehicle::YellowJacket, false, level);
        put(&mut r.cpu, 0x8018c3b4, level as u32);
        r.cpu.call(points, &[]);
        r.cpu.random_state = r.player.initial_seed;
        put(&mut r.cpu, 0x8018c378, level as u32 + 1);
        let mut heli = Helicopter::new(&mut r.player);
        r.cpu.call(0x80124d34, &[HELI, u32::MAX, 1]);
        let trigger = r.cpu.call(0x8011c2d0, &[HELI, 0]);
        put(&mut r.cpu, HELI + 0x20c + 0x64, trigger);
        r.cpu.call(0x80120f60, &[HELI]);
        r.cpu.put(HELI + 1, &[0]);
        for entry in [0x8015060c, 0x801505c4, 0x8010ad84] {
            r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        let camera = RENDER_SCRATCH;
        put(&mut r.cpu, 0x8018c658, camera);

        for tick in 0..200 {
            let fields = [3, 4, 5][tick % 3];
            r.player.step(
                Input {
                    accelerate: tick < 150,
                    left: (25..90).contains(&tick),
                    reverse: tick >= 150,
                    ..Default::default()
                },
                fields,
            );
            if level == Level::Rooftops && (45..113).contains(&tick) {
                let height: i32 = if tick < 100 {
                    6000 - (tick as i32 - 45) * 60
                } else {
                    12000
                };
                r.player.state[0x390..0x394].copy_from_slice(&height.to_le_bytes());
            }
            r.cpu.put(PLAYER, &r.player.state);
            put(&mut r.cpu, PLAYER + 0xf4, 4);
            vec3_cpu(&mut r.cpu, PLAYER + 0x48 + 3 * 12, [0, -640, 320]);
            r.cpu.put(0x80170238, &(fields as u16).to_le_bytes());
            r.cpu
                .put(0x8017023a, &(60u16 / fields as u16).to_le_bytes());
            put(&mut r.cpu, 0x8017023c, r.player.frame);
            if matches!(tick, 0 | 113) {
                heli.update(&r.player, fields, true);
                r.cpu.call(0x8014f3b4, &[]);
                let previous =
                    std::array::from_fn(|i| r.cpu.word(HELI + 0x444 + 4 * i as u32) as i32);
                vec3_cpu(&mut r.cpu, camera + 0x5c, previous);
            }
            r.cpu.call(0x80116bcc, &[PLAYER]);
            r.cpu.call(0x8014ea98, &[0]);
            heli.update(&r.player, fields, false);
            let pos: [i32; 3] =
                std::array::from_fn(|i| r.cpu.word(camera + 0x5c + 4 * i as u32) as i32);
            for i in 0..3 {
                let offset = 0x444 + i * 4;
                assert_eq!(
                    &heli.state[offset..offset + 4],
                    &r.cpu.word(HELI + offset as u32).to_le_bytes(),
                    "{level:?} tick {tick} helicopter axis {i}"
                );
            }
            if level == Level::Rooftops && tick == 90 {
                assert!(pos[2] < 6800 && pos[2] > r.cpu.word(HELI + 0x44c) as i32);
            }
            if level == Level::Rooftops && tick == 112 {
                assert!(pos[2] >= 6800);
            }
            assert_eq!(heli.pose[..3], pos, "{level:?} tick {tick}");
            assert_eq!(heli.pose[3], r.cpu.word(PLAYER + 4) as i16 as i32);
            assert_eq!(heli.pose[4], r.cpu.word(PLAYER + 8) as i16 as i32);
        }
    }
}
fn vec3_cpu(cpu: &mut Cpu, at: u32, xyz: [i32; 3]) {
    for (i, v) in xyz.into_iter().enumerate() {
        put(cpu, at + 4 * i as u32, v as u32);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn positional_sound_matches_retail() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    let mut car = PlayerVehicle::contest(
        &exe,
        Vehicle::RoadKill,
        Level::Warehouse,
        1,
        Difficulty::Medium,
    )
    .unwrap();
    let node = RENDER_SCRATCH;
    let parent = node + 0x200;
    let source = node + 0x400;
    put(&mut cpu, 0x8018c4b0, WORLD);
    put(&mut cpu, node + 0x10, parent);
    for i in 0..16 {
        let yaw = i * 257 - 2048;
        let tilt = i * 23 - 160;
        let bank = i * 15 - 101;
        let pos = [600, -300, 7200];
        vec3(&mut car.state, 0x388, pos);
        vec3(&mut car.state, 0x3a0, [tilt, bank, yaw]);
        let angles = [tilt as i16, bank as i16, yaw as i16];
        cpu.put(parent + 0x54, &angles.map(i16::to_le_bytes).concat());
        cpu.call(0x8015d8a4, &[parent + 0x54, parent + 0x14]);
        let mat = ((parent + 0x14) & 0x1fffff) as usize;
        car.state[0x3d0..0x3e2].copy_from_slice(&cpu.ram[mat..mat + 18]);
        for (mode, detached, view, fov) in [
            (CarView::Chase, false, 0, 6),
            (CarView::Chase, false, 0, 11),
            (CarView::Chase, false, 1, 12),
            (CarView::Cockpit, false, 0, 6),
            (CarView::Far, false, 0, 6),
            (CarView::Chase, true, 0, 6),
        ] {
            put(&mut cpu, 0x8018c50c + view * 4, node);
            put(&mut cpu, 0x8018c51c + view * 4, fov);
            let camera = if fov >= 11 {
                VehicleCamera::duel(&car)
            } else {
                VehicleCamera::new(&car, mode)
            };
            let fixed = detached.then_some([1500, -120, 8100, tilt, yaw]);
            let listener = SoundListener::new(&car, &camera, fixed);
            let (rotation, location, offset, pitch) = if let Some([x, y, z, p, h]) = fixed {
                ([p as i16, 0, h as i16], [x, y, z], [0; 3], 0)
            } else {
                (
                    if mode == CarView::Cockpit {
                        angles
                    } else {
                        [0, 0, yaw as i16]
                    },
                    pos,
                    camera.offset,
                    camera.pitch,
                )
            };
            cpu.put(parent + 0x54, &rotation.map(i16::to_le_bytes).concat());
            vec3_cpu(&mut cpu, parent + 0x5c, location);
            cpu.put(
                node + 0x54,
                &([pitch as i16, 0, 0].map(i16::to_le_bytes)).concat(),
            );
            vec3_cpu(&mut cpu, node + 0x5c, offset);
            cpu.call_until(0x8010a774, &[view], 0x8010a9f4);
            for d in [
                [0, 0, 0],
                [1, -2, 3],
                [-320, 0, 0],
                [320, 0, 0],
                [101, 237, -333],
                [3358, 9500, 0],
                [-9000, -6500, -12000],
            ] {
                let point = std::array::from_fn(|j| location[j] + d[j]);
                vec3_cpu(&mut cpu, source, point);
                cpu.call(0x8010461c, &[source]);
                let range = cpu.call(0x80104400, &[]) as i32;
                let x = cpu.call(0x8010440c, &[]) as i32;
                assert_eq!(
                    listener.range_and_x(point),
                    (range, x),
                    "{mode:?} {yaw} {point:?}"
                );
                // Enter the JP mixer with the current request and its division constant.
                for full_range in [false, true] {
                    put(
                        &mut cpu,
                        0x801984ec,
                        if full_range { 0 } else { range as u32 },
                    );
                    put(&mut cpu, 0x801984f0, (-x) as u32);
                    put(&mut cpu, 0x801ff048, 0x801984ec);
                    put(&mut cpu, 0x801ff038, 0x66666667);
                    for volume in [0u8, 10, 33, 57] {
                        put(&mut cpu, 0x8016fed0, u32::from(volume));
                        cpu.call_until(0x801063e0, &[], 0x801065c4);
                        assert_eq!(
                            listener.channels(point, volume, full_range),
                            [cpu.r[20] as i32, cpu.r[18] as i32]
                        );
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn turbo_reserve_matches_original_dashboard() {
    let mut r = Reference::new();
    // Supply the player pointer in s6 before entering the retail fuel calculation.
    put(&mut r.cpu, 0x80142d14, 0x0080b021); // addu s6, a0, zero
    for reserve in [0i16, 1, 8, 9, 10, 11, 12] {
        for remaining in [0i32, 1, 17, 18, 161, 162, 179, 180] {
            r.player.state[0x49e..0x4a0].copy_from_slice(&reserve.to_le_bytes());
            r.player.state[0x230..0x234].copy_from_slice(&remaining.to_le_bytes());
            r.cpu.put(PLAYER, &r.player.state);
            r.cpu.call_until(0x80142d14, &[PLAYER], 0x80142d84);
            r.player.cap_turbo_reserve();
            assert_eq!(r.player.turbo_charge(), r.cpu.r[4] as i32);
            let start = (PLAYER & 0x1fffff) as usize;
            assert_eq!(
                r.player.state[..],
                r.cpu.ram[start..start + r.player.state.len()]
            );
        }
    }
    // A pickup can take the bank over capacity while the current charge is partly used.
    r.player.state[0x49e..0x4a0].copy_from_slice(&11i16.to_le_bytes());
    r.player.state[0x230..0x234].copy_from_slice(&90i32.to_le_bytes());
    r.scene
        .step(&mut r.player, &mut r.police, Input::default(), 3);
    assert_eq!(r.player.turbo_units(), 100);
    assert_eq!(r.player.word(0x230), 0);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn windshield_damage_original() {
    let mut r = Reference::new();
    for pct in -1..=101 {
        let (tier, holes) = PlayerVehicle::windshield_damage(pct);
        assert_eq!(u32::from(tier), r.cpu.call(0x8014279c, &[pct as u32, 1]));
        assert_eq!(holes as u32, r.cpu.word(0x8018c5e4));
    }
    for seed in [1, 0x12345678, u32::MAX] {
        r.player.seed = seed;
        r.cpu.random_state = seed;
        r.cpu.call(0x80146008, &[]);
        for (i, [x, y]) in r.player.windshield_holes().into_iter().enumerate() {
            let coord = r.cpu.word(0x8019d444 + 0x69c + i as u32 * 20 + 8);
            assert_eq!([x, y], [coord as i16 as i32, (coord >> 16) as i16 as i32]);
        }
        assert_eq!(r.player.seed, r.cpu.random_state);
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn cockpit_wheel_tracks_retail_steering_primitive() {
    let mut r = Reference::new();
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let quad = tm1_assets::scene::Dmd::parse(&data)
        .unwrap()
        .steering_wheel()
        .unwrap();
    let [right, left] = quad.handlebars();
    let primitives = [
        (quad, [125, 230], 0x8019e54c),
        (left, [159, 202], 0x8019e574),
        (right, [159, 202], 0x8019e59c),
    ];
    let texture_origin = [0, 8].map(|shift| {
        (0..4)
            .map(|i| (r.cpu.word(0x8019d978 + 8 * i) >> shift & 255) as i32)
            .min()
            .unwrap()
    });
    let mut angle = 0;
    for flags in [1, 1, 1, 5, 5, 2, 2, 10, 10, 3, 0, 0, 0, 0, 0] {
        for (bit, offset) in [0xca, 0xcb, 0xce, 0xcf].into_iter().enumerate() {
            r.player.state[offset] = (flags >> bit) & 1;
        }
        r.cpu.put(PLAYER, &r.player.state);
        r.cpu.call(0x80145264, &[RENDER_SCRATCH]);
        r.cpu.call(0x80145568, &[RENDER_SCRATCH]);
        angle = r.player.cockpit_steering_angle(angle);
        let rotation = r.player.wheel_sin_cos(i64::from(angle * 4096 / 360));
        for (face, center, at) in primitives {
            for (i, expected) in face.project(center, rotation).into_iter().enumerate() {
                let xy = r.cpu.word(at + 8 * i as u32);
                let uv = r.cpu.word(at + 8 * i as u32 + 4);
                assert_eq!(
                    face.uv[i],
                    [
                        (uv & 255) as i32 - texture_origin[0],
                        (uv >> 8 & 255) as i32 - texture_origin[1]
                    ]
                );
                assert_eq!(
                    expected,
                    [xy as i16 as i32, (xy >> 16) as i16 as i32],
                    "flags={flags} angle={angle} corner={i}"
                );
            }
        }
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn control_options_match_original() {
    let mut r = Reference::new();
    r.cpu.put(PLAYER, &r.player.state);
    for layout in [0, 1, 2, 3, 1] {
        r.player.set_pad_layout(layout);
        r.cpu.call(0x80150318, &[PLAYER, u32::from(layout)]);
        let start = (PLAYER & 0x1fffff) as usize + 0xc8;
        assert_eq!(
            &r.player.state[0xc8..0xc8 + 44],
            &r.cpu.ram[start..start + 44]
        );
        for input in [
            Input {
                accelerate: true,
                ..Default::default()
            },
            Input {
                fire_guns: true,
                ..Default::default()
            },
            Input {
                missile: true,
                ..Default::default()
            },
            Input {
                reverse: true,
                left: true,
                hard_turn: true,
                ..Default::default()
            },
        ] {
            let mask = r.player.keyboard_buttons(input);
            if layout == 1 {
                assert_eq!(mask, input.ps1_buttons());
            }
            r.cpu.call(0x800ffb98, &[2, u32::from(mask)]);
            // Re-evaluate the selected original bindings for these keyboard actions.
            for (binding, active) in [
                (29, input.accelerate),
                (34, input.fire_guns),
                (35, input.missile),
                (25, input.reverse),
                (27, input.left),
                (31, input.hard_turn),
            ] {
                let btn = u32::from(r.cpu.ram[start + binding]);
                assert_eq!(r.cpu.call(0x800ffbe4, &[btn, 0]) != 0, active);
            }
        }
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftops_passage_view_matches_disc() {
    use CarView::{Chase, Cockpit, Far};
    use PlayerView::{Car, Helicopter};
    let mut r = Reference::on_level(Vehicle::YellowJacket, false, Level::Rooftops);
    r.cpu.put(PLAYER, &r.player.state);
    put(&mut r.cpu, 0x8018c378, 6);
    put(&mut r.cpu, 0x8018c658, RENDER_SCRATCH);
    r.cpu.put(0x8018c364, &[1]); // helicopter unlocked
    for entry in [0x8015060c, 0x801505c4, 0x8010ad84, 0x8010add4, 0x80144be8] {
        r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    let number = |view| match view {
        Car(mode) => mode as u32,
        Helicopter => 4,
    };
    for initial in [Car(Cockpit), Car(Chase), Car(Far), Helicopter] {
        let mut passage = RooftopPassage::default();
        let mut view = initial;
        put(&mut r.cpu, 0x8018c33c, 1);
        put(&mut r.cpu, PLAYER + 0xf4, number(initial));
        put(&mut r.cpu, PLAYER + 0xf8, number(initial));
        for (group, direction) in [
            (5, 0),
            (7, 0),
            (8, 0),
            (5, 0),
            (7, 0),
            (6, 1),
            (6, 0),
            (6, -1),
            (6, 0),
            (5, 0),
        ] {
            let before = view;
            view = match (direction, view) {
                (1, Car(Cockpit)) | (-1, Car(Far)) => Car(Chase),
                (1, Car(Chase)) | (-1, Helicopter) => Car(Far),
                (1, Car(Far)) | (-1, Car(Cockpit)) => Helicopter,
                (1, Helicopter) | (-1, Car(Chase)) => Car(Cockpit),
                _ => view,
            };
            let selection = passage.update(group, before, view, false);
            view = selection.unwrap_or(view);
            r.cpu.put(PLAYER + 0xc0, &group.to_le_bytes());
            r.cpu.put(
                PLAYER + 0xd9,
                &[(direction == -1).into(), (direction == 1).into()],
            );
            r.cpu.call(0x8014eff4, &[PLAYER]);
            assert_eq!(
                number(view),
                r.cpu.word(PLAYER + 0xf4),
                "{initial:?} group {group} direction {direction}"
            );
            assert_eq!(passage.return_view.is_none(), r.cpu.word(0x8018c33c) != 0);
            if let Some(saved) = passage.return_view {
                assert_eq!(number(saved), r.cpu.word(PLAYER + 0xf8));
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftops_fall_camera_original() {
    let mut r = Reference::on_level(Vehicle::YellowJacket, false, Level::Rooftops);
    let cs = RENDER_SCRATCH;
    put(&mut r.cpu, 0x8018c658, cs);
    put(&mut r.cpu, 0x8018c378, Level::Rooftops as u32 + 1);
    for entry in [0x8015060c, 0x801505c4, 0x8010ad84, 0x8010add4, 0x80144be8] {
        r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    r.cpu.put(0x8010a690, &[8, 0, 224, 3, 60, 0, 2, 36]);
    for (mode, heading) in [
        (CarView::Chase, 523),
        (CarView::Cockpit, 1230),
        (CarView::Far, -343),
    ] {
        r.player.state[0x390..0x394].copy_from_slice(&6801i32.to_le_bytes());
        r.player.state[0x3a8..0x3ac].copy_from_slice(&i32::to_le_bytes(heading));
        let mut view = VehicleCamera::new(&r.player, mode);
        assert!(view.fixed_pose.is_none());
        r.cpu.put(PLAYER, &r.player.state);
        r.cpu.call(0x80124474, &[PLAYER]);
        put(&mut r.cpu, PLAYER + 0xf4, mode as u32);
        vec3_cpu(&mut r.cpu, PLAYER + 0xc, view.offset);
        put(&mut r.cpu, PLAYER + 4, view.pitch as u32);
        for i in 0..35 {
            let pos = [321 + i * 12, 208 - i * 6, 6799 - i * i];
            for (j, value) in pos.into_iter().enumerate() {
                let at = 0x388 + j * 4;
                r.player.state[at..at + 4].copy_from_slice(&i32::to_le_bytes(value));
            }
            vec3_cpu(&mut r.cpu, PLAYER + 0x388, pos);
            r.cpu.call(0x80116bcc, &[PLAYER]);
            r.cpu.call(0x8014fc14, &[0, PLAYER + 0x388, VIEW_OUTPUT]);
            view.update(&r.player, false);
            let pose = view.fixed_pose.unwrap();
            let expected: [i32; 3] =
                std::array::from_fn(|j| r.cpu.word(cs + 0x5c + 4 * j as u32) as i32);
            assert_eq!(pose[..3], expected);
            assert_eq!(pose[3], r.cpu.word(cs + 0x54) as i16 as i32);
            assert_eq!(pose[4], r.cpu.word(cs + 0x58) as i16 as i32);
            assert_eq!(r.cpu.word(PLAYER + 0xf4), 5);
        }
        r.player.state[0x390..0x394].copy_from_slice(&6800i32.to_le_bytes());
        put(&mut r.cpu, PLAYER + 0x390, 6800);
        r.cpu.call(0x8014fc14, &[0, PLAYER + 0x388, VIEW_OUTPUT]);
        view.update(&r.player, false);
        assert!(view.fixed_pose.is_none());
        assert_eq!(view.mode as u32, r.cpu.word(PLAYER + 0xf4));
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn vehicle_and_wall_contact_original() {
    for opponent in [true, false] {
        let mut r = Reference::new();
        // First fixture drives into a stationary police car on the arena floor.
        // Second starts in the normal taxi spawn and drives into the arena wall.
        let pos = r.player.position();
        if opponent {
            vec3(&mut r.police.state, 0x444, [pos[0] + 380, pos[1] + 380, 16]);
        }
        for entry in [0x80125bc0, 0x8012cd90, 0x80130680, 0x80130a8c, 0x801312f8] {
            r.cpu.call(entry, &[]);
        }
        let mut hit = false;
        for tick in 0..400 {
            r.player.step(
                Input {
                    accelerate: true,
                    ..Default::default()
                },
                3,
            );
            r.sync();
            let mut object = 0;
            r.cpu
                .call_observed(0x80113858, &[PLAYER, 1], 0x80000080, |cpu| {
                    if cpu.pc() == 0x801141b8 {
                        object = cpu.word(cpu.word(cpu.r[5] + 32) + 0xc0);
                    }
                });
            r.cpu.call(0x8013148c, &[PLAYER, 1]);
            if r.cpu.word(PLAYER + 0x258 + 0x14) >> 16 >= 3 {
                for (src, dst, n) in [
                    (0x20, 0x9c, 12),
                    (8, 0x90, 12),
                    (0x38, 0xa8, 12),
                    (0x50, 0xb4, 32),
                ] {
                    let at = (PLAYER as usize & 0x1fffff) + 0x380 + src;
                    let data = r.cpu.ram[at..at + n].to_vec();
                    r.cpu.put(PLAYER + 0x380 + dst, &data);
                }
            }
            r.scene.phase(&mut r.player, &mut r.police, 1, 3);
            r.compare();
            if object == if opponent { 50 } else { 8 } {
                println!(
                    "{} at tick {tick}, taxi {:?}, police {:?}",
                    if opponent {
                        "vehicle fixture"
                    } else {
                        "wall fixture"
                    },
                    r.player.position(),
                    r.police.position()
                );
                hit = true;
                break;
            }
        }
        assert!(hit);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn police_update_original() {
    let mut r = Reference::new();
    r.police.state[0x40] = 0;
    // Level-specific tables and disabled subsystems for the isolated fixture.
    for entry in [0x80125bc0, 0x8012cd90, 0x80130680, 0x80130a8c, 0x801312f8] {
        r.cpu.call(entry, &[]);
    }
    for at in [
        0x801417d0, 0x801466b0, 0x80146930, 0x801073cc, 0x8014b208, 0x80108604,
    ] {
        r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    r.cpu.put(0x8018c378, &1u16.to_le_bytes());
    put(&mut r.cpu, 0x8018c3b4, 0);
    for tick in 1..=45 {
        r.player.frame = tick;
        r.sync();
        r.cpu.call(0x8011950c, &[POLICE]);
        r.police
            .update(&mut r.player, 3, None, r.scene.native.as_ptr());
        r.compare();
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn player_special_launch_original() {
    for vehicle in Vehicle::ROSTER.into_iter().filter(|&v| {
        !matches!(
            v,
            Vehicle::Outlaw
                | Vehicle::Hammerhead
                | Vehicle::Thumper
                | Vehicle::Warthog
                | Vehicle::RoadKill
                | Vehicle::Spectre
        )
    }) {
        let mut r = Reference::with_vehicle(vehicle);
        for at in [0x80107f84, 0x80108034] {
            r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        vec3(&mut r.player.state, 0x388, [2700, 2200, 400]);
        vec3(&mut r.police.state, 0x444, [2750, 3000, 400]);
        r.player.state[0xd2] = 1;
        r.sync();
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        r.cpu.call(0x8013148c, &[PLAYER, 1]);
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        for _ in 0..3 {
            let pose = |cpu: &Cpu| {
                let b = 0x80198c28;
                [0, 4, 8, 12, 16, 20, 0x34, 0x28].map(|off| {
                    if off == 0x34 {
                        (cpu.word(b + off) & 255) as i32
                    } else {
                        cpu.word(b + off) as i32
                    }
                })
            };
            let main_pose = pose(&r.cpu);
            r.cpu.call(0x8010d03c, &[]);
            r.cpu.call(0x80137d2c, &[]);
            r.scene.phase(&mut r.player, &mut r.police, 2, 3);
            r.compare();
            for (secondary, expected) in [(false, main_pose), (true, pose(&r.cpu))] {
                let mut bullets = [[0; 8]; 40];
                assert_eq!(r.scene.drawn_bullets(secondary, &mut bullets), 1);
                assert_eq!(bullets[0], expected, "{vehicle:?} secondary {secondary}");
            }
            let mut segments = [[[0; 3]; 2]; 40];
            assert_eq!(r.scene.bullet_segments(&mut segments), 1, "{vehicle:?}");
            for (point, offset) in [12, 0].into_iter().enumerate() {
                for axis in 0..3 {
                    assert_eq!(
                        segments[0][point][axis],
                        r.cpu.word(0x80198c28 + offset + 4 * axis as u32) as i32,
                        "{vehicle:?}"
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn outlaw_taser_original() {
    for (level, distance, alive, expected_target) in [
        (Level::Arena, 647, true, 50),
        (Level::Arena, 648, true, -1),
        (Level::Arena, 120, false, -1),
        (Level::Rooftops, 648, true, -1),
    ] {
        let mut r = Reference::on_level(Vehicle::Outlaw, false, level);
        put(&mut r.cpu, 0x8018c3b4, level as u32);
        for at in [0x80107f84, 0x80108034, 0x80136a28, 0x80136c00, 0x80137140] {
            r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        vec3(&mut r.player.state, 0x388, [2700, 2200, 400]);
        vec3(&mut r.police.state, 0x444, [2700, 2200 + distance, 400]);
        if !alive {
            r.police.state[0x24c..0x250].fill(0);
        }
        r.player.state[0xd2] = 1;
        let hp = r.police.health().0;
        r.sync();
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        r.cpu.call(0x8013148c, &[PLAYER, 1]);
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        assert_eq!(r.cpu.word(0x80199d88 + 12) as i32, expected_target);
        assert_eq!(r.player.weapon().1, 14);
        // Return a zero camera translation so the inputs to the retail projection
        // contain the world-space polyline generated by draw_tasers.
        r.cpu.put(0x8010af54, &[8, 0, 224, 3, 0x0b, 0x80, 2, 0x3c]);
        r.cpu.put(0x800b0000, &[0; 12]);
        for frame in 1..=11 {
            let mut points = Vec::new();
            let mut effects = Vec::new();
            r.cpu
                .call_observed(0x801333d0, &[RENDER_SCRATCH, 0], 0x80000080, |cpu| {
                    if cpu.pc() == 0x800fe9c4 {
                        points.push([0, 4, 8].map(|off| cpu.word(cpu.r[5] + off) as i32));
                    } else {
                        let kind = match cpu.pc() {
                            0x8013721c => 32, // do_puff
                            0x801369b0 => 1,  // do_simple_spark
                            0x80135f90 => {
                                // do_flash passes the selected BurnInfo sprite to init_explosion.
                                let sprite = cpu.word(cpu.r[29] + 16);
                                38 + ((sprite - 0x8019c6d8) / 8) as i32
                            }
                            _ => return,
                        };
                        effects.push(BattleEvent::Effect {
                            kind,
                            position: [0, 4, 8].map(|off| cpu.word(cpu.r[4] + off) as i32),
                        });
                    }
                });
            r.cpu.call(0x80137d2c, &[]);
            r.scene.phase(&mut r.player, &mut r.police, 2, 3);
            r.compare();
            let mut arcs = [[[0; 3]; 16]; 3];
            let count = r.scene.taser_arcs(&mut arcs);
            assert_eq!(count, usize::from(frame <= 10));
            assert_eq!(points.as_slice(), arcs[..count].as_flattened());
            assert_eq!(r.scene.audio, effects, "{level:?} update {frame}");
            assert_eq!(
                r.police.health().0,
                hp - if frame >= 10 && expected_target == 50 {
                    8
                } else {
                    0
                }
            );
        }
    }
}

fn identity_projection(cpu: &mut Cpu) -> ([i32; 2], [i32; 2]) {
    let matrix = cpu.call(0x8010af70, &[0]);
    cpu.put(matrix, &[0; 32]);
    for offset in [0, 8, 16] {
        cpu.put(matrix + offset, &4096i16.to_le_bytes());
    }
    let translation = cpu.call(0x8010af54, &[0]);
    cpu.put(translation, &[0; 12]);
    cpu.call(0x8010aed4, &[0, VIEW_OUTPUT, VIEW_OUTPUT + 4]);
    let center = [0, 4].map(|k| cpu.word(VIEW_OUTPUT + k) as i32);
    let focal = [0x80109e3c, 0x80109e74].map(|f| cpu.call(f, &[0]) as i32);
    (center, focal)
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn fragment_draw_packets_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let art = tm1_assets::scene::Dmd::parse(&data)
        .unwrap()
        .fragment_frames()
        .unwrap();
    let mut r = Reference::new();
    let cpu = &mut r.cpu;
    let (center, focal) = identity_projection(cpu);
    let f = 0x80199db8;
    let packet = RENDER_SCRATCH + 0x100;
    put(cpu, RENDER_SCRATCH, 0x80010000);
    put(cpu, RENDER_SCRATCH + 12, packet + 160);
    put(cpu, f + 28, 40);
    for (group, frames) in art.iter().enumerate() {
        put(cpu, f + 32, 0x8019c5b8 + group as u32 * 32);
        for (phase, sprite) in frames.iter().enumerate() {
            cpu.put(f + 24, &[phase as u8]);
            for pos in [
                [-26, 17, 240],
                [0, 0, -33],
                [0, 0, -32],
                [0, 0, 7],
                [0, 0, 32000],
                [0, 0, 32001],
                [900, 0, 8],
            ] {
                for (axis, value) in pos.into_iter().enumerate() {
                    put(cpu, f + axis as u32 * 4, value as u32);
                }
                put(cpu, RENDER_SCRATCH + 8, packet);
                cpu.call(0x80137e7c, &[RENDER_SCRATCH, 0]);
                let emitted = cpu.word(RENDER_SCRATCH + 8) != packet;
                let corners = project_fragment(pos, focal, center);
                assert_eq!(emitted, corners.is_some(), "{pos:?}");
                let Some(corners) = corners else {
                    continue;
                };
                assert_eq!(cpu.word(packet + 4) >> 24, 0x2d);
                for (i, [x, y]) in corners.into_iter().enumerate() {
                    let xy = cpu.word(packet + 8 + i as u32 * 8);
                    assert_eq!(
                        [xy as i16 as i32, (xy >> 16) as i16 as i32],
                        [center[0] + x, center[1] + y]
                    );
                    let uv = cpu.word(packet + 12 + i as u32 * 8);
                    assert_eq!([uv as u8, (uv >> 8) as u8], sprite.uv[i]);
                }
                assert_eq!((cpu.word(packet + 12) >> 16) as u16, sprite.texture.clut);
                assert_eq!((cpu.word(packet + 20) >> 16) as u16, sprite.texture.page);
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn particles_match_retail_packets() {
    for (kind, routine, duration) in [
        (35, 0x80136e18, 104),
        (53, 0x801370d0, 2),
        (8, 0x80137140, 88),
        (31, 0x80137190, 24),
        (55, 0x80136aa0, 32),
        (29, 0x80136cf0, 16),
    ] {
        verify_particle_packets(kind, routine, duration);
    }
}

fn verify_particle_packets(kind: i32, routine: u32, duration: i32) {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let dmd = tm1_assets::scene::Dmd::parse(&data).unwrap();
    let art = particle_frame(kind, duration).unwrap().animation;
    let frames = match art {
        ParticleAnimation::Fire => dmd.flame_frames().unwrap().to_vec(),
        ParticleAnimation::Smoke | ParticleAnimation::Steam => dmd
            .smoke_frames(art == ParticleAnimation::Steam)
            .unwrap()
            .to_vec(),
        ParticleAnimation::Explosion => dmd.explosion_frames().unwrap().to_vec(),
        _ => unreachable!(),
    };
    let mut r = Reference::build(Vehicle::YellowJacket, true);
    r.cpu.call(0x80137268, &[]);
    let pos = [32, if kind == 8 { 3200 } else { 400 }, 36];
    let ot = 0x80010000;
    let packet = RENDER_SCRATCH + 0x100;
    put(&mut r.cpu, RENDER_SCRATCH, ot);
    put(&mut r.cpu, RENDER_SCRATCH + 12, packet + 256);
    for update in 0..=duration + 1 {
        r.cpu.call(0x801360bc, &[]);
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        for view in 0..=1 {
            if update == 0 && view == 1 {
                for (k, n) in pos.into_iter().enumerate() {
                    put(&mut r.cpu, VIEW_OUTPUT + k as u32 * 4, n as u32);
                }
                r.cpu.call(routine, &[VIEW_OUTPUT]);
                {
                    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
                    unsafe extern "C" {
                        fn arena_effect(p: *const i32, kind: i32);
                    }
                    unsafe {
                        tm1_use_scene(r.scene.native.as_ptr());
                        arena_effect(pos.as_ptr(), kind);
                        tm1_use_scene(std::ptr::null_mut());
                    }
                }
                assert_eq!(
                    r.scene.drawn_particles(false, &mut [[0; 5]; 80]),
                    0,
                    "late spawn leaked into primary view"
                );
            }
            let m = r.cpu.call(0x8010af70, &[view]);
            let matrix = crate::math::Matrix {
                rotation: if update % 2 == 0 {
                    [[4096, 0, 0], [0, 0, -4096], [0, 4096, 0]]
                } else {
                    [[2896, 0, 2896], [2896, 0, -2896], [0, 4096, 0]]
                },
                ..Default::default()
            };
            for (i, v) in matrix.rotation.iter().flatten().enumerate() {
                r.cpu.put(m + i as u32 * 2, &v.to_le_bytes());
            }
            let translation = r.cpu.call(0x8010af54, &[view]);
            r.cpu.put(translation, &[0; 12]);
            r.cpu
                .call(0x8010aed4, &[view, VIEW_OUTPUT, VIEW_OUTPUT + 4]);
            let center = [0, 4].map(|k| r.cpu.word(VIEW_OUTPUT + k) as i32);
            let focal = [0x80109e3c, 0x80109e74].map(|f| r.cpu.call(f, &[view]) as i32);
            put(&mut r.cpu, RENDER_SCRATCH + 8, packet);
            r.cpu.call(0x80136288, &[RENDER_SCRATCH, view]);
            let count = (r.cpu.word(RENDER_SCRATCH + 8) - packet) / 160;
            let mut particles = [[0; 5]; 80];
            assert_eq!(
                r.scene.drawn_particles(view == 1, &mut particles),
                count as usize,
                "update {update}, view {view}"
            );
            if count == 0 {
                continue;
            }
            assert_eq!(count, 1);
            let [x, y, z, drawn_kind, life] = particles[0];
            assert_eq!(kind, drawn_kind);
            let sprite = particle_frame(kind, life).unwrap();
            let phase = sprite.frame;
            assert_eq!(
                [x, y, z],
                [
                    pos[0],
                    pos[1],
                    pos[2] + if kind == 8 { 10 * update } else { 0 }
                ]
            );
            let [base, top] =
                [[x, y, z], [x, y, z + 80]].map(|p| crate::math::multiply(&matrix, p, false));
            let corners = r
                .player
                .particle_quad(
                    base,
                    (kind == 35).then_some(top),
                    focal,
                    center,
                    frames[sprite.first].width,
                    sprite.scale,
                )
                .unwrap();
            for (i, [u, v]) in corners.into_iter().enumerate() {
                let xy = r.cpu.word(packet + 8 + i as u32 * 8);
                assert_eq!(
                    [u + center[0], v + center[1]],
                    [xy as i16 as i32, (xy >> 16) as i16 as i32],
                    "update {update} view {view} corner {i}"
                );
                let uv = r.cpu.word(packet + 12 + i as u32 * 8);
                assert_eq!(frames[phase as usize].uv[i], [uv as u8, (uv >> 8) as u8]);
            }
            if kind == 55 {
                assert_eq!(r.cpu.word(packet + 4), 0x2eff4040);
            } else {
                assert_eq!(r.cpu.word(packet + 4) >> 24, 0x2f);
            }
            assert_eq!(
                (r.cpu.word(packet + 12) >> 16) as u16,
                frames[phase].texture.clut
            );
            assert_eq!(
                (r.cpu.word(packet + 20) >> 16) as u16,
                frames[phase].texture.page
            );
            let bucket = ((base[2] >> 3) - sprite.depth_bias / 8).max(0);
            assert_eq!(
                r.cpu.word(ot + 4 * bucket as u32) & 0xffffff,
                packet & 0xffffff
            );
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn bullet_draw_packets_original() {
    use crate::bullet::{Primitive, project};
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let art = tm1_assets::scene::Dmd::parse(&data)
        .unwrap()
        .special_bullet_frames()
        .unwrap();
    let mut r = Reference::new();
    let cpu = &mut r.cpu;
    let (center, focal) = identity_projection(cpu);
    let ot = 0x80010000;
    let packet = RENDER_SCRATCH + 0x100;
    put(cpu, RENDER_SCRATCH, ot);
    put(cpu, RENDER_SCRATCH + 12, packet + 256);
    let bullet = 0x80198c28;
    for kind in [0, 2, 4, 6, 5, 7, 15, 17, 19, 21] {
        for life in 1..=8 {
            let head = [-95, 41, 390];
            let tail = [101, -33, 422];
            for (i, n) in head.into_iter().chain(tail).enumerate() {
                put(cpu, bullet + 4 * i as u32, n as u32);
            }
            cpu.put(bullet + 0x34, &[kind as u8]);
            put(cpu, bullet + 0x28, life as u32);
            put(cpu, RENDER_SCRATCH + 8, packet);
            put(cpu, ot + 4 * (head[2] as u32 >> 3), 0xffffff);
            cpu.put(packet, &[0; 160]);
            cpu.call(0x8010d434, &[RENDER_SCRATCH, 0]);
            assert_eq!(
                cpu.word(ot + 4 * (head[2] as u32 >> 3)) & 0xffffff,
                packet & 0xffffff
            );
            let xy = |at| {
                let w = cpu.word(packet + at);
                [
                    w as i16 as i32 - center[0],
                    (w >> 16) as i16 as i32 - center[1],
                ]
            };
            let color = |at| {
                let [r, g, b, _] = cpu.word(packet + at).to_le_bytes();
                [r, g, b]
            };
            match project(kind, life, head, tail, focal, center).unwrap() {
                Primitive::Line { points, colors } => {
                    assert_eq!(points, [8, 16].map(xy));
                    assert_eq!(colors, [4, 12].map(color));
                }
                Primitive::Triangle { points, colors } => {
                    assert_eq!(points, [8, 16, 24].map(xy));
                    assert_eq!(colors, [4, 12, 20].map(color));
                }
                Primitive::Sprite {
                    center: [x, y],
                    radius,
                    group,
                    frame,
                    tint,
                } => {
                    assert_eq!(
                        [8, 16, 24, 32].map(xy),
                        [
                            [x - radius, y - radius],
                            [x + radius, y - radius],
                            [x - radius, y + radius],
                            [x + radius, y + radius]
                        ]
                    );
                    assert_eq!(color(4), tint);
                    let sprite = &art[group].unwrap()[frame];
                    let uv = [12, 20, 28, 36].map(|at| {
                        let [u, v, _, _] = cpu.word(packet + at).to_le_bytes();
                        [u, v]
                    });
                    assert_eq!(sprite.uv, uv, "kind {kind}, life {life}");
                    assert_eq!(sprite.texture.clut, (cpu.word(packet + 12) >> 16) as u16);
                    assert_eq!(sprite.texture.page, (cpu.word(packet + 20) >> 16) as u16);
                }
            }
        }
    }
}

fn observe_flame_audio(
    cpu: &Cpu,
    origin: &mut [i32; 3],
    special: i32,
    requests: &mut Vec<BattleEvent>,
) {
    let kind = match cpu.pc() {
        0x8010461c => {
            *origin = [0, 4, 8].map(|off| cpu.word(cpu.r[4] + off) as i32);
            return;
        }
        0x80107c6c if cpu.r[4] == 7 => 11,
        0x80107f84 => special,
        _ => return,
    };
    requests.push(BattleEvent::Effect {
        kind,
        position: *origin,
    });
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn flame_launch_positions_and_full_emitters_match_retail() {
    let mut r = Reference::with_vehicle(Vehicle::Thumper);
    for at in [0x80107c6c, 0x80107f84, 0x80108034] {
        r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    let initial = r.player.state.clone();
    for (shot, rear) in [false, true, false, false, false, true, false]
        .into_iter()
        .enumerate()
    {
        r.player.state.clone_from(&initial);
        r.player.state[0xd2] = 1;
        vec3(&mut r.player.state, 0x388, [2700, 2200, 1000]);
        vec3(
            &mut r.player.state,
            0x3a0,
            [127, 0, 234 + 713 * shot as i32],
        );
        if rear {
            r.player.state[0x460..0x464].copy_from_slice(&7i32.to_le_bytes());
            r.player.state[0x472..0x474].copy_from_slice(&2i16.to_le_bytes());
        }
        r.sync();
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        let mut origin = [0; 3];
        let mut expected = Vec::new();
        r.cpu
            .call_observed(0x8013148c, &[PLAYER, 1], 0x80000080, |cpu| {
                observe_flame_audio(cpu, &mut origin, 10, &mut expected)
            });
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        assert_eq!(expected.len(), if rear || shot >= 5 { 1 } else { 2 });
        assert_eq!(
            expected,
            r.scene
                .audio
                .iter()
                .filter(|v| matches!(v, BattleEvent::Effect { kind: 10 | 11, .. }))
                .cloned()
                .collect::<Vec<_>>(),
            "shot {shot}"
        );
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn flamethrowers_original() {
    for (rear, distance, saturated) in [
        (false, 320, false),
        (true, -240, false),
        (false, 1000, false),
        (false, 320, true),
    ] {
        let mut r = Reference::build(Vehicle::Thumper, true);
        for addr in [0x80107c6c, 0x80107f84, 0x80108034] {
            r.cpu.put(addr, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        r.cpu.call(0x80137268, &[]);
        vec3(&mut r.player.state, 0x388, [2700, 2200, 1000]);
        vec3(&mut r.police.state, 0x444, [2700, 2200 + distance, 1000]);
        r.player.state[0xd2] = 1;
        if rear {
            r.player.state[0x460..0x464].copy_from_slice(&7i32.to_le_bytes());
            r.player.state[0x472..0x474].copy_from_slice(&2i16.to_le_bytes());
        }
        r.sync();
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        r.cpu.call(0x8013148c, &[PLAYER, 1]);
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        assert_eq!(r.player.weapon().1, if rear { 1 } else { 15 });
        if saturated {
            unsafe extern "C" {
                fn arena_effect(p: *const i32, kind: i32);
            }
            let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            for slot in 0..84 {
                let pos = [-1000 - slot, 100, 200];
                for (k, n) in pos.into_iter().enumerate() {
                    put(&mut r.cpu, VIEW_OUTPUT + 4 * k as u32, n as u32);
                }
                let flash = slot % 19 == 1;
                r.cpu
                    .call(if flash { 0x801370d0 } else { 0x80136e18 }, &[VIEW_OUTPUT]);
                unsafe {
                    tm1_use_scene(r.scene.native.as_ptr());
                    arena_effect(pos.as_ptr(), if flash { 53 } else { 35 });
                    tm1_use_scene(std::ptr::null_mut());
                }
            }
            assert_eq!(r.cpu.call(0x8013624c, &[]), u32::MAX);
        }
        let hp = r.police.health().0;
        let mut bursts = 0;
        for _ in 0..26 {
            r.cpu.call(0x801351b0, &[]);
            r.cpu.call(0x801360bc, &[]);
            r.scene.phase(&mut r.player, &mut r.police, 2, 3);
            r.compare();
            let mut particles = [[0; 4]; 80];
            let count = r.scene.flame_particles(&mut particles);
            let mut original = Vec::new();
            let mut original_static = Vec::new();
            for slot in 0..80 {
                let p = 0x8019a088 + slot * 64;
                let age = r.cpu.word(p + 0x18) as i32;
                if age >= 0 {
                    let x = r.cpu.word(p) as i32;
                    if saturated && x <= -1000 {
                        let flash = (-1000 - x) % 19 == 1;
                        let life = if flash {
                            2
                        } else {
                            (r.cpu.word(p + 0x2c) as i16 as i32 + 1) * 8
                        };
                        original_static.push([
                            x,
                            100,
                            200,
                            if flash { 53 } else { 35 },
                            life - age,
                        ]);
                        continue;
                    }
                    let frame = (r.cpu.word(p + 0x24) - 0x8019c638) / 8;
                    original.push([
                        r.cpu.word(p) as i32,
                        r.cpu.word(p + 4) as i32,
                        r.cpu.word(p + 8) as i32,
                        frame as i32 + age,
                    ]);
                }
            }
            assert_eq!(original, particles[..count]);
            if saturated {
                let mut effects = [[0; 5]; 122];
                let n = r.scene.effects(&mut effects);
                assert!(
                    original_static
                        .iter()
                        .eq(effects[..n].iter().filter(|e| matches!(e[3], 35 | 53)))
                );
            }
            bursts += count;
        }
        if saturated {
            assert!(bursts > 0 && bursts < 290);
        } else {
            assert_eq!(bursts, if rear { 66 } else { 290 });
        }
        assert_eq!(r.police.health().0 < hp, distance.abs() < 400);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn special_missiles_original() {
    for vehicle in [Vehicle::Warthog, Vehicle::Spectre, Vehicle::RoadKill] {
        for locked in [true, false] {
            let mut r = Reference::with_vehicle(vehicle);
            for at in [
                0x80107c6c, 0x80107d18, 0x80107dbc, 0x80136a28, 0x80136c00, 0x80137140, 0x80137190,
                0x80107f84, 0x80108034,
            ] {
                r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
            }
            let missile = |i| 0x801a9c10u32 + 0x3e8 * i;
            let cs = |i| MISSILE_CS + 0x110 * i;
            // Retail's world-loaded missile nodes enable its trail update. Tag each
            // sprite UV with its frame index to read the animated polygon order.
            put(&mut r.cpu, 0x8018bfac, POLICE_CS);
            put(&mut r.cpu, 0x8018bfb4, POLICE_CS);
            put(&mut r.cpu, 0x8018bfbc, TAXI_CS);
            put(&mut r.cpu, 0x8018c52c, 0x8019c7b8);
            for frame in 0..8 {
                r.cpu.put(0x8019c7b8 + 8 * frame, &[frame as u8, 0, 0, 0]);
            }
            for i in 0..10 {
                put(&mut r.cpu, missile(i) + 0x24, cs(i));
                put(&mut r.cpu, missile(i) + 0x100, MISSILE_AUX + 0x110 * i);
                put(&mut r.cpu, missile(i) + 0x34, (-15i32) as u32);
                put(&mut r.cpu, cs(i) + 0xc0, 200 + i);
                r.cpu.put(missile(i) + 0x30, &[i as u8]);
            }
            vec3(&mut r.player.state, 0x388, [2700, 2200, 400]);
            vec3(&mut r.player.state, 0x3a0, [0, 0, 0]);
            vec3(
                &mut r.police.state,
                0x444,
                [2750, if locked { 3000 } else { 1000 }, 400],
            );
            r.player.state[0xd2] = 1;
            r.sync();
            r.cpu.call(0x80113858, &[PLAYER, 1]);
            r.cpu.call(0x8013148c, &[PLAYER, 1]);
            r.scene.phase(&mut r.player, &mut r.police, 1, 3);
            assert_eq!(
                r.player.weapon().1,
                if vehicle == Vehicle::Spectre { 15 } else { 13 }
            );
            for tick in 0..93 {
                let mut poses = [[0; 13]; 10];
                let count = r.scene.missile_poses(&mut poses);
                let active: Vec<_> = (0..10)
                    .filter(|&i| r.cpu.word(missile(i) + 0x34) as i32 > 0)
                    .collect();
                assert_eq!(count, active.len(), "{vehicle:?} {tick}");
                if tick == 0 {
                    assert_eq!(count, if vehicle == Vehicle::Warthog { 3 } else { 1 });
                    assert_eq!(r.cpu.word(missile(0) + 0x28) != 0, locked);
                }
                for (row, i) in poses.iter().zip(active) {
                    for j in 0..3 {
                        assert_eq!(
                            row[1 + j],
                            r.cpu.word(cs(i) + 0x5c + 4 * j as u32) as i32,
                            "{vehicle:?} {tick}"
                        );
                    }
                    let mat = (cs(i) as usize & 0x1fffff) + 0x14;
                    for j in 0..9 {
                        assert_eq!(
                            row[4 + j],
                            i16::from_le_bytes(
                                r.cpu.ram[mat + 2 * j..mat + 2 * j + 2].try_into().unwrap()
                            ) as i32,
                            "{vehicle:?} {tick}"
                        );
                    }
                }
                let mut faces = [[0; 14]; 180];
                let count = r.scene.contrail_faces(&mut faces);
                let mut expected = Vec::new();
                for i in 0..10 {
                    let m = missile(i);
                    if r.cpu.word(MISSILE_AUX + 0x110 * i + 8) == 0 {
                        continue;
                    }
                    let half = |addr: u32| {
                        let a = addr as usize & 0x1fffff;
                        i16::from_le_bytes(r.cpu.ram[a..a + 2].try_into().unwrap()) as i32
                    };
                    for j in 0..18 {
                        let poly = m + 0x160 + 36 * j;
                        if half(poly + 4) == half(poly + 6) {
                            continue;
                        }
                        let mut face = [0; 14];
                        face[0] = (r.cpu.word(poly + 0x14) & 255) as i32;
                        let rgb = r.cpu.word(poly + 0x10);
                        face[1] =
                            (((rgb & 255) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 255)) as i32;
                        for v in 0..4 {
                            let vi = half(poly + 4 + 2 * v) as u32;
                            for k in 0..3 {
                                face[(2 + 3 * v + k) as usize] = r.cpu.word(m + 0x18 + 4 * k)
                                    as i32
                                    + half(m + 0x58 + 8 * vi + 2 * k);
                            }
                        }
                        expected.push(face);
                    }
                }
                faces[..count].sort_unstable();
                expected.sort_unstable();
                assert_eq!(faces[..count], expected, "{vehicle:?} {tick} trail");
                r.compare();
                r.cpu.call(0x80110810, &[]);
                r.cpu.call(0x80137d2c, &[]);
                r.scene.phase(&mut r.player, &mut r.police, 2, 3);
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn regular_missile_impacts_original() {
    for (slot, damage) in [7, 0, 15, 25, 3, 7].into_iter().enumerate() {
        let mut r = Reference::new();
        let m = 0x801a9c10;
        let cs = 0x800be400;
        put(&mut r.cpu, m + 0x24, cs);
        put(&mut r.cpu, cs + 0xc0, 200);
        for at in [
            0x80107c6c, 0x80107d18, 0x80107dbc, 0x8010e6e8, 0x8010e738, 0x8010e924, 0x8010ee7c,
            0x80136a28, 0x80136c00, 0x80137140,
        ] {
            r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        vec3(&mut r.player.state, 0x388, [2700, 2200, 400]);
        vec3(&mut r.player.state, 0x3a0, [0, 0, 0]);
        vec3(
            &mut r.police.state,
            0x444,
            [2700, if slot == 5 { 1400 } else { 3000 }, 400],
        );
        r.player.state[0xd2] = 1;
        r.player.state[0x460..0x464].copy_from_slice(&(slot as i32).to_le_bytes());
        let ammo = 0x464 + 2 * slot;
        r.player.state[ammo..ammo + 2].copy_from_slice(&2i16.to_le_bytes());
        r.sync();
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        r.cpu.call(0x8013148c, &[PLAYER, 1]);
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        assert_eq!(r.player.weapon().1, 1);
        let before = r.police.health().0;
        let mut hits = 0;
        for _ in 0..30 {
            if r.cpu.word(m + 0x34) as i32 <= 0 {
                break;
            }
            let mut expected = Vec::new();
            let mut source = [0; 3];
            r.cpu.call_observed(0x80110210, &[m], 0x80000080, |cpu| {
                let position = |p| [0, 4, 8].map(|d| cpu.word(p + d) as i32);
                let kind = match cpu.pc() {
                    0x80136a28 => 28,
                    0x80136aa0 => 55,
                    0x80136c00 => 56,
                    0x80137140 => 8,
                    0x80137190 => 31,
                    0x8010461c => {
                        source = position(cpu.r[4]);
                        return;
                    }
                    0x80107dbc => {
                        expected.push(BattleEvent::WeaponImpact {
                            weapon: cpu.r[4] as u8,
                            position: source,
                        });
                        return;
                    }
                    _ => return,
                };
                expected.push(BattleEvent::Effect {
                    kind,
                    position: position(cpu.r[4]),
                });
            });
            r.cpu.call(0x80137d2c, &[]);
            r.scene.phase(&mut r.player, &mut r.police, 2, 3);
            r.compare();
            let events: Vec<_> = r
                .scene
                .audio
                .iter()
                .copied()
                .filter(|event| {
                    matches!(
                        event,
                        BattleEvent::WeaponImpact { .. }
                            | BattleEvent::Effect {
                                kind: 28 | 55 | 56 | 8 | 31,
                                ..
                            }
                    )
                })
                .collect();
            assert_eq!(events, expected, "slot {slot} tick {hits}");
            let mut fx = [[0; 5]; 122];
            let count = r.scene.effects(&mut fx);
            let missile = fx[..count].iter().find(|v| v[3] == 3);
            if r.cpu.word(m + 0x34) as i32 > 0 {
                let v = missile.unwrap();
                for i in 0..3 {
                    assert_eq!(v[i], r.cpu.word(cs + 0x5c + 4 * i as u32) as i32);
                }
            } else {
                assert!(missile.is_none());
                hits += 1;
            }
        }
        assert_eq!(hits, 1);
        assert_eq!(r.police.health().0, before - damage, "slot {slot}");
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn dropped_weapon_contacts_original() {
    for weapon in [6u8, 8, 9, 10] {
        let mut r = Reference::new();
        let d = 0x801996b8;
        vec3(&mut r.player.state, 0x388, [2700, 2200, 400]);
        r.player.state[0xd2] = 1;
        r.player.state[0x460..0x464].copy_from_slice(&i32::from(weapon).to_le_bytes());
        let ammo = 0x464 + 2 * usize::from(weapon);
        r.player.state[ammo..ammo + 2].copy_from_slice(&2i16.to_le_bytes());
        r.sync();
        for at in [0x80107c6c, 0x80107d18, 0x80107dbc, 0x80136c78, 0x80137140] {
            r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        r.cpu.call(0x80113858, &[PLAYER, 1]);
        r.cpu.call(0x8013148c, &[PLAYER, 1]);
        r.scene.phase(&mut r.player, &mut r.police, 1, 3);
        r.compare();
        assert_eq!(r.cpu.word(d), u32::from(weapon));
        let mut pos = [8, 12, 16].map(|k| r.cpu.word(d + k) as i32);
        assert!(r.scene.audio.contains(&BattleEvent::DropLaunch {
            weapon,
            position: pos
        }));
        let floor = r.cpu.word(d + 20) as i32;
        pos[2] = if pos[2] > floor {
            (pos[2] - 12).max(floor)
        } else {
            pos[2]
        };
        vec3(&mut r.police.state, 0x444, [pos[0], pos[1], pos[2] + 24]);
        r.sync();
        put(&mut r.cpu, d + 16, pos[2] as u32);
        r.cpu.put(d + 4, &399i16.to_le_bytes());
        let mut expected = Vec::new();
        r.cpu.call_observed(0x80133fd4, &[0], 0x80000080, |cpu| {
            let kind = match cpu.pc() {
                0x80107dbc => {
                    expected.push(BattleEvent::WeaponImpact {
                        weapon: cpu.r[4] as u8,
                        position: pos,
                    });
                    return;
                }
                0x80136c78 => 37,
                0x80137140 => 8,
                0x8013721c => 32,
                _ => return,
            };
            expected.push(BattleEvent::Effect {
                kind,
                position: [0, 4, 8].map(|k| cpu.word(cpu.r[4] + k) as i32),
            });
        });
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        assert!(
            matches!(expected.first(), Some(BattleEvent::WeaponImpact { .. })),
            "weapon {weapon} at {pos:?}"
        );
        assert_eq!(r.scene.audio, expected, "weapon {weapon}");
        r.compare();
        let mut fx = [[0; 5]; 122];
        let n = r.scene.effects(&mut fx);
        assert_eq!(fx[..n].iter().any(|e| e[3] == 6), weapon == 9);
        let mut drawn = [[0; 4]; 12];
        assert_eq!(r.scene.drawn_drops(&mut drawn), 1);
        assert_eq!(drawn[0], [pos[0], pos[1], pos[2], i32::from(weapon)]);
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        assert_eq!(r.scene.drawn_drops(&mut drawn), usize::from(weapon == 9));
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn coupled_arena_original() {
    compare_encounter(Vehicle::YellowJacket, 20, Level::Arena);
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn hammerhead_crush_original() {
    compare_encounter(Vehicle::Hammerhead, 20, Level::Arena);
    compare_encounter(Vehicle::Hammerhead, 5, Level::Arena);
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn warehouse_encounter_original() {
    for player in Vehicle::ROSTER {
        println!("Warehouse retail encounter with {player:?}");
        compare_encounter(player, 20, Level::Warehouse);
    }
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn temporary_pothole_matches_retail_clock() {
    let mut r = Reference::on_level(Vehicle::YellowJacket, false, Level::CityPark);
    r.cpu.call(0x80130938, &[]);
    unsafe extern "C" {
        fn MakeFakePotHole() -> i16;
        static numCheckPotHoles: i16;
    }
    let bind = |scene: &CollisionScene, create: bool| {
        let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_level(Level::CityPark as i32);
            tm1_use_scene(scene.native.as_ptr());
            if create {
                MakeFakePotHole();
            }
            let count = numCheckPotHoles;
            tm1_use_scene(std::ptr::null_mut());
            count
        }
    };
    // A second impact replaces the temporary slot while its timer keeps running.
    for tick in 1..=120 {
        if [1, 15, 70].contains(&tick) {
            bind(&r.scene, true);
            r.cpu.call(0x80120704, &[]);
        }
        let fields = [2, 3, 5][tick % 3];
        r.cpu.put(0x80170238, &(fields as u16).to_le_bytes());
        r.cpu.call(0x80120728, &[]);
        r.scene.step_bout(
            &mut r.player,
            std::slice::from_mut(&mut r.police),
            Input::default(),
            fields,
        );
        let count = r.cpu.word(0x8018bfdc) >> 16;
        assert_eq!(bind(&r.scene, false) as u32, count, "tick {tick}");
        if tick == 31 || tick == 110 {
            assert_eq!(count, 1);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn city_park_slick_drive_matches_retail() {
    let mut r = Reference::on_level(Vehicle::YellowJacket, false, Level::CityPark);
    // Approach the south edge of the rectangular slick along its long axis.
    vec3(&mut r.player.state, 0x388, [6300, -1400, 0]);
    r.sync();
    put(&mut r.cpu, 0x8018c3b4, Level::CityPark as u32);
    for entry in [0x80127694, 0x8012da2c, 0x80130938, 0x80130c20, 0x8013131c] {
        r.cpu.call(entry, &[]);
    }
    for entry in [0x80108378, 0x801083b0, 0x801417d0, 0x801466b0, 0x80146930] {
        r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    let mut entered = false;
    let mut left = false;
    for step in 1..=150 {
        let input = Input {
            accelerate: true,
            left: (30..36).contains(&step),
            ..Default::default()
        };
        let fields = [3u8, 4, 5][step % 3];
        compare_drive_step(&mut r, step, input, fields);
        let slick = r.player.state[0xa0 + 13] != 0;
        if entered && !slick {
            left = true;
        }
        entered |= slick;
    }
    assert!(
        entered && left,
        "expected to cross slick, ended at {:?}",
        r.player.position()
    );
}

fn compare_drive_step(r: &mut Reference, step: usize, input: Input, fields: u8) {
    r.cpu.call(0x800ffb98, &[2, u32::from(input.ps1_buttons())]);
    r.cpu.put(0x80170238, &(fields as u16).to_le_bytes());
    r.cpu
        .put(0x8017023a, &(60u16 / fields as u16).to_le_bytes());
    put(&mut r.cpu, 0x8017023c, step as u32);
    r.cpu.call(0x801125ac, &[PLAYER]);
    r.cpu.call_until(0x8011125c, &[PLAYER], 0x801115c0);
    r.player.step(input, fields);
    let start = (PLAYER & 0x1fffff) as usize;
    assert_eq!(
        &r.player.state[..],
        &r.cpu.ram[start..start + r.player.state.len()],
        "drive step {step}"
    );
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftop_edge_fall_matches_retail() {
    let mut r = Reference::on_level(Vehicle::YellowJacket, false, Level::Rooftops);
    vec3(&mut r.player.state, 0x388, [15400, 1800, 7760]);
    vec3(&mut r.player.state, 0x3a0, [0, 0, 0]);
    vec3(&mut r.player.state, 0x3b8, [0, 0, 0]);
    r.sync();
    put(&mut r.cpu, 0x8018c3b4, Level::Rooftops as u32);
    put(&mut r.cpu, 0x8018c34c, 1);
    for entry in [0x8012b5a4, 0x8013026c, 0x80130a7c, 0x801312ec, 0x80131368] {
        r.cpu.call(entry, &[]);
    }
    for entry in [0x80108378, 0x801083b0, 0x801417d0, 0x801466b0, 0x80146930] {
        r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    let mut airborne = false;
    let mut camera = VehicleCamera::new(&r.player, CarView::Chase);
    let mut anchor = None;
    for step in 1..=300 {
        compare_drive_step(
            &mut r,
            step,
            Input {
                accelerate: true,
                ..Default::default()
            },
            3,
        );
        let h = r.player.position()[2];
        camera.update(&r.player, false);
        assert_eq!(camera.fixed_pose.is_some(), h < 6800);
        if let Some([x, y, z, _, _]) = camera.fixed_pose {
            assert_eq!(*anchor.get_or_insert([x, y, z]), [x, y, z]);
        }
        airborne |= h < 7000 && h >= 160 && r.player.health().0 > 0;
        if r.player.health().0 <= 0 {
            assert!(airborne && h < 160);
            println!("Rooftops player fell from 7760 to {h} and died at step {step}");
            return;
        }
    }
    panic!("expected a fatal fall, ended at {:?}", r.player.position());
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn cyburbia_visibility_and_ground_groups_match_retail() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    let mut samples = 0;
    for db in ["SUBURB1", "SUBURB2"] {
        let data = disc.read(&format!("UADMD/{db}.DMD")).unwrap();
        let scene = tm1_assets::scene::Dmd::parse(&data)
            .unwrap()
            .extract_kind(1)
            .unwrap();
        cpu.put(0x800188b8, &data);
        assert!(!scene.visibility.is_empty());
        for controller in scene.visibility {
            let count = cpu.ram[(controller.node as usize & 0x1fffff) + 10];
            let boxes: Vec<_> = (0..count)
                .map(|i| cpu.word(controller.node + 20 + 4 * u32::from(i)))
                .collect();
            let mut outcomes = [false; 2];
            for &b in &boxes {
                let half = |at: u32| {
                    let i = (at & 0x1fffff) as usize;
                    i32::from(i16::from_le_bytes(cpu.ram[i..i + 2].try_into().unwrap()))
                };
                let lo = [32, 34, 36].map(|o| half(b + o));
                let size = [40, 42, 44].map(|o| half(b + o));
                let basis = [0, 1, 2].map(|row| [0, 1, 2].map(|col| half(b + 2 * (row * 3 + col))));
                for axis in 0..3 {
                    for distance in [
                        -1,
                        0,
                        1,
                        size[axis] / 2,
                        size[axis] - 1,
                        size[axis],
                        size[axis] + 1,
                    ] {
                        let mut local = [0, 1, 2].map(|j| size[j] / 2 - lo[j]);
                        local[axis] = distance - lo[axis];
                        let eye = [0, 1, 2]
                            .map(|j| (0..3).map(|k| basis[k][j] * local[k]).sum::<i32>() / 4096);
                        for (j, v) in eye.iter().enumerate() {
                            put(&mut cpu, 0x80011800 + 4 * j as u32, (-v) as u32);
                        }
                        let inside = boxes
                            .iter()
                            .any(|&b| cpu.call(0x800fe388, &[b, 0x80011800]) != 0);
                        assert_eq!(
                            controller.camera_inside(eye.map(|v| v as f32)),
                            inside,
                            "{db} controller {:x} eye {eye:?}",
                            controller.node
                        );
                        outcomes[usize::from(inside)] = true;
                        samples += 1;
                    }
                }
            }
            assert_eq!(
                outcomes,
                [true, true],
                "{db} controller {:x}",
                controller.node
            );
        }
    }
    cpu.call(0x801281b8, &[]);
    put(&mut cpu, 0x8018c34c, 1);
    unsafe extern "C" {
        fn GetGroupTestPointIsIn(x: i32, z: i32, y: i32) -> i16;
    }
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        tm1_bind_level(Level::Cyburbia as i32);
    }
    let groups = 0x801ac35c;
    for i in 0..12 {
        let [x, z, y] = [0, 4, 8].map(|offset| cpu.word(groups + 0x49c + i * 12 + offset) as i32);
        let w = cpu.word(groups + 0x544 + 4 * i) as i32;
        let d = cpu.word(groups + 0x57c + 4 * i) as i32;
        for px in [x - 1, x, x + w / 2, x + w, x + w + 1] {
            for pz in [z - 1, z, z + d / 2, z + d, z + d + 1] {
                for py in [y - 81, y - 80, y - 79] {
                    assert_eq!(
                        unsafe { GetGroupTestPointIsIn(px, pz, py) } as u32,
                        cpu.call(0x8011ce9c, &[px as u32, pz as u32, py as u32])
                    );
                }
            }
        }
    }
    println!("Compared {samples} Cyburbia camera probes and 900 ground probes with retail.");
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn minion_initial_record_matches_retail() {
    let mut r = Reference::on_level(Vehicle::YellowJacket, false, Level::Rooftops);
    let slot = 3;
    let addr = POLICE + 0x560 * slot;
    let cs = 0x80010200;
    put(&mut r.cpu, 0x8018c3b4, Level::Rooftops as u32);
    put(&mut r.cpu, 0x8019f788 + 4 * slot, cs);
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        tm1_bind_level(Level::Rooftops as i32);
    }
    for initial_seed in [1, 0x12345678, 0xfffffff0] {
        let mut state = [0u8; 0x560];
        state[6..8].copy_from_slice(&(slot as u16).to_le_bytes());
        r.cpu.put(addr, &state);
        r.cpu.call(0x8012b5a4, &[]);
        r.cpu.random_state = initial_seed;
        r.cpu.call(0x80124d34, &[addr, 130, 1]);
        r.cpu.call(0x80142064, &[addr]);
        let mut seed = initial_seed;
        let mut used = 0;
        unsafe {
            tm1_ai_step(
                std::ptr::null_mut(),
                state.as_mut_ptr(),
                r.player.state.as_mut_ptr(),
                r.player.trig.as_ptr(),
                &mut seed,
                130,
                &mut used,
                3,
                0,
            );
        }
        let start = (addr & 0x1fffff) as usize;
        for (i, v) in state.into_iter().enumerate() {
            assert_eq!(v, r.cpu.ram[start + i], "Minion byte {i:x}");
        }
        assert_eq!(seed, r.cpu.random_state);

        // Slot initialization was checked above. Execute the remaining retail
        // transition including CarInit for a player who has fallen below the roof.
        r.cpu.put(0x8014d838, &[8, 0, 224, 3, 0, 0, 0, 0]);
        // PadGetConfig returns the supported keyboard mapping's PS1 pad profile.
        r.cpu.put(0x801502d4, &[8, 0, 224, 3, 1, 0, 2, 36]);
        for (group, height) in [(5i16, 6400i32), (6, 6399), (12, 8000)] {
            let mut boss = state;
            let mut player = r.player.state;
            player[0xc0..0xc2].copy_from_slice(&group.to_le_bytes());
            player[0x390..0x394].copy_from_slice(&height.to_le_bytes());
            player[0x394..0x3a0].fill(71);
            player[0x3ac..0x3b8].fill(93);
            player[0x464..0x466].copy_from_slice(&13i16.to_le_bytes());
            player[0x47a..0x47c].copy_from_slice(&2i16.to_le_bytes());
            player[0x48e..0x490].copy_from_slice(&120i16.to_le_bytes());
            player[0x178..0x17c].copy_from_slice(&25i32.to_le_bytes());
            let fov = if group == 6 && seed & 1 == 0 { 55 } else { 60 };
            put(&mut r.cpu, 0x8018bf9c, fov);
            let mut placement_seed = seed;
            r.cpu.random_state = seed;
            r.cpu.put(addr, &boss);
            r.cpu.put(PLAYER, &player);

            r.cpu.call(0x80150d04, &[]);
            unsafe {
                tm1_place_boss(
                    boss.as_mut_ptr(),
                    player.as_mut_ptr(),
                    r.player.trig.as_ptr(),
                    &mut placement_seed,
                    fov as i32,
                )
            };
            assert_eq!(placement_seed, r.cpu.random_state);
            assert_eq!(boss, r.cpu.ram[start..start + boss.len()]);
            let p = (PLAYER & 0x1fffff) as usize;
            for (i, b) in player.iter().enumerate() {
                assert_eq!(*b, r.cpu.ram[p + i], "player byte {i:x}, height {height}");
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn campaign_player_carryover_matches_retail() {
    let mut r = Reference::new();
    let mut player = r.player;
    r.cpu.put(0x801502d4, &[8, 0, 224, 3, 1, 0, 2, 36]);
    put(&mut r.cpu, 0x8018bf9c, 60);
    while let Some(level) = player.level().next() {
        player.state[0x178..0x17c].copy_from_slice(&17i32.to_le_bytes());
        player.state[0x460..0x464].copy_from_slice(&2u32.to_le_bytes());
        player.state[0x468..0x46a].copy_from_slice(&9i16.to_le_bytes());
        player.state[0x47a..0x47c].copy_from_slice(&1i16.to_le_bytes());
        player.state[0x48e..0x490].copy_from_slice(&110i16.to_le_bytes());
        let points = match level {
            Level::Warehouse => 0x801262ec,
            Level::Freeway => 0x801268f8,
            Level::CityPark => 0x80127694,
            Level::Cyburbia => 0x801281b8,
            Level::Rooftops => 0x8012b5a4,
            Level::Arena | Level::Arena5 => unreachable!(),
        };
        r.cpu.call(points, &[]);
        put(&mut r.cpu, 0x8018c3b4, level as u32);
        r.cpu.put(PLAYER, &player.state);
        r.cpu.random_state = player.seed;
        r.cpu.call(0x8012287c, &[PLAYER, player.vehicle as u32, 0]);
        player = player.for_next_stage(level);
        let base = (PLAYER & 0x1fffff) as usize;
        for (i, v) in player.state.iter().enumerate() {
            assert_eq!(*v, r.cpu.ram[base + i], "{level:?} player offset {i:x}");
        }
        assert_eq!(player.seed, r.cpu.random_state);
        let initial = player.state;
        player.state.fill(0);
        player.reset();
        assert_eq!(player.state, initial);
        player.state[0x178..0x17c].copy_from_slice(&(-5i32).to_le_bytes());
        r.cpu.put(PLAYER, &player.state);
        r.cpu.random_state = player.seed;
        r.cpu.call(0x8012287c, &[PLAYER, player.vehicle as u32, 1]);
        r.scene.respawn_player(&mut player);
        for (i, v) in player.state.iter().enumerate() {
            assert_eq!(*v, r.cpu.ram[base + i], "{level:?} new life offset {i:x}");
        }
        assert_eq!(player.seed, r.cpu.random_state);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftops_encounter_original() {
    for player in Vehicle::ROSTER {
        println!("Rooftops retail encounter with {player:?}");
        compare_encounter(player, 20, Level::Rooftops);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn cyburbia_encounter_original() {
    for player in Vehicle::ROSTER {
        println!("Cyburbia retail encounter with {player:?}");
        compare_encounter(player, 20, Level::Cyburbia);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn city_park_encounter_original() {
    for player in Vehicle::ROSTER {
        println!("City Park retail encounter with {player:?}");
        compare_encounter(player, 20, Level::CityPark);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn freeway_encounter_original() {
    for player in Vehicle::ROSTER {
        println!("Freeway retail encounter with {player:?}");
        compare_encounter(player, 20, Level::Freeway);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn database_and_opponents_match_retail_shell() {
    let mut r = Reference::new();
    for (level, masks) in [
        (Level::Warehouse, 0x8017179cusize),
        (Level::Freeway, 0x801717b4),
        (Level::CityPark, 0x801717cc),
        (Level::Cyburbia, 0x801717e4),
        (Level::Rooftops, 0x801717fc),
    ] {
        put(&mut r.cpu, 0x8018c3b4, level as u32);
        let start = (r.cpu.word(0x8017187c + 4 * level as u32) & 0x1fffff) as usize;
        for (i, vehicle) in Vehicle::CARD_ORDER.into_iter().enumerate() {
            r.cpu.call(0x80152518, &[vehicle as u32]);
            let db = level.database(vehicle);
            assert_eq!(&r.cpu.ram[start..start + db.len()], db.as_bytes());
            let ids = level.opponents(vehicle, &mut ArenaDeck::default(), &mut 1);
            let mask = Vehicle::CARD_ORDER
                .iter()
                .enumerate()
                .fold(0u16, |mask, (i, v)| {
                    mask | (u16::from(ids.contains(v)) << i)
                });
            let at = (masks & 0x1fffff) + 2 * i;
            assert_eq!(mask.to_le_bytes(), r.cpu.ram[at..at + 2]);
        }
    }
}

fn register_retail_stands(cpu: &mut Cpu, data: &[u8]) {
    for at in [0x80147508, 0x8010844c] {
        cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    cpu.call(0x80135d7c, &[]);
    let world = tm1_assets::scene::Dmd::parse(data)
        .unwrap()
        .extract_kind(1)
        .unwrap();
    for (slot, pos) in world.health_stands {
        for (axis, value) in pos.into_iter().enumerate() {
            put(cpu, 0x80011000 + axis as u32 * 4, value as u32);
        }
        cpu.call(0x80135da8, &[u32::from(slot) + 1, 0x80011000]);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn warehouse_stands_match_retail() {
    let mut r = Reference::on_level(Vehicle::YellowJacket, true, Level::Warehouse);
    r.sync();
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    register_retail_stands(&mut r.cpu, &disc.read("UADMD/WH1.DMD").unwrap());
    put(&mut r.cpu, 0x8018c080, 4000);
    unsafe extern "C" {
        fn CheckHealthStands(car: *mut u8, is_player: i32);
        fn tm1_recharge_player(car: *mut u8) -> u8;
        fn tm1_regenerate_stands();
    }
    #[repr(align(4))]
    struct Record([u8; 0x4a4]);
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { tm1_use_scene(r.scene.native.as_ptr()) };
    struct Clear;
    impl Drop for Clear {
        fn drop(&mut self) {
            unsafe { tm1_use_scene(std::ptr::null_mut()) }
        }
    }
    let _clear = Clear;
    assert_eq!(r.scene.health_stands().0, 3);
    for id in 1..=2 {
        let mut car = Record(r.player.state);
        car.0[0x270 + 14] = id;
        car.0[0x178..0x17c].copy_from_slice(&25i32.to_le_bytes());
        r.cpu.put(PLAYER, &car.0);
        r.cpu.call(0x8011f038, &[PLAYER, 1]);
        unsafe { CheckHealthStands(car.0.as_mut_ptr(), 1) };
        assert_eq!(car.0[1], id);
        let start = (PLAYER & 0x1fffff) as usize;
        for tick in 0..=500 {
            assert_eq!(&car.0[..], &r.cpu.ram[start..start + car.0.len()]);
            let retail = r.cpu.call(0x8014e20c, &[0]);
            let native = unsafe { tm1_recharge_player(car.0.as_mut_ptr()) };
            assert_eq!(u32::from(native), retail);
            if native == 0 {
                assert!(tick > 0);
                assert_eq!(&car.0[..], &r.cpu.ram[start..start + car.0.len()]);
                break;
            }
        }
        assert_eq!(r.scene.health_stands().0 & (1 << (id - 1)), 0);
    }
    for tick in 1..=4002 {
        unsafe { tm1_regenerate_stands() };
        r.cpu.call(0x80135e10, &[]);
        let retail = r.cpu.call(0x80135c74, &[1]) | (r.cpu.call(0x80135c74, &[2]) << 1);
        assert_eq!(u32::from(r.scene.health_stands().0), retail, "tick {tick}");
        assert_eq!(retail, if tick == 4002 { 3 } else { 0 });
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn warehouse_repair_holds_view_until_the_release_update() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/WH1.DMD").unwrap();
    for cap in [80i32, 100] {
        let mut r = Reference::on_level(Vehicle::YellowJacket, true, Level::Warehouse);
        r.player.state[1] = 1;
        r.player.state[0xb6] = 1;
        for (at, value) in [(0x178, cap - 4), (0x17c, 100), (0x18c, 2), (0x190, cap)] {
            r.player.state[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        r.sync();
        register_retail_stands(&mut r.cpu, &data);
        let mut camera = setup_chase_camera(&mut r);
        // Enter mid-transition so an extra CarViewUpdate is visible in XYZ and pitch.
        camera.offset = [80, -600, 210];
        camera.pitch = 90;
        vec3_cpu(&mut r.cpu, PLAYER + 0xc, camera.offset);
        put(&mut r.cpu, PLAYER + 4, camera.pitch as u32);
        for tick in 1..=3 {
            let held_at_stand = r.cpu.call(0x8014e20c, &[0]) != 0;
            let opponents = std::slice::from_mut(&mut r.police);
            let step = r.scene.begin_bout_step(&mut r.player, opponents, 3);
            assert_eq!(step.updates_player(), !held_at_stand);
            assert_eq!(step.updates_player(), tick == 3);
            assert_eq!(r.player.health().0 as u32, r.cpu.word(PLAYER + 0x178));
            if !held_at_stand {
                r.cpu.put(PLAYER + 0xda, &[1]);
                r.cpu.call_until(0x8014eff4, &[PLAYER], 0x8014f278);
            }
            if step.updates_player() {
                camera.cycle(true);
                r.player
                    .set_battle_view(PlayerView::Car(camera.mode), false);
            }
            let updated = step.updates_player();
            r.scene.finish_bout_step(&mut r.player, opponents, 0, step);
            camera.after_step(&r.player, updated);
            if !held_at_stand {
                r.cpu.put(PLAYER + 0x138, &r.player.state[0x138..]);
                r.cpu.call(0x80116bcc, &[PLAYER]); // CarViewUpdate
            }
            let offset = std::array::from_fn(|i| r.cpu.word(PLAYER + 0xc + 4 * i as u32) as i32);
            assert_eq!(camera.offset, offset);
            assert_eq!(camera.pitch, r.cpu.word(PLAYER + 4) as i16 as i32);
            assert_eq!(camera.mode as u32, r.cpu.word(PLAYER + 0xf4));
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_repair_only_holds_the_occupants_camera() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let level = Level::Warehouse;
    let data = disc
        .read(&format!("UA2PLAY/{}.DMD", level.duel_database().unwrap()))
        .unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let kinds = [Vehicle::YellowJacket, Vehicle::Outlaw];
    let dmd = tm1_assets::scene::Dmd::parse(&data).unwrap();
    let root = |kind| dmd.roots.iter().find(|r| r.1 == kind).unwrap().0;
    for slot in 0..2 {
        let mut scene =
            CollisionScene::for_duel(&data, root(1), kinds.map(|v| root(v as u16))).unwrap();
        let mut cars =
            PlayerVehicle::duel_players(&exe, kinds, level, 1, Difficulty::Medium).unwrap();
        let car = &mut cars[slot];
        car.state[1] = 1;
        car.state[0xb6] = 1;
        let cap = car.health().1.min(car.word(0x190));
        car.state[0x178..0x17c].copy_from_slice(&(cap - 2).to_le_bytes());
        car.state[0x18c..0x190].copy_from_slice(&2i32.to_le_bytes());
        for tick in 1..=2 {
            scene.step_duel(&mut cars, [Input::default(); 2], 3);
            let updated = scene.duel_updated_players();
            assert!(updated[1 - slot]);
            assert_eq!(updated[slot], tick == 2);
        }
    }
}

fn compare_encounter(vehicle: Vehicle, charge: i16, level: Level) {
    compare_battle(vehicle, charge, level, false);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn minion_encounter_original() {
    for vehicle in Vehicle::ROSTER {
        compare_battle(vehicle, 20, Level::Rooftops, true);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn bonus_arena_original() {
    for vehicle in Vehicle::ROSTER {
        compare_encounter(vehicle, 20, Level::Arena5);
    }
}

fn start_retail_death(cpu: &mut Cpu, car: u32, player: bool) {
    let flags = car + if player { 0xa0 } else { 0x66 };
    let health = car + if player { 0x178 } else { 0x24c };
    if cpu.word(health) as i32 <= 0 && cpu.ram[(flags & 0x1fffff) as usize] == 0 {
        cpu.put(flags, &[1]);
        cpu.call(0x8014e8dc, &[car, u32::from(player)]);
    }
}

fn compare_battle(vehicle: Vehicle, charge: i16, level: Level, boss: bool) {
    compare_battle_skill(vehicle, charge, level, boss, Difficulty::Medium);
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn difficulty_encounters_original() {
    for difficulty in [Difficulty::Easy, Difficulty::Hard] {
        compare_battle_skill(Vehicle::YellowJacket, 20, Level::Arena, false, difficulty);
        compare_battle_skill(Vehicle::Outlaw, 20, Level::Warehouse, false, difficulty);
        compare_battle_skill(Vehicle::RoadKill, 20, Level::Rooftops, true, difficulty);
    }
}
fn compare_battle_skill(
    vehicle: Vehicle,
    charge: i16,
    level: Level,
    boss: bool,
    difficulty: Difficulty,
) {
    compare_battle_controls(vehicle, charge, level, boss, difficulty, false);
}
#[test]
#[ignore = "requires the supported TM1_CUE"]
fn god_mode_pad_sequences_original() {
    compare_battle_controls(
        Vehicle::YellowJacket,
        20,
        Level::Warehouse,
        false,
        Difficulty::Medium,
        true,
    );
}
fn compare_battle_controls(
    vehicle: Vehicle,
    charge: i16,
    level: Level,
    boss: bool,
    difficulty: Difficulty,
    cheats: bool,
) {
    let mut r = Reference::on_difficulty(vehicle, true, level, difficulty);
    r.cpu.call(0x80137268, &[]);
    let crush = level == Level::Arena && vehicle == Vehicle::Hammerhead;
    let mut engaged = false;
    put(&mut r.cpu, 0x8019edd8, vehicle as u32);
    put(&mut r.cpu, 0x8019eddc, r.police.vehicle as u32);
    if crush {
        r.player.state[0x47a..0x47c].copy_from_slice(&charge.to_le_bytes());
        let [x, y, _] = r.player.position();
        vec3(&mut r.police.state, 0x444, [x + 380, y + 380, 16]);
        r.police.state[0x66 + 16] = 1;
        r.police.state[0x20c + 0x18..0x20c + 0x1a].copy_from_slice(&1000i16.to_le_bytes());
    }
    r.police.state[0x40] = 0;
    for entry in match level {
        Level::Arena | Level::Arena5 => {
            [0x80125bc0, 0x8012cd90, 0x80130680, 0x80130a8c, 0x801312f8]
        }
        Level::Rooftops => [0x8012b5a4, 0x8013026c, 0x80130a7c, 0x801312ec, 0x80131368],
        Level::Cyburbia => [0x801281b8, 0x8012ed70, 0x801309c8, 0x80130ee4, 0x8013135c],
        Level::CityPark => [0x80127694, 0x8012da2c, 0x80130938, 0x80130c20, 0x8013131c],
        Level::Freeway => [0x801268f8, 0x8012d3e0, 0x80130884, 0x80130c14, 0x80131310],
        Level::Warehouse => [0x801262ec, 0x8012cf58, 0x801307a4, 0x80130a98, 0x80131304],
    } {
        r.cpu.call(entry, &[]);
    }
    for at in [
        0x80146930, 0x801073cc, 0x80108604, 0x80116bcc, 0x80107f84, 0x801371cc, 0x80137190,
        0x8014eff4, 0x801370d0, 0x80108378, 0x801083b0,
    ] {
        r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    r.cpu.put(0x8018c378, &(level as u16 + 1).to_le_bytes());
    r.cpu.put(
        0x8018c34c,
        &[u8::from(matches!(level, Level::Cyburbia | Level::Rooftops))],
    );
    r.cpu.put(
        0x8018c374,
        &[u8::from(!matches!(
            level,
            Level::Arena | Level::Arena5 | Level::Rooftops
        ))],
    );
    put(&mut r.cpu, 0x8018c3b4, level as u32);
    r.cpu.put(PLAYER + 0x388, &[0xff; 0x24]);
    r.cpu.call(0x80122ea4, &[PLAYER, 1]);
    for (axis, position) in r.player.position().into_iter().enumerate() {
        assert_eq!(
            r.cpu.word(PLAYER + 0x388 + 4 * axis as u32) as i32,
            position
        );
    }
    assert_eq!(r.cpu.word(PLAYER + 0x3a8) as i32, r.player.heading());
    // RT draws drops and tasers between missile and bullet movement. The
    // fixture has no packet space, but those passes still advance game state.
    for at in [
        0x80107c6c, 0x80107d18, 0x80107dbc, 0x8010e6e8, 0x8010e738, 0x8010e924, 0x8010ee7c,
        0x80136a28, 0x80136c00, 0x80137140,
    ] {
        r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    for i in 0..10 {
        let m = 0x801a9c10 + 0x3e8 * i;
        put(&mut r.cpu, m + 0x34, (-12i32) as u32);
        put(&mut r.cpu, m + 0x24, MISSILE_CS + 0x110 * i);
        put(&mut r.cpu, m + 0x100, MISSILE_AUX + 0x110 * i);
        r.cpu.put(m + 0x30, &[i as u8]);
        put(&mut r.cpu, MISSILE_CS + 0x110 * i + 0xc0, 200 + i);
    }
    for i in 0..50 {
        let node = r.cpu.word(0x8019ce2c + i * 4);
        if node != 0 {
            let kind = r.cpu.word(node) >> 16;
            r.cpu.call(0x801356f4, &[node + 4, kind, i + 1]);
        }
    }
    r.sync();
    let mut lineup = Vec::new();
    let mut slots = vec![POLICE_CS];
    if level != Level::Arena {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc
            .read(&format!("UADMD/{}.DMD", level.database(vehicle)))
            .unwrap();
        register_retail_stands(&mut r.cpu, &data);
        let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
        let mut kinds = level.opponents(vehicle, &mut ArenaDeck::default(), &mut 1);
        r.player.reset();
        lineup = Opponent::spawn_bout(&mut r.player, &kinds);
        if boss {
            for rival in &mut lineup {
                rival.state[0x24c..0x250].copy_from_slice(&0i32.to_le_bytes());
            }
            kinds.push(Vehicle::Minion);
            lineup.push(Opponent::spawn_minion(&mut r.player));
            r.cpu.put(PLAYER, &r.player.state);
        }
        let root = |v| roots.iter().find(|r| r.1 == v).unwrap().0;
        r.scene = CollisionScene::for_opponents(
            &data,
            root(1),
            root(vehicle as u16),
            &kinds.iter().map(|v| root(*v as u16)).collect::<Vec<_>>(),
        )
        .unwrap();
        r.cpu.random_state = r.player.seed;
        slots = std::iter::once(POLICE_CS)
            .chain((0..kinds.len() as u32 - 1).map(|n| EXTRA_MODELS + 0x200 * n))
            .collect();
        r.cpu
            .put(0x8018c336, &((1 + kinds.len()) as u16).to_le_bytes());
        r.cpu.put(0x8018c338, &(kinds.len() as u16).to_le_bytes());
        for (i, car) in lineup.iter().enumerate() {
            let n = i as u32;
            let cs = slots[i];
            r.cpu.put(POLICE + 0x560 * n, &car.state);
            Reference::sync_model(&mut r.cpu, cs, &car.state, 0x43c, 0x20c, 50 + n);
            for (addr, value) in [
                (cs + 0x104, root(car.vehicle as u16)),
                (0x8019f788 + 4 * n, cs),
                (0x8019ee10 + 4 * n, cs),
                (0x8019ed8c + 4 * n, 1),
                (0x8019eddc + 4 * n, car.vehicle as u32),
            ] {
                put(&mut r.cpu, addr, value);
            }
            r.cpu.put(0x801a22a8 + 2 * n, &(i as u16 + 1).to_le_bytes());
            r.cpu.put(0x8019edbe + 2 * n, &(i as u16).to_le_bytes());
        }
        let mut head = 0;
        for (_, kind) in roots {
            let cs = if matches!(kind, 750 | 752) {
                r.cpu.word(0x801f83fc + u32::from(kind == 752) * 28 + 4)
            } else if kind == vehicle as u16 {
                TAXI_CS
            } else if let Some(i) = kinds.iter().position(|v| *v as u16 == kind) {
                slots[i]
            } else {
                continue;
            };
            put(&mut r.cpu, cs + 0x108, head);
            head = cs;
        }
        put(&mut r.cpu, 0x8018bf14, head);
    }
    initialize_retail_music(&mut r.cpu, level);
    r.start_pedestrians(level);
    if cheats {
        r.player.set_weapon_and_damage_modes(false, true);
        r.cpu.call(0x8014d354, &[0, 1]);
    }
    // Observe the retail tire switch after every actor update, including reverse.
    let tire = |slot: u32| 0x8000d000 + slot * 32;
    for slot in 1..=13 {
        put(&mut r.cpu, 0x8019cd78 + slot * 12, tire(slot));
        r.cpu.put(tire(slot) + 6, &[251, 0]);
    }
    // Register three shadow groups for each model as the original database scanner does.
    let shadow = |kind: Vehicle, stage: u32| 0x8000d400 + (kind as u32 / 10 * 3 + stage) * 40;
    let rivals = if lineup.is_empty() {
        std::slice::from_ref(&r.police)
    } else {
        &lineup
    };
    for (kind, on) in std::iter::once((vehicle, r.player.shadow_visible())).chain(
        rivals
            .iter()
            .map(|car| (car.vehicle(), car.shadow_visible())),
    ) {
        for stage in 0..3 {
            let node = shadow(kind, stage);
            r.cpu.put(node, &[1, 0, (kind as u8) + 3, 0]);
            r.cpu.put(node + 22, &((stage + 1) as u16).to_le_bytes());
            r.cpu.call(0x8014174c, &[node]);
        }
        r.cpu.call(0x801417d0, &[kind as u32, u32::from(on)]);
    }
    let observe_guns =
        |cpu: &mut Cpu, entry, args: &[u32], car: Vehicle, flame_audio: &mut Vec<BattleEvent>| {
            let mut request = None;
            let mut origin = [0; 3];
            cpu.call_observed(entry, args, 0x80000080, |cpu| {
                if boss && car == Vehicle::Minion {
                    observe_flame_audio(cpu, &mut origin, 21, flame_audio);
                }
                let firing = match cpu.pc() {
                    0x801073cc => true,
                    0x80107524 | 0x80107a1c => false,
                    _ => return,
                };
                assert_eq!(cpu.r[4], car as u32);
                request = Some(firing);
            });
            request
        };
    let observe_actor_sounds = |cpu: &mut Cpu, address: u32, player: bool| {
        let mut sounds = Vec::new();
        if !player && cpu.ram[((address + 0x40) & 0x1fffff) as usize] == 0 {
            return sounds;
        }
        let offset = if player { 0x388 } else { 0x444 };
        let position = [0, 4, 8].map(|i| cpu.word(address + offset + i) as i32);
        cpu.call_observed(
            0x80150790,
            &[0, address, u32::from(player)],
            0x80000080,
            |cpu| {
                if cpu.pc() != 0x80106d60 {
                    return;
                }
                let level = cpu.word(cpu.r[29] + 16) as i32;
                let sound = match cpu.r[4] {
                    0x4d..=0x59 => return, // Engine pitch is checked separately.
                    0x6a => CarSound::Contact(ContactVoice::LightCrash, level),
                    0x6b => CarSound::Contact(ContactVoice::MediumCrash, level),
                    0x6c => CarSound::Contact(ContactVoice::HeavyCrash, level),
                    0x6f => CarSound::Contact(ContactVoice::Skid, level),
                    0x73 => {
                        assert_eq!((cpu.r[5], level), (0, 99));
                        CarSound::Explosion
                    }
                    0x74..=0x80 => CarSound::Signature,
                    id => panic!("unexpected actor sound {id:#x}"),
                };
                sounds.push((sound, position));
            },
        );
        sounds
    };
    for tick in 1..=360 {
        let input = Input {
            accelerate: true,
            fire_guns: !crush && tick > 10,
            missile: crush && tick == 1,
            left: (24..40).contains(&tick),
            ..Default::default()
        };
        let fields = if crush { 3 } else { [3u8, 4, 5, 3][tick % 4] };
        put(&mut r.cpu, 0x8017023c, tick as u32);
        r.cpu.put(0x80170238, &(fields as u16).to_le_bytes());
        r.cpu
            .put(0x8017023a, &(60u16 / fields as u16).to_le_bytes());
        let pads = if cheats {
            match tick {
                1 | 6 | 11 | 15 | 25 | 34 => 0x30c9,
                3 | 4 | 66 => 0xc036,
                8 | 18 => 0x3000,
                13 | 21 => 0xc035,
                23 => 0x30ca,
                27 | 28 | 32 => 0xc036_0000,
                30 => 0x30c9_0000,
                _ => 0,
            }
        } else {
            u32::from(input.ps1_buttons())
        };
        r.cpu.call(0x800ffb98, &[2, pads]);
        r.cpu.call(0x80110810, &[]);
        r.cpu.call(0x801344e0, &[RENDER_SCRATCH, 0]);
        r.cpu.call(0x801351b0, &[]);
        r.cpu.call(0x801333d0, &[RENDER_SCRATCH, 0]);
        r.cpu.call(0x8010e3d8, &[]);
        let read_fragments = |cpu: &Cpu| {
            (0..20)
                .filter_map(|i| {
                    let f = 0x80199db8 + 36 * i;
                    (cpu.word(f + 28) as i32 > 0).then(|| {
                        let variant = match cpu.word(f + 32) {
                            0x8019c5b8 => 0,
                            0x8019c5d8 => 1,
                            p => panic!("unexpected fragment animation {p:x}"),
                        };
                        [
                            cpu.word(f) as i32,
                            cpu.word(f + 4) as i32,
                            cpu.word(f + 8) as i32,
                            variant,
                            cpu.word(f + 24) as i32,
                        ]
                    })
                })
                .collect::<Vec<_>>()
        };

        // animate_explosions advances active fragments once.
        r.cpu.call(0x801360bc, &[]);
        let primary_fragments = read_fragments(&r.cpu);
        let mut gun_requests = [None; 9];
        let mut flame_audio = Vec::new();
        if r.cpu.call(0x8014e20c, &[0]) == 0 {
            gun_requests[0] = observe_guns(
                &mut r.cpu,
                0x8014ecd4,
                &[TAXI_CS, PLAYER],
                vehicle,
                &mut flame_audio,
            );
        }
        if r.cpu.word(PLAYER + 0x178) as i32 <= 0
            && r.cpu.ram[((PLAYER + 0xa0) & 0x1fffff) as usize] == 0
        {
            gun_requests[0] = Some(false);
        }
        let mut actor_sounds = vec![observe_actor_sounds(&mut r.cpu, PLAYER, true)];
        start_retail_death(&mut r.cpu, PLAYER, true);
        // Run the retail PlayGame battle scheduler up to the AI update loop.
        r.cpu.call_until(0x8014dcf0, &[], 0x8014df5c);
        for (i, &cs) in slots.iter().enumerate() {
            let address = POLICE + 0x560 * i as u32;
            let car = lineup.get(i).unwrap_or(&r.police).vehicle();
            gun_requests[1 + i] = observe_guns(
                &mut r.cpu,
                0x8014ee14,
                &[cs, address],
                car,
                &mut flame_audio,
            );
            actor_sounds.push(observe_actor_sounds(&mut r.cpu, address, false));
            start_retail_death(&mut r.cpu, address, false);
        }
        r.cpu.call_until(0x8014e0b4, &[], 0x8014e10c);
        r.cpu.call(0x8011c080, &[]);
        r.cpu.call(0x80135784, &[]);
        r.cpu.call(0x80135e10, &[]);
        r.cpu.call_until(0x8014e130, &[], 0x8014e1b0);
        if level != Level::Arena {
            r.cpu.call(0x8014843c, &[]);
            r.cpu.call(0x80149764, &[]);
            r.cpu.call(0x8014aaa0, &[]);
        }
        r.cpu.call(0x80120728, &[]);
        if lineup.is_empty() {
            r.scene.step(&mut r.player, &mut r.police, input, fields);
            r.compare();
        } else {
            r.scene
                .step_bout_with_pads(&mut r.player, &mut lineup, pads, fields);
            if cheats {
                match tick {
                    3 | 4 | 66 => assert_eq!(r.player.state[0x184], 1),
                    8 => assert_eq!(r.player.state[0x4a1], 1),
                    18 => assert_eq!(r.player.state[0x4a1], 0),
                    23 => assert_eq!(r.player.state[0x23c], 200),
                    27 | 28 => assert!(lineup.iter().all(|ai| ai.state[0] == 0)),
                    32 => assert!(lineup.iter().all(|ai| ai.state[0] == 1)),
                    _ => {}
                }
            }
            for index in 0..50 {
                let Some(record) = target_record(&r.scene, index) else {
                    break;
                };
                assert_eq!(
                    record[2],
                    r.cpu.word(0x801ac968 + index as u32 * 28 + 8) as i32,
                    "target {index} {:?} tick {tick}",
                    &record[..2]
                );
            }
            for (addr, state) in std::iter::once((PLAYER, &r.player.state[..])).chain(
                lineup
                    .iter()
                    .enumerate()
                    .map(|(i, ai)| (POLICE + 0x560 * i as u32, &ai.state[..])),
            ) {
                for (j, &v) in state.iter().enumerate() {
                    assert_eq!(
                        v,
                        r.cpu.ram[(addr as usize & 0x1fffff) + j],
                        "tick {tick} car {addr:x} byte {j:x}"
                    );
                }
            }
            assert_eq!(r.player.seed, r.cpu.random_state, "tick {tick} RNG");
        }
        let mut fragments = [[0; 5]; 20];
        for (secondary, expected) in [(false, primary_fragments), (true, read_fragments(&r.cpu))] {
            let count = r.scene.fragment_poses(secondary, &mut fragments);
            assert_eq!(
                fragments[..count],
                expected,
                "fragment sprites at tick {tick} secondary {secondary}"
            );
        }
        for (slot, expected) in actor_sounds.into_iter().enumerate() {
            assert_eq!(
                r.scene.car_sound_requests(slot).collect::<Vec<_>>(),
                expected,
                "{level:?} tick {tick} slot {slot}"
            );
        }
        assert_eq!(
            r.scene.gun_sound_requests(),
            gun_requests,
            "{level:?} tick {tick}"
        );
        let rivals = if lineup.is_empty() {
            std::slice::from_ref(&r.police)
        } else {
            &lineup
        };
        for (kind, on) in std::iter::once((vehicle, r.player.shadow_visible())).chain(
            rivals
                .iter()
                .map(|car| (car.vehicle(), car.shadow_visible())),
        ) {
            for stage in 0..3 {
                assert_eq!(
                    u8::from(on),
                    r.cpu.ram[((shadow(kind, stage) + 20) & 0x1fffff) as usize],
                    "{kind:?} shadow stage {stage} tick {tick}"
                );
            }
        }
        let steps = r.scene.wheel_steps();
        for slot in 1..=13 {
            if slot == 4 {
                assert_eq!(
                    steps[4],
                    i64::from(r.cpu.word(0x8018c2f8) as i32),
                    "Hammerhead tick {tick}"
                );
            } else {
                assert_eq!(
                    steps[slot as usize].rem_euclid(251),
                    i64::from(r.cpu.ram[((tire(slot) + 7) & 0x1fffff) as usize]),
                    "wheel slot {slot} tick {tick}"
                );
            }
        }
        assert_eq!(r.scene.level_won(), r.cpu.word(0x8018c360) != 0);
        assert_eq!(
            r.scene.battle_music_playing(),
            matches!(r.cpu.word(0x8018bf64), 5 | 7),
            "{level:?} tick {tick} music"
        );
        if boss {
            let native: Vec<_> = r
                .scene
                .audio
                .iter()
                .filter(|v| matches!(v, BattleEvent::Effect { kind: 11 | 21, .. }))
                .cloned()
                .collect();
            assert_eq!(native, flame_audio, "Minion tick {tick}");
            compare_projectiles(&r, vehicle, tick);
            if tick == 360 {
                let charge = i16::from_le_bytes(lineup[3].state[0x536..0x538].try_into().unwrap());
                assert!(charge < 20, "expected Minion to use the special weapon");
            }
        }
        if crush {
            let active = r.player.state[0xa0 + 27] != 0;
            if tick == 1 || charge < 6 {
                assert_eq!(r.player.weapon().1, charge);
                assert!(!active);
                if tick == 24 {
                    return;
                }
            }
            if !engaged && active {
                assert_eq!(r.player.weapon().1, 14);
                engaged = true;
            } else if engaged && !active {
                println!("Hammerhead completed the crush at step {tick}");
                return;
            }
        }
    }
    assert!(!crush, "expected to drive over the opponent and separate");
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn damage_body_switch_matches_retail() {
    let mut r = Reference::new();
    r.sync();
    // Keep the original health decision and stage write. Omit the presentation
    // callbacks for headlights, smoke and the actual DMD switch in this fixture.
    for entry in [0x8014158c, 0x80141c64, 0x80137140] {
        r.cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    for (car, stats, update, maximum) in [
        (PLAYER, 0x138, 0x8014e34c, r.player.health().1),
        (POLICE, 0x20c, 0x8014e6ac, r.police.health().1),
    ] {
        for hp in (1..=maximum).rev().chain(1..=maximum) {
            put(&mut r.cpu, car + stats + 0x40, hp as u32);
            r.cpu.call(update, &[0]);
            let expected = ((r.cpu.word(car + stats + 4) >> 16) & 255) as usize;
            let observed = if car == PLAYER {
                r.player.state[0x178..0x17c].copy_from_slice(&hp.to_le_bytes());
                r.player.body_stage()
            } else {
                r.police.state[0x24c..0x250].copy_from_slice(&hp.to_le_bytes());
                r.police.body_stage()
            };
            assert_eq!(observed, expected, "car {car:#x} health {hp}");
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn arena_animation_matches_retail() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let d = disc.read("UADMD/XARENA1.DMD").unwrap();
    let arena = tm1_assets::scene::Dmd::parse(&d)
        .unwrap()
        .extract_kind(1)
        .unwrap();
    let animation = arena
        .polygons
        .iter()
        .find_map(|p| p.animation.first())
        .unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    cpu.put(0x800188b8, &d);
    for visits in 1..=512 {
        cpu.call(0x800fe488, &[animation.node]);
        assert_eq!(
            animation.selected(visits),
            cpu.word(animation.node + 12) as u8
        );
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn radar_signals_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut player = PlayerVehicle::from_executable(&exe).unwrap();
    let mut cpu = Cpu::new(&exe);
    const RADAR: u32 = 0x8019e648;
    const OFFSET: u32 = 0x80030000;
    cpu.call(0x8014728c, &[]);
    let mut radar = Radar {
        ready: true,
        ..Default::default()
    };
    for heading in [0i32, 193, 512, 1024, -1536, 2048, 3737] {
        player.state[0x3a8..0x3ac].copy_from_slice(&heading.to_le_bytes());
        for vehicle in Vehicle::ROSTER {
            for [dx, dy] in [
                [0i32, 0],
                [1, 2],
                [-4020, 3021],
                [12000, 0],
                [-12000, 0],
                [12300, 0],
                [-12300, 0],
                [0, 7334],
                [0, -7334],
                [0, 7700],
                [0, -7700],
                [17000, 17000],
                [-17200, -17000],
                [-15000, 17000],
            ] {
                radar.display(true);
                radar.add(player.radar_offset(dx, dy, heading), Some(vehicle));
                for (i, v) in [dx, dy, 0].into_iter().enumerate() {
                    put(&mut cpu, OFFSET + 4 * i as u32, v as u32);
                }
                cpu.call(0x80146bd4, &[]);
                cpu.call(0x80147508, &[vehicle as u32, OFFSET, heading as u32]);
                let edges = cpu.word(RADAR + 0x35c);
                match radar.pending.marks.as_slice() {
                    &[RadarSignal::Contact { x, y, rgb }] => {
                        assert_eq!(edges, 0);
                        assert_eq!(cpu.word(RADAR + 0x360), 2);
                        let prim = RADAR + 0x30 + cpu.word(RADAR + 0x36c) * 16;
                        assert_eq!(
                            cpu.word(prim + 4) & 0xffffff,
                            u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0])
                        );
                        assert_eq!(
                            cpu.word(prim + 8),
                            (x + 53) as u32 | (((y + 55) as u32) << 16)
                        );
                    }
                    _ => {
                        assert!(radar.pending.marks.is_empty());
                        assert_eq!(cpu.word(RADAR + 0x360), 0);
                        assert_eq!(edges, u32::from_le_bytes(radar.pending.edges.map(u8::from)));
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn radar_hidden_frames_and_repair_stands_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let player = PlayerVehicle::from_executable(&exe).unwrap();
    let mut cpu = Cpu::new(&exe);
    const RADAR: u32 = 0x8019e648;
    const OFFSET: u32 = 0x80030000;
    cpu.call(0x8014728c, &[]);
    let mut radar = Radar::default();
    for visible in [true, false, false, true, true, true] {
        radar.display(visible);
        if visible && radar.ready {
            let frame = radar.frame().unwrap();
            assert_eq!(frame.marks.len() as u32 * 2, cpu.word(RADAR + 0x360));
            assert_eq!(
                u32::from_le_bytes(frame.edges.map(u8::from)),
                cpu.word(RADAR + 0x35c)
            );
            for (i, signal) in frame.marks.iter().enumerate() {
                let (x, y, rgb) = match *signal {
                    RadarSignal::Stand { x, y } => (x + 52, y + 54, [255, 200, 255]),
                    RadarSignal::Contact { x, y, rgb } => (x + 53, y + 55, rgb),
                };
                let prim = RADAR + 0x30 + 16 * (cpu.word(RADAR + 0x36c) + 4 * i as u32);
                assert_eq!(cpu.word(prim + 8), x as u32 | ((y as u32) << 16));
                assert_eq!(
                    cpu.word(prim + 4) & 0xffffff,
                    u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0])
                );
            }
            cpu.call(0x80146bd4, &[]);
        } else {
            assert!(radar.frame().is_none());
        }
        for (i, dx) in [0i32, 2, 500, 3000, -3900, 730, 19000, -19000]
            .into_iter()
            .enumerate()
        {
            let vehicle = (i != 0).then_some(Vehicle::ROSTER[i]);
            put(&mut cpu, OFFSET, dx as u32);
            put(&mut cpu, OFFSET + 4, 640);
            cpu.call(0x80147508, &[vehicle.map_or(920, |v| v as u32), OFFSET, 0]);
            radar.add(player.radar_offset(dx, 640, 0), vehicle);
        }
        radar.ready = true;
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn barricade_impacts_match_retail_switches() {
    unsafe extern "C" {
        fn UAeffectBarricade(tag: i32, sub: i32, bearing: i32);
    }
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let level = Level::Freeway;
    let data = disc
        .read(&format!(
            "UADMD/{}.DMD",
            level.database(Vehicle::YellowJacket)
        ))
        .unwrap();
    let world = tm1_assets::scene::Dmd::parse(&data)
        .unwrap()
        .extract_kind(1)
        .unwrap();
    let mut r = Reference::on_level(Vehicle::YellowJacket, true, level);
    for slot in 0..4 {
        let branches = world
            .polygons
            .iter()
            .flat_map(|p| p.arena_switches.iter().copied())
            .filter(|(s, _)| usize::from(*s) == slot)
            .fold(0u8, |mask, (_, branch)| mask | (1 << branch));
        assert_eq!(branches, 0b111);
        // Register the original switch record in retail RAM through UA_SW.c.
        let header = [9, 0, 0xbc, 2, slot as u8 + 1, 0, 3];
        let offset = (0..data.len() - 8)
            .step_by(4)
            .find(|&i| data[i] == header[0] && data[i + 2..i + 7] == header[2..])
            .expect("missing barricade switch");
        let addr = 0x800188b8 + offset as u32;
        r.cpu.call(0x80140f54, &[addr, 700]);
        assert_eq!(r.cpu.word(0x8019cfe4 + 4 * slot as u32), addr);
        let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        for bearing in [1, 2, 0, -1, 1] {
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                UAeffectBarricade(700, slot as i32 + 1, bearing);
                tm1_use_scene(std::ptr::null_mut());
            }
            r.cpu
                .call(0x801468bc, &[700, slot as u32 + 1, bearing as u32]);
            assert_eq!(
                r.scene.arena_switch_branches()[slot],
                r.cpu.ram[((addr + 7) & 0x1fffff) as usize],
                "{level:?} slot {slot} bearing {bearing}"
            );
        }
    }
    r.scene.reset_projectiles();
    assert_eq!(r.scene.arena_switch_branches(), [0; 8]);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftop_drop_box_matches_retail() {
    compare_rooftop_switches(1016, 1018, 4, [0x8018c5c8, 0x8018c5c4], 0x8014b0a4);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftop_breaking_window_matches_retail() {
    compare_rooftop_switches(1001, 1019, 6, [0x8019cef4, 0x8018c5cc], 0x8014b104);
}

fn compare_rooftop_switches(
    trigger: u16,
    animation: u16,
    first_slot: u8,
    globals: [u32; 2],
    advance: u32,
) {
    unsafe extern "C" {
        fn bulDispatchDamage(id: i32, tag: i32, sub: i32, damage: i32, dir: *const i32, owner: i32);
    }
    for vehicle in [Vehicle::YellowJacket, Vehicle::Hammerhead] {
        let level = Level::Rooftops;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc
            .read(&format!("UADMD/{}.DMD", level.database(vehicle)))
            .unwrap();
        let world = tm1_assets::scene::Dmd::parse(&data)
            .unwrap()
            .extract_kind(1)
            .unwrap();
        let mut r = Reference::on_level(vehicle, true, level);
        let mut records = Vec::new();
        let mut sub = 0;
        for (slot, kind, global) in [
            (first_slot, trigger, globals[0]),
            (first_slot + 1, animation, globals[1]),
        ] {
            let offset = (0..data.len() - 8)
                .step_by(4)
                .find(|&i| data[i] == 9 && data[i + 2..i + 4] == kind.to_le_bytes())
                .expect("missing roof switch");
            let addr = 0x800188b8 + offset as u32;
            if kind == 1001 {
                sub = r.cpu.word(addr + 4) & 65535;
                assert_eq!(r.cpu.word(global + sub * 4), addr);
            } else {
                r.cpu.call(0x80140f54, &[addr, u32::from(kind)]);
                assert_eq!(r.cpu.word(global), addr);
            }
            assert!(
                world
                    .polygons
                    .iter()
                    .any(|p| p.arena_switches.iter().any(|&(s, _)| s == slot))
            );
            records.push(offset + 7);
        }
        if trigger == 1001 {
            r.cpu.put(0x80108604, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        // Aim the midpoint collision sample inside the intact model.
        let vertices = world
            .polygons
            .iter()
            .filter(|p| p.arena_switches.contains(&(first_slot, 0)))
            .flat_map(|p| p.vertices.iter().map(|v| v.position));
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for pos in vertices {
            for i in 0..3 {
                lo[i] = lo[i].min(pos[i]);
                hi[i] = hi[i].max(pos[i]);
            }
        }
        let mut pos: [i32; 3] = std::array::from_fn(|i| ((lo[i] + hi[i]) * 0.5) as i32);
        // The pane is a plane. Its collision volume lies behind the visible surface.
        let center = pos;
        pos = (0..=400)
            .flat_map(|d| [center[0] - d, center[0] + d])
            .map(|x| [x, center[1], center[2]])
            .find(|&p| r.scene.world_point(p)[2] == i32::from(trigger))
            .expect("tagged trigger volume");
        pos[0] -= 125;
        let dir = [4096, 0, 0];
        unsafe extern "C" {
            fn create_bullet(owner: i32, dir: *const i32, pos: *const i32, damage: i32);
        }
        r.sync();
        for (addr, values) in [(0x800be000, dir), (0x800be020, pos)] {
            for (i, value) in values.into_iter().enumerate() {
                put(&mut r.cpu, addr + 4 * i as u32, value as u32);
            }
        }
        r.cpu.call(0x8010c930, &[1, 0x800be000, 0x800be020, 1]);
        {
            let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                create_bullet(1, dir.as_ptr(), pos.as_ptr(), 1);
                tm1_use_scene(std::ptr::null_mut());
            }
        }
        r.cpu.call(0x8010d03c, &[]);
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        assert_eq!(r.cpu.ram[0x188b8 + records[0]], 1, "retail shot at {pos:?}");
        assert_eq!(r.scene.arena_switch_branches()[usize::from(first_slot)], 1);
        if trigger == 1001 {
            let mut effects = [[0; 5]; 122];
            let count = r.scene.effects(&mut effects);
            let steam = effects[..count]
                .iter()
                .find(|e| e[3] == 31 && e[4] == 23)
                .unwrap();
            assert!(r.scene.audio.contains(&BattleEvent::Effect {
                kind: 33,
                position: [steam[0], steam[1], steam[2]]
            }));
            for (kind, life, rise) in [(8, 87, 10), (32, 15, 6)] {
                assert!(
                    effects[..count]
                        .iter()
                        .any(|e| e[..3] == [steam[0], steam[1], steam[2] + rise]
                            && e[3..] == [kind, life])
                );
            }
        }
        for tick in 0..25 {
            if matches!(tick, 4 | 15) {
                let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
                unsafe {
                    tm1_use_scene(r.scene.native.as_ptr());
                    bulDispatchDamage(8, i32::from(trigger), sub as i32, 1, std::ptr::null(), 0);
                    tm1_use_scene(std::ptr::null_mut());
                }
                r.cpu.call(0x80110f74, &[8, u32::from(trigger), sub, 1]);
            }
            r.scene
                .step(&mut r.player, &mut r.police, Input::default(), 3);
            r.cpu.call(advance, &[]);
            for (i, &offset) in records.iter().enumerate() {
                assert_eq!(
                    r.scene.arena_switch_branches()[usize::from(first_slot) + i],
                    r.cpu.ram[0x188b8 + offset],
                    "tick {tick}, switch {i}"
                );
            }
        }
        r.scene.reset_projectiles();
        assert_eq!(
            &r.scene.arena_switch_branches()[usize::from(first_slot)..usize::from(first_slot) + 2],
            &[0, 0]
        );
        let impact = [pos[0] + 125, pos[1], pos[2]];
        assert_eq!(r.scene.world_point(impact)[2], i32::from(trigger));
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn smoke_puff_and_steam_match_retail() {
    let r = Reference::new();
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let dmd = tm1_assets::scene::Dmd::parse(&data).unwrap();
    let sprite = (0..data.len() - 4)
        .step_by(4)
        .find(|&p| data[p..p + 4] == [0, 255, 0x8b, 2])
        .unwrap() as u32
        + 0x800188b8;
    cpu.put(0x800188b8, &data);
    cpu.call(0x8013785c, &[sprite]);
    for (steam, address) in [(false, 0x19c738), (true, 0x19c8c8)] {
        for (i, frame) in dmd.smoke_frames(steam).unwrap().iter().enumerate() {
            let at = address + 8 * i;
            assert_eq!(frame.uv[0], cpu.ram[at..at + 2]);
            assert_eq!(frame.width, cpu.ram[at + 2]);
            assert_eq!(frame.uv[2][1].wrapping_sub(frame.uv[0][1]), cpu.ram[at + 3]);
            assert_eq!(frame.texture.page.to_le_bytes(), cpu.ram[at + 4..at + 6]);
            assert_eq!(frame.texture.clut.to_le_bytes(), cpu.ram[at + 6..at + 8]);
        }
    }
    const PYRO: u32 = 0x8019a088;
    for i in 0..80 {
        put(&mut cpu, PYRO + i * 64 + 24, u32::MAX);
    }
    let pos = [42i32, 100, 400];
    for (i, value) in pos.into_iter().enumerate() {
        put(&mut cpu, 0x800be000 + 4 * i as u32, value as u32);
    }
    for address in [0x80137140, 0x8013721c, 0x80137190, 0x801371cc] {
        cpu.call(address, &[0x800be000]);
    }
    unsafe extern "C" {
        fn do_smoke(pos: *const i32);
        fn do_puff(pos: *const i32);
        fn arena_effect(pos: *const i32, kind: i32);
        fn arena_animate_effects();
    }
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        tm1_use_scene(r.scene.native.as_ptr());
        do_smoke(pos.as_ptr());
        do_puff(pos.as_ptr());
        arena_effect(pos.as_ptr(), 31);
        arena_effect(pos.as_ptr(), 34);
        tm1_use_scene(std::ptr::null_mut());
    }
    for tick in 0..=88 {
        let mut effects = [[0; 5]; 122];
        let count = r.scene.effects(&mut effects);
        for (slot, (kind, ttl)) in [(8, 88), (32, 16), (31, 24), (34, 88)]
            .into_iter()
            .enumerate()
        {
            let row = effects[..count].iter().find(|e| e[3] == kind);
            let addr = PYRO + slot as u32 * 64;
            let age = cpu.word(addr + 24) as i32;
            assert_eq!(age >= 0, row.is_some(), "tick {tick} kind {kind}");
            if let Some(row) = row {
                assert_eq!(row[..3], [0, 4, 8].map(|off| cpu.word(addr + off) as i32));
                assert_eq!(row[4], ttl - tick);
                let cycle = if matches!(kind, 8 | 34) && tick >= 24 {
                    (tick - 24) % 32
                } else {
                    tick
                };
                assert_eq!(cycle, age, "tick {tick} kind {kind}");
            }
        }
        unsafe {
            tm1_use_scene(r.scene.native.as_ptr());
            arena_animate_effects();
            tm1_use_scene(std::ptr::null_mut());
        }
        cpu.call(0x801360bc, &[]);
    }
}

fn target_record(scene: &CollisionScene, index: i32) -> Option<[i32; 6]> {
    unsafe extern "C" {
        fn tm1_target_record(scene: *const std::ffi::c_void, index: i32, out: *mut i32) -> i32;
    }
    let mut record = [0; 6];
    let found = unsafe { tm1_target_record(scene.native.as_ptr(), index, record.as_mut_ptr()) };
    (found != 0).then_some(record)
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn arena_target_registry_matches_retail() {
    let mut seen = std::collections::HashSet::new();
    for level in [
        Level::Arena,
        Level::Warehouse,
        Level::Freeway,
        Level::CityPark,
        Level::Cyburbia,
        Level::Rooftops,
    ] {
        for car in Vehicle::ROSTER {
            let db = level.database(car);
            if !seen.insert(db) {
                continue;
            }
            let mut r = Reference::on_level(car, true, level);
            let mut count = 0;
            for slot in 0..50 {
                let at = 0x801ac968 + slot * 28;
                let native = target_record(&r.scene, slot as i32);
                let record = [0, 4, 8, 16, 20, 24].map(|off| r.cpu.word(at + off) as i32);
                assert_eq!(
                    native.is_some(),
                    record[2] >= 0,
                    "{db} slot {slot}: {record:?}"
                );
                if let Some(native) = native {
                    assert_eq!(native, record, "{db} slot {slot}");
                    count += 1;
                }
            }
            compare_target_impacts(&mut r, count, db);
            println!("{db} has {count} targets");
        }
    }
}

fn compare_target_impacts(r: &mut Reference, count: i32, db: &str) {
    unsafe extern "C" {
        fn arena_hit_target(pos: *const i32, owner: i32, radius: i32) -> i32;
        fn bulDispatchDamage(id: i32, tag: i32, sub: i32, damage: i32, dir: *const i32, owner: i32);
    }
    // Isolate the arena targets from cars, pedestrians and audio.
    for addr in [0x8014f914, 0x8014f94c, 0x8014a4c4, 0x801085d0, 0x80108604] {
        r.cpu.put(addr, &[8, 0, 224, 3, 0x21, 0x10, 0, 0]);
    }
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read(&format!("UADMD/{db}.DMD")).unwrap();
    let art = tm1_assets::scene::Dmd::parse(&data)
        .unwrap()
        .extract_kind(1)
        .unwrap();
    let initial_records: Vec<_> = (0..count).map(|i| target_record(&r.scene, i)).collect();
    let initial_branches: Vec<_> = art
        .damage_branches
        .iter()
        .map(|(ptr, _, _)| r.scene.damage_branch(*ptr))
        .collect();
    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
    for index in 0..count {
        let [kind, sub, hp, x, y, z] = target_record(&r.scene, index).unwrap();
        for (dx, owner, radius) in [
            (0, 1, 0),
            (39, 1, 0),
            (40, 1, 0),
            (-40, 1, 0),
            (62, 1, 3),
            (64, 1, 3),
            (0, -kind, 0),
        ] {
            let point = [x + dx, y, z];
            for (i, n) in point.into_iter().enumerate() {
                put(&mut r.cpu, 0x80011000 + i as u32 * 4, n as u32);
            }
            let retail = r.cpu.call(0x8010c1d0, &[0x80011000, owner as u32, radius]);
            let native = unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                let result = arena_hit_target(point.as_ptr(), owner, radius as i32);
                tm1_use_scene(std::ptr::null_mut());
                result
            };
            assert_eq!(native as u32, retail, "{db} target {index} dx {dx}");
        }
        // Some retail registrations use the first coordinate of a translation
        // as the sub-index. Only destructible slots within the UA_SW arrays have models.
        if sub >= 30 || hp <= 0 {
            continue;
        }
        for (id, damage) in [(500 + index, (hp - 1).max(0)), (8, 1), (8, 1)] {
            r.cpu.call(
                0x80110f74,
                &[id as u32, kind as u32, sub as u32, damage as u32, 0, 0],
            );
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                bulDispatchDamage(id, kind, sub, damage, std::ptr::null(), 0);
                tm1_use_scene(std::ptr::null_mut());
            }
            for slot in 0..count {
                let health = r.cpu.word(0x801ac968 + 28 * slot as u32 + 8) as i32;
                assert_eq!(
                    target_record(&r.scene, slot).unwrap()[2],
                    health,
                    "{db} hit {index} health {slot}"
                );
            }
            for &(ptr, _, _) in &art.damage_branches {
                let p = (ptr & 0x1fffff) as usize;
                let branch = if r.cpu.ram[p] == 1 {
                    u8::from(r.cpu.ram[p + 20] == 0)
                } else {
                    r.cpu.ram[p + 7]
                };
                assert_eq!(
                    r.scene.damage_branch(ptr),
                    branch,
                    "{db} hit {index} model {ptr:x}"
                );
            }
        }
    }
    drop(_guard);
    r.scene.reset_projectiles();
    assert!(
        initial_records
            .into_iter()
            .eq((0..count).map(|i| target_record(&r.scene, i)))
    );
    assert!(
        initial_branches.into_iter().eq(art
            .damage_branches
            .iter()
            .map(|(ptr, _, _)| r.scene.damage_branch(*ptr)))
    );
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn rooftop_pyramid_bullet_matches_retail_clouds() {
    verify_pyramid_clouds(0);
    verify_pyramid_clouds(80);
}

fn verify_pyramid_clouds(prefill: usize) {
    let mut r = Reference::on_level(Vehicle::YellowJacket, true, Level::Rooftops);
    let (slot, record) = (0..50)
        .filter_map(|i| target_record(&r.scene, i).map(|v| (i, v)))
        .find(|(_, v)| v[0] == 1000 && v[1] < 30)
        .unwrap();
    assert_eq!(record[2], 100);
    let sample = (8000..9200)
        .step_by(10)
        .map(|z| [10800, 5200, z])
        .find(|&point| r.scene.world_point(point)[2] == 1000)
        .expect("pyramid collision volume");
    let pos = [sample[0] - 125, sample[1], sample[2]];
    let dir = [4096, 0, 0];
    r.sync();
    for (at, values) in [(0x800be000, dir), (0x800be020, pos)] {
        for (i, v) in values.into_iter().enumerate() {
            put(&mut r.cpu, at + i as u32 * 4, v as u32);
        }
    }
    r.cpu.call(0x80137268, &[]);
    for _ in 0..prefill {
        r.cpu.call(0x80136e18, &[0x800be020]);
    }
    r.cpu.call(0x8010c930, &[1, 0x800be000, 0x800be020, 100]);
    {
        let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe extern "C" {
            fn arena_effect(p: *const i32, kind: i32);
            fn create_bullet(owner: i32, dir: *const i32, pos: *const i32, damage: i32);
        }
        unsafe {
            tm1_use_scene(r.scene.native.as_ptr());
            for _ in 0..prefill {
                arena_effect(pos.as_ptr(), 35);
            }
            create_bullet(1, dir.as_ptr(), pos.as_ptr(), 100);
            tm1_use_scene(std::ptr::null_mut());
        }
    }
    let mut clouds = Vec::new();
    // Observe particle creation at the original call boundary. Audio output
    // is external to the collision and RNG comparison.
    r.cpu.put(0x80108604, &[8, 0, 224, 3, 0, 0, 0, 0]);
    r.cpu.call_observed(0x8010d03c, &[], 0x80000080, |cpu| {
        let kind = match cpu.pc() {
            0x801371cc => 34,
            0x80136d68 => 30,
            _ => return,
        };
        clouds.push([
            cpu.word(cpu.r[4]) as i32,
            cpu.word(cpu.r[4] + 4) as i32,
            cpu.word(cpu.r[4] + 8) as i32,
            kind,
        ]);
    });
    r.scene.phase(&mut r.player, &mut r.police, 2, 3);
    assert_eq!(target_record(&r.scene, slot).unwrap()[2], 0);
    assert_eq!(r.cpu.word(0x801ac968 + slot as u32 * 28 + 8), 0);
    assert_eq!(clouds.iter().filter(|c| c[3] == 34).count(), 45);
    let mut effects = [[0; 5]; 122];
    let count = r.scene.effects(&mut effects);
    let native: Vec<_> = effects[..count]
        .iter()
        .filter(|e| matches!(e[3], 30 | 34))
        .map(|e| [e[0], e[1], e[2], e[3]])
        .collect();
    let emitted: Vec<_> = r
        .scene
        .audio
        .iter()
        .filter_map(|event| match *event {
            BattleEvent::Effect {
                kind: kind @ (30 | 34),
                position: [x, y, z],
            } => Some([x, y, z, kind]),
            _ => None,
        })
        .collect();
    assert_eq!(emitted, clouds);
    if prefill == 80 {
        assert!(native.is_empty());
        assert_eq!(effects[..count].iter().filter(|e| e[3] == 35).count(), 80);
        for slot in 0..80 {
            let p = 0x8019a088 + slot * 64;
            assert_eq!(r.cpu.word(p + 24), 0);
            assert_eq!(r.cpu.word(p + 0x20), 80);
        }
    } else {
        for p in &mut clouds {
            if p[3] == 34 {
                p[2] += 10;
            }
        }
        let mut visible = native;
        visible.sort();
        clouds.sort();
        assert_eq!(visible, clouds);
    }
    assert_eq!(r.player.seed, r.cpu.random_state);
}

fn compare_projectiles(r: &Reference, vehicle: Vehicle, tick: usize) {
    let mut poses = [[0; 13]; 10];
    let count = r.scene.missile_poses(&mut poses);
    let expected = (0..10)
        .map(|i| 0x801a9c10 + 0x3e8 * i)
        .filter(|&m| r.cpu.word(m + 0x34) as i32 > 0)
        .map(|m| {
            let cs = r.cpu.word(m + 0x24);
            [0, 4, 8].map(|d| r.cpu.word(cs + 0x5c + d) as i32)
        });
    assert!(
        poses[..count]
            .iter()
            .map(|row| [row[1], row[2], row[3]])
            .eq(expected),
        "missiles {vehicle:?} tick {tick}"
    );
    let mut shots = [[[0; 3]; 2]; 40];
    let count = r.scene.bullet_segments(&mut shots);
    let expected = (0..40)
        .map(|i| 0x80198c28 + 0x3c * i)
        .filter(|&b| r.cpu.word(b + 0x28) as i32 > 0)
        .map(|b| [12, 0].map(|p| [0, 4, 8].map(|d| r.cpu.word(b + p + d) as i32)));
    assert!(
        shots[..count].iter().copied().eq(expected),
        "bullets {vehicle:?} tick {tick}"
    );
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn static_cop_encounters_match_disc() {
    let mut kinds = std::collections::BTreeSet::new();
    for level in [
        Level::Warehouse,
        Level::Freeway,
        Level::CityPark,
        Level::Cyburbia,
        Level::Rooftops,
    ] {
        let r = Reference::on_level(Vehicle::YellowJacket, true, level);
        for slot in 0..25 {
            let at = 0x8019eac0 + 28 * slot;
            let kind = r.cpu.word(at);
            if kind == 0 || !kinds.insert(kind) {
                continue;
            }
            let mut r = Reference::on_level(Vehicle::YellowJacket, true, level);
            let ptr = r.cpu.word(at + 4);
            let [x, y, z] = [16, 20, 24].map(|off| r.cpu.word(at + off) as i32);
            println!("{level:?} slot {slot} kind {kind} at {x},{y},{z}");
            for i in 0..10 {
                let m = 0x801a9c10 + 0x3e8 * i;
                put(&mut r.cpu, m + 0x34, (-12i32) as u32);
                put(&mut r.cpu, m + 0x24, MISSILE_CS + 0x110 * i);
                put(&mut r.cpu, m + 0x100, MISSILE_AUX + 0x110 * i);
                r.cpu.put(m + 0x30, &[i as u8]);
            }
            r.cpu.put(0x801085d0, &[8, 0, 224, 3, 0, 0, 0, 0]);
            r.sync();
            let mut phases = std::collections::BTreeSet::new();
            for t in 0..5600 {
                let dx = match t {
                    0..=11 => 1200,
                    12..=83 => 1199,
                    _ => 100000,
                };
                let pos = [x + dx, y, z];
                vec3(&mut r.player.state, 0x388, pos);
                vec3_cpu(&mut r.cpu, PLAYER + 0x388, pos);
                if t == 90 {
                    r.cpu.call(0x8014b010, &[slot + 1]);
                    unsafe extern "C" {
                        fn bulDispatchDamage(
                            id: i32,
                            tag: i32,
                            sub: i32,
                            dmg: i32,
                            dir: *const i32,
                            owner: i32,
                        );
                    }
                    let _guard = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
                    unsafe {
                        tm1_use_scene(r.scene.native.as_ptr());
                        bulDispatchDamage(8, kind as i32, slot as i32 + 1, 1, std::ptr::null(), 0);
                        tm1_use_scene(std::ptr::null_mut());
                    }
                    assert_eq!(r.scene.damage_branch(ptr), 4);
                    assert_eq!(r.cpu.ram[(ptr as usize & 0x1fffff) + 7], 4);
                    phases.insert(4);
                }
                r.cpu.call(0x8014aaa0, &[]);
                r.scene.phase(&mut r.player, &mut r.police, 3, 3);
                let branch = r.cpu.ram[(ptr as usize & 0x1fffff) + 7];
                assert_eq!(
                    r.scene.damage_branch(ptr),
                    branch,
                    "{level:?} {kind} tick {t}"
                );
                assert_eq!(
                    r.player.seed, r.cpu.random_state,
                    "{level:?} {kind} RNG tick {t}"
                );
                if t < 85 {
                    compare_projectiles(&r, Vehicle::YellowJacket, t);
                }
                phases.insert(branch);
            }
            assert_eq!(
                phases.into_iter().collect::<Vec<_>>(),
                (0..7).collect::<Vec<_>>()
            );
            assert_eq!(r.scene.damage_branch(ptr), 0);
        }
    }
    assert_eq!(kinds.into_iter().collect::<Vec<_>>(), [753, 754, 755]);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn hover_weapon_and_buffered_motor_queue_matches_disc() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    // Supply the world-to-eye result directly; positional_sound_matches_retail checks VIEW.c.
    cpu.put(0x8010461c, &[8, 0, 224, 3, 0, 0, 0, 0]);
    let listener = SoundListener {
        eye: [0; 3],
        matrix: [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
    };
    let table = cpu.call(0x801073bc, &[]);
    for (id, program) in [(0x83, 38), (0x84, 12), (0x15, 19)] {
        assert_eq!(cpu.ram[((table + 6 * id + 2) & 0x1fffff) as usize], program);
    }
    for (x0, x1) in [(500, -100), (100, -500), (500, -500)] {
        for first in [0, 1, 3, 5, 7] {
            for second in [0, 1, 3, 5, 7] {
                cpu.call(0x80108a50, &[]);
                put(&mut cpu, 0x8018bf7c, u32::MAX);
                let sources = [(x0, first), (x1, second)]
                    .map(|(x, flags)| (flags != 0).then_some([0, x, 0, 0, flags]));
                for [_, x, y, z, flags] in sources.into_iter().flatten() {
                    let (range, pan) = listener.range_and_x([x, y, z]);
                    put(&mut cpu, 0x8018c4dc, range as u32);
                    put(&mut cpu, 0x80196378, pan as u32);
                    cpu.call(
                        0x80148380,
                        &[0, (flags as u32 >> 1) & 1, (flags as u32 >> 2) & 1],
                    );
                }
                cpu.call(0x801089f0, &[]);
                let port: Vec<_> = hover_requests(sources, &listener).collect();
                assert_eq!((cpu.word(0x8018bf7c) as i32).max(0) as usize, port.len());
                for (i, (sound, position)) in port.into_iter().enumerate() {
                    let q = 0x801984e8 + i as u32 * 28;
                    let (id, kind, mode) = match sound {
                        HoverSound::Guns(on) => (0x84, if on { 2 } else { 3 }, 0),
                        HoverSound::Motor => (0x83, 2, 0),
                        HoverSound::Launch => (0x15, 2, 7),
                    };
                    assert_eq!(
                        (cpu.word(q), cpu.word(q + 24), cpu.word(q + 12)),
                        (id, kind, mode)
                    );
                    if kind == 2 {
                        let (range, pan) = listener.range_and_x(position);
                        assert_eq!(
                            (cpu.word(q + 4), cpu.word(q + 8)),
                            (range as u32, (-pan) as u32)
                        );
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn hover_mercenaries_match_disc() {
    for level in [
        Level::Warehouse,
        Level::Freeway,
        Level::CityPark,
        Level::Cyburbia,
        Level::Rooftops,
    ] {
        let mut r = Reference::on_level(Vehicle::YellowJacket, true, level);
        put(&mut r.cpu, 0x8018c3b4, level as u32);
        for at in [0x801085d0, 0x80108604] {
            r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        for i in 0..10 {
            let m = 0x801a9c10 + 0x3e8 * i;
            put(&mut r.cpu, m + 0x34, (-12i32) as u32);
            put(&mut r.cpu, m + 0x24, MISSILE_CS + 0x110 * i);
            put(&mut r.cpu, m + 0x100, MISSILE_AUX + 0x110 * i);
            r.cpu.put(m + 0x30, &[i as u8]);
        }
        r.sync();
        let mut phases = [
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        ];
        for t in 0..2200 {
            let target = if t < 1000 {
                [9000, 6500, 600]
            } else {
                [-6500, 1100, 800]
            };
            vec3(&mut r.player.state, 0x388, target);
            vec3_cpu(&mut r.cpu, PLAYER + 0x388, target);
            if t == 900 {
                for slot in 0..2 {
                    if r.scene.merc_pose(slot).is_none() {
                        continue;
                    }
                    let kind = [750, 752][slot];
                    r.cpu.call(0x80148e6c, &[kind, 1]);
                    r.scene
                        .phase(&mut r.player, &mut r.police, 5 + slot as u8, 3);
                }
            }
            let mut hover_requests = Vec::new();
            r.cpu.call_observed(0x8014843c, &[], 0x80000080, |cpu| {
                if cpu.pc() == 0x80148380 {
                    hover_requests.push((cpu.r[4], cpu.r[5], cpu.r[6]));
                }
            });
            r.scene.phase(&mut r.player, &mut r.police, 4, 3);
            for (slot, seen) in phases.iter_mut().enumerate() {
                let merc = 0x801f83fc + 28 * slot as u32;
                let cs = r.cpu.word(merc + 4);
                let pose = r.scene.merc_pose(slot);
                assert_eq!(cs != 0, pose.is_some());
                if let Some([branch, x, y, z, hover]) = pose {
                    let request = hover_requests
                        .iter()
                        .find(|(source, _, _)| *source == cs + 0x5c);
                    assert_eq!(hover != 0, request.is_some());
                    let expected = request.map_or(0, |(_, firing, launching)| {
                        1 | (firing << 1) | (launching << 2)
                    });
                    assert_eq!(
                        r.scene.merc_state(slot).unwrap()[4] as u32,
                        expected,
                        "{level:?} slot {slot} tick {t}"
                    );
                    let sw = r.cpu.word(merc + 8);
                    assert_eq!(
                        branch,
                        r.cpu.ram[(sw as usize & 0x1fffff) + 7] as i32,
                        "{level:?} {slot} {t}"
                    );
                    assert_eq!(
                        [x, y, z],
                        [0, 4, 8].map(|d| r.cpu.word(cs + 0x5c + d) as i32),
                        "{level:?} {slot} {t}"
                    );
                    seen.insert(branch);
                }
            }
            assert_eq!(r.player.seed, r.cpu.random_state, "{level:?} tick {t} RNG");
            compare_projectiles(&r, Vehicle::YellowJacket, t);
        }
        for seen in phases.iter().filter(|p| !p.is_empty()) {
            assert_eq!(seen.iter().copied().collect::<Vec<_>>(), [0, 1, 2, 3, 4]);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn pedestrians_match_disc() {
    for (level, kinds) in [
        (Level::Warehouse, &[760][..]),
        (Level::Freeway, &[756][..]),
        (Level::CityPark, &[781, 756, 773][..]),
        (Level::Cyburbia, &[756, 778, 760][..]),
        (Level::Rooftops, &[756][..]),
    ] {
        let mut r = Reference::on_level(Vehicle::YellowJacket, true, level);
        r.cpu.put(0x801085d0, &[8, 0, 224, 3, 0, 0, 0, 0]);
        r.start_pedestrians(level);
        assert_eq!(
            kinds,
            r.pedestrian_nodes.iter().map(|n| n.0).collect::<Vec<_>>()
        );
        r.player.state[0x3a8..0x3aa].copy_from_slice(&517i16.to_le_bytes());
        r.sync();
        let start_poses: Vec<_> = (0..kinds.len())
            .map(|i| r.scene.pedestrian_pose(i))
            .collect();
        let mut hidden = vec![false; kinds.len()];
        let mut shots = false;
        for tick in 0..2000 {
            let slot = tick / 500 % kinds.len();
            let base = 0x8019e9b8 + 44 * slot as u32;
            let cs = r.cpu.word(base);
            let mut player = [0, 4, 8].map(|off| r.cpu.word(cs + 0x5c + off) as i32);
            player[0] += match tick % 500 {
                0..=299 => 479,
                300..=319 => 959,
                320..=329 => 960,
                339 => 56,
                340 => 55,
                _ => 100000,
            };
            vec3(&mut r.player.state, 0x388, player);
            vec3_cpu(&mut r.cpu, PLAYER + 0x388, player);
            if tick % 500 == 339 || tick % 500 == 340 {
                vec3_cpu(&mut r.cpu, 0x80011000, player);
                let hit = r.cpu.call(0x8014a4c4, &[0x80011000, 1]);
                assert_eq!(hit, u32::from(kinds[slot] != 760 && tick % 500 == 340));
                r.scene.phase(&mut r.player, &mut r.police, 14, 3);
            }
            if tick % 500 == 365 {
                r.cpu.call(0x8014a484, &[slot as u32, 1]);
                r.scene
                    .phase(&mut r.player, &mut r.police, 8 + slot as u8, 3);
            }
            r.cpu.call_observed(0x80149764, &[], 0x80000080, |cpu| {
                if cpu.pc() == 0x801370d0 {
                    shots = true;
                }
            });
            r.scene.phase(&mut r.player, &mut r.police, 7, 3);
            for (i, _) in kinds.iter().enumerate() {
                let ped = 0x8019e9b8 + 44 * i as u32;
                let cs = r.cpu.word(ped);
                let ptr = r.cpu.word(ped + 4);
                let mut pose = [0i32; 14];
                pose[0] = r.cpu.ram[(ptr as usize & 0x1fffff) + 7].into();
                for k in 0..3 {
                    pose[1 + k] = r.cpu.word(cs + 0x5c + 4 * k as u32) as i32;
                }
                for k in 0..9 {
                    let p = (cs as usize & 0x1fffff) + 0x14 + 2 * k;
                    pose[4 + k] =
                        i16::from_le_bytes(r.cpu.ram[p..p + 2].try_into().unwrap()) as i32;
                }
                pose[13] = r.cpu.word(cs + 8) as i32;
                hidden[i] |= pose[13] == 0;
                assert_eq!(
                    Some(pose),
                    r.scene.pedestrian_pose(i),
                    "{level:?} tick {tick} slot {i}"
                );
            }
            assert_eq!(r.cpu.random_state, r.player.seed, "{level:?} tick {tick}");
            compare_projectiles(&r, Vehicle::YellowJacket, tick);
        }
        assert!(hidden.iter().zip(kinds).all(|(&h, &k)| h == (k != 760)));
        assert_eq!(shots, kinds.contains(&756), "{level:?}");
        r.scene.reset_projectiles();
        assert_eq!(
            start_poses,
            (0..kinds.len())
                .map(|i| r.scene.pedestrian_pose(i))
                .collect::<Vec<_>>()
        );
    }
}

fn initialize_retail_music(cpu: &mut Cpu, level: Level) {
    // Keep soundStartPlayDA's current track store and bypass the CD device.
    for (i, ins) in [0x3c088019, 0x03e00008, 0xad04bf64].into_iter().enumerate() {
        put(cpu, 0x801057e0 + 4 * i as u32, ins);
    }
    put(cpu, 0x8018c378, level as u32 + 1);
    let track = match level {
        Level::Warehouse | Level::CityPark => 4,
        Level::Cyburbia => 6,
        Level::Arena => 3,
        Level::Rooftops => 8,
        _ => 2,
    };
    put(cpu, 0x8018bf64, track);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn hunt_and_battle_music_match_disc() {
    unsafe extern "C" {
        fn arena_request_battle_music();
        fn tm1_update_battle_music(scene: *mut std::ffi::c_void, level: i32, engaged: i32);
    }
    for level in [
        Level::Warehouse,
        Level::CityPark,
        Level::Cyburbia,
        Level::Freeway,
    ] {
        let mut r = Reference::on_level(Vehicle::YellowJacket, false, level);
        initialize_retail_music(&mut r.cpu, level);
        let mut changes = Vec::new();
        let mut last = false;
        for tick in 0..290 {
            let engaged = matches!(tick, 2..=145|198..=250);
            let fired = (tick < 2 || tick % 3 != 0) && !(99..120).contains(&tick);
            let _lock = crate::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            if fired {
                r.cpu.call(0x8014b208, &[]);
            }
            r.cpu.put(0x8018bfd0, &i16::from(engaged).to_le_bytes());
            r.cpu.call_until(0x8014e130, &[], 0x8014e1b0);
            unsafe {
                tm1_use_scene(r.scene.native.as_ptr());
                if fired {
                    arena_request_battle_music();
                }
                tm1_update_battle_music(r.scene.native.as_ptr(), level as i32, i32::from(engaged));
                tm1_use_scene(std::ptr::null_mut());
            }
            let battle = r.scene.battle_music_playing();
            if battle != last {
                changes.push((tick, battle));
                last = battle;
            }
            let track = r.cpu.word(0x8018bf64);
            assert_eq!(
                battle,
                matches!(track, 5 | 7),
                "{level:?} tick {tick}, track {track}"
            );
            if battle {
                assert_eq!(track, if level == Level::CityPark { 5 } else { 7 });
            }
        }
        if level == Level::Freeway {
            assert!(changes.is_empty());
        } else {
            assert_eq!(
                changes.iter().map(|&(_, b)| b).collect::<Vec<_>>(),
                [true, false, true, false]
            );
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn split_screen_player_starts_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    cpu.call(0x800ffad8, &[1]);
    for level in [
        Level::Arena,
        Level::Warehouse,
        Level::Freeway,
        Level::CityPark,
        Level::Cyburbia,
        Level::Rooftops,
        Level::Arena5,
    ] {
        put(&mut cpu, 0x8018c3b4, level as u32);
        let players = PlayerVehicle::duel_players(
            &exe,
            [Vehicle::Outlaw, Vehicle::Darkside],
            level,
            1,
            Difficulty::Medium,
        )
        .unwrap();
        for (slot, car) in players.iter().enumerate() {
            cpu.put(PLAYER + 0x388, &[0xff; 0x30]);
            cpu.call(0x80122ea4, &[PLAYER, u32::from(slot == 0)]);
            assert_eq!(car.state[2..4], (slot as u16).to_le_bytes());
            for offset in [0x388, 0x38c, 0x390, 0x3a8, 0x3ac, 0x3b0, 0x3b4] {
                assert_eq!(
                    car.word(offset),
                    cpu.word(PLAYER + offset as u32) as i32,
                    "{level:?} player {slot} offset {offset:#x}"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_pad_routing_matches_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    let mut cars = PlayerVehicle::duel_players(
        &exe,
        [Vehicle::YellowJacket; 2],
        Level::Arena,
        1,
        Difficulty::Medium,
    )
    .unwrap();
    for (tick, buttons) in [0x00000080, 0x00800000, 0xa0802080, 0x00000000]
        .into_iter()
        .enumerate()
    {
        cpu.call(0x800ffb98, &[2, buttons]);
        put(&mut cpu, 0x8017023c, tick as u32 + 1);
        for car in &mut cars {
            cpu.put(PLAYER, &car.state);
            cpu.call(0x801125ac, &[PLAYER]);
            car.step_with_pads(buttons, 3);
            let pad = (PLAYER & 0x1fffff) as usize + 0xc8;
            assert_eq!(&car.state[0xc8..0xc8 + 44], &cpu.ram[pad..pad + 44]);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_machine_gun_sound_ids_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    // Observe the sound queue calls; projectile geometry and audio hardware aren't used by this trace.
    for entry in [0x8010461c, 0x80106d60, 0x80106e88, 0x8010c930, 0x801370d0] {
        cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    let kinds = [Vehicle::YellowJacket, Vehicle::Outlaw];
    let mut cars =
        PlayerVehicle::duel_players(&exe, kinds, Level::Arena, 1, Difficulty::Medium).unwrap();
    let data = disc.read("UA2PLAY/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |k| roots.iter().find(|r| r.1 == k).unwrap().0;
    let mut scene =
        CollisionScene::for_duel(&data, root(1), kinds.map(|v| root(v as u16))).unwrap();
    let mut hot = [false; 2];
    for tick in 0..300 {
        let held = [tick >= 8, tick < 7 || tick > 25];
        let mut expected = [None; 2];
        for slot in 0..2 {
            let address = PLAYER + (slot * cars[slot].state.len()) as u32;
            cpu.put(address, &cars[slot].state);
            cpu.put(address + 0xd1, &[u8::from(held[slot])]);
            cpu.call_observed(0x8013199c, &[address, 1], 0x80000080, |cpu| {
                let firing = match cpu.pc() {
                    0x80106d60 => true,
                    0x80106e88 => false,
                    _ => return,
                };
                assert_eq!(cpu.r[4], [91, 94][slot]);
                expected[slot] = Some(firing);
            });
            hot[slot] |= cars[slot].gun_overheated() && expected[slot] == Some(false);
        }
        scene.step_duel(
            &mut cars,
            held.map(|fire_guns| Input {
                fire_guns,
                ..Default::default()
            }),
            3,
        );
        assert_eq!(scene.duel_gun_requests(), expected, "tick {tick}");
    }
    assert_eq!(hot, [true; 2]);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn hammerhead_duel_crush() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let data = disc.read("UA2PLAY/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |kind| roots.iter().find(|r| r.1 == kind).unwrap().0;
    for slot in 0..2 {
        for charge in [5i16, 6] {
            let target = 1 - slot;
            let mut kinds = [Vehicle::Outlaw; 2];
            kinds[slot] = Vehicle::Hammerhead;
            let mut cars =
                PlayerVehicle::duel_players(&exe, kinds, Level::Arena, 1, Difficulty::Medium)
                    .unwrap();
            let mut scene =
                CollisionScene::for_duel(&data, root(1), kinds.map(|v| root(v as u16))).unwrap();
            for (i, car) in cars.iter_mut().enumerate() {
                vec3(
                    &mut car.state,
                    0x388,
                    [2700, if i == slot { 2000 } else { 2450 }, 0],
                );
                vec3(&mut car.state, 0x3a0, [0; 3]);
                vec3(&mut car.state, 0x3b8, [0; 3]);
            }
            cars[slot].state[0x47a..0x47c].copy_from_slice(&charge.to_le_bytes());
            // Execute InitMonsterSmash on the two retail player records. Capture the
            // audio queue and damage call at their endpoints, before SPU or debris work.
            let mut cpu = Cpu::new(&exe);
            let victim = PLAYER + cars[0].state.len() as u32;
            cpu.put(PLAYER, &cars[slot].state);
            cpu.put(victim, &cars[target].state);
            put(&mut cpu, 0x8018c634 + 4 * target as u32, TAXI_CS);
            put(&mut cpu, TAXI_CS + 0xc0, 1 + target as u32);
            for endpoint in [0x80106d60, 0x80117a68] {
                cpu.put(endpoint, &[8, 0, 224, 3, 0, 0, 0, 0]);
            }
            let mut retail_sound = Vec::new();
            let mut damage = None;
            cpu.call_observed(
                0x801188b4,
                &[PLAYER, 1, victim, 1, 0],
                0x80000080,
                |cpu| match cpu.pc() {
                    0x80106d60 => retail_sound.push([cpu.r[4], cpu.r[5], cpu.r[6]]),
                    0x80117a68 => {
                        assert_eq!(cpu.r[4], target as u32 + 1);
                        damage = Some(cpu.r[5] as i32);
                    }
                    _ => {}
                },
            );
            assert_eq!(damage, (charge >= 6).then_some(12));
            let hp = cars[target].health().0;
            let mut hit = false;
            let mut separated = false;
            let mut sounds = 0;
            for tick in 0..60 {
                let mut input = [Input::default(); 2];
                input[slot] = Input {
                    accelerate: true,
                    missile: tick == 0,
                    ..Default::default()
                };
                scene.step_duel(&mut cars, input, 3);
                sounds += scene
                    .audio
                    .iter()
                    .filter(|e| **e == BattleEvent::MonsterSmash(slot))
                    .count();
                if cars[slot].state[0xa0 + 27] != 0 {
                    if !hit {
                        assert_eq!(cars[target].state[0xa0 + 28], 1);
                        assert_eq!(
                            cars[target].health().0,
                            hp - damage.unwrap(),
                            "slot {slot} tick {tick}"
                        );
                        assert_eq!(
                            cars[slot].weapon().1,
                            (cpu.word(PLAYER + 0x478) >> 16) as i16
                        );
                        for offset in [0x138 + 0x2a, 0x138 + 0x2c, 0x138 + 0x30] {
                            let start = ((PLAYER & 0x1fffff) as usize) + offset;
                            assert_eq!(
                                cars[slot].state[offset..offset + 2],
                                cpu.ram[start..start + 2]
                            );
                        }
                    }
                    hit = true;
                } else if hit {
                    separated = true;
                    assert_eq!(cars[target].state[0xa0 + 28], 0);
                    break;
                }
            }
            assert_eq!(hit, damage.is_some(), "slot {slot} charge {charge}");
            assert_eq!(separated, hit);
            assert_eq!(sounds, retail_sound.len());
            if hit {
                assert_eq!(retail_sound, [[0x36, 0, 0]]);
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn shared_duel_crossfire() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let data = disc
        .read(&format!(
            "UA2PLAY/{}.DMD",
            Level::Arena.duel_database().unwrap()
        ))
        .unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |kind| roots.iter().find(|r| r.1 == kind).unwrap().0;
    for shooter in 0..2 {
        let mut cars = PlayerVehicle::duel_players(
            &exe,
            [Vehicle::YellowJacket, Vehicle::Outlaw],
            Level::Arena,
            1,
            Difficulty::Medium,
        )
        .unwrap();
        let mut scene = CollisionScene::for_duel(
            &data,
            root(1),
            [
                root(Vehicle::YellowJacket as u16),
                root(Vehicle::Outlaw as u16),
            ],
        )
        .unwrap();
        let target = 1 - shooter;
        for (slot, car) in cars.iter_mut().enumerate() {
            vec3(
                &mut car.state,
                0x388,
                [2700, if slot == shooter { 2000 } else { 2600 }, 0],
            );
            vec3(&mut car.state, 0x3a0, [0; 3]);
            vec3(&mut car.state, 0x3b8, [0; 3]);
        }
        let before = cars[target].health().0;
        let shooter_health = cars[shooter].health().0;
        let mut input = [Input::default(); 2];
        input[shooter].fire_guns = true;
        for _ in 0..60 {
            scene.step_duel(&mut cars, input, 3);
        }
        assert!(
            cars[target].health().0 < before,
            "shots from slot {shooter} missed"
        );
        assert_eq!(cars[shooter].health().0, shooter_health);
        assert_eq!(cars[0].seed, cars[1].seed);
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_stages_from_disc() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    for level in [
        Level::Arena,
        Level::Warehouse,
        Level::Freeway,
        Level::CityPark,
        Level::Cyburbia,
        Level::Rooftops,
    ] {
        let stem = level.duel_database().unwrap();
        let data = disc.read(&format!("UA2PLAY/{stem}.DMD")).unwrap();
        let dmd = tm1_assets::scene::Dmd::parse(&data).unwrap();
        let root = |kind| dmd.roots.iter().find(|r| r.1 == kind).unwrap().0;
        let kinds = [Vehicle::YellowJacket, Vehicle::Outlaw];
        let mut scene =
            CollisionScene::for_duel(&data, root(1), kinds.map(|v| root(v as u16))).unwrap();
        let mut cars =
            PlayerVehicle::duel_players(&exe, kinds, level, 1, Difficulty::Medium).unwrap();
        for (slot, car) in cars.iter_mut().enumerate() {
            car.state[0x49e..0x4a0].copy_from_slice(&(11 + slot as i16).to_le_bytes());
        }
        let initial = cars.each_ref().map(PlayerVehicle::position);
        let inputs = [Input {
            accelerate: true,
            ..Default::default()
        }; 2];
        for _ in 0..20 {
            scene.step_duel(&mut cars, inputs, 3);
        }
        for (slot, car) in cars.iter().enumerate() {
            assert_eq!(car.turbo_units(), 100, "{stem} slot {slot}");
            assert_ne!(
                initial[slot][..2],
                car.position()[..2],
                "{stem} slot {slot}"
            );
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_round_reseeds_from_retail_timer() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = Cpu::new(&exe);
    let mut cars = PlayerVehicle::duel_players(
        &exe,
        [Vehicle::YellowJacket, Vehicle::Outlaw],
        Level::Arena,
        1,
        Difficulty::Medium,
    )
    .unwrap();
    // Run the final GetCurTics/srand calls of uaInitDB and InitLevel. Only the
    // physical counter read is supplied; GetCurTics combines it with gTotalTics.
    for (entry, total, counter) in [
        (0x8014b6fc, 65535u32, 7u16),
        (0x8014c2fc, 0xffff_ff00, 0x345),
    ] {
        put(&mut cpu, 0x80170240, total);
        put(&mut cpu, 0x80162f40, 0x03e00008);
        put(&mut cpu, 0x80162f44, 0x34020000 | u32::from(counter));
        cpu.call_until(entry, &[], entry + 16);
        let ticks = total.wrapping_add(u32::from(counter));
        assert_eq!(cpu.random_state, ticks);
        let draws: Vec<_> = (0..32).map(|_| cpu.call(0x80154898, &[])).collect();
        for car in &mut cars {
            car.set_round_seed(ticks);
            for &draw in &draws {
                assert_eq!(car.random(), draw);
            }
            car.reset();
            assert_eq!(car.random(), draws[0]);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn duel_winner_and_return_delay_original() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let data = disc.read("UA2PLAY/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |kind| roots.iter().find(|r| r.1 == kind).unwrap().0;
    let kinds = [Vehicle::YellowJacket, Vehicle::Outlaw];
    for death_ticks in [[0, 200], [200, 0], [0, 0], [2, 0]] {
        let mut cpu = Cpu::new(&exe);
        cpu.call(0x800ff718, &[]);
        let mut cars =
            PlayerVehicle::duel_players(&exe, kinds, Level::Arena, 1, Difficulty::Medium).unwrap();
        let mut scene =
            CollisionScene::for_duel(&data, root(1), kinds.map(|v| root(v as u16))).unwrap();
        let mut winner = None;
        for tick in 0..100 {
            let fields = [1u8, 5, 3][tick % 3];
            cpu.put(0x80170238, &u16::from(fields).to_le_bytes());
            for (slot, car) in cars.iter_mut().enumerate() {
                if tick == death_ticks[slot] {
                    car.state[0x178..0x17c].fill(0);
                    cpu.call(0x800ff6dc, &[if slot == 0 { 4 } else { 3 }, 180]);
                }
            }
            winner = scene.step_duel(&mut cars, [Input::default(); 2], fields);
            let returned = cpu.call(0x800ff760, &[]) != 0;
            assert_eq!(
                winner.is_some(),
                returned,
                "deaths {death_ticks:?}, tick {tick}"
            );
            if returned {
                assert_eq!(winner, Some((cpu.word(0x8018bf1c) - 3) as usize));
                break;
            }
        }
        assert!(winner.is_some());
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn voice_priority_and_range_partitions_match_retail() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    let list = 0x80010000;
    for n in [0, 1, 2, 23, 24, 25, 39, 48] {
        for pattern in 0..4 {
            let mut voices = Vec::new();
            for i in 0..n {
                let address = list + 0x1000 + 0x84 * i;
                let priority = if pattern < 2 {
                    21
                } else {
                    ((17 * i + 5) % 7) as u8
                };
                let range: u32 = match pattern {
                    0 => 0,
                    1 => i % 3,
                    2 => 0,
                    _ => i.wrapping_mul(0xb12056c5),
                };
                cpu.put(address + 6, &[priority]);
                cpu.put(address + 0x74, &range.to_le_bytes());
                cpu.put(list + 4 * i, &address.to_le_bytes());
                voices.push((address, priority, range));
            }
            cpu.call(0x80104f98, &[list, 0, n.wrapping_sub(1)]);
            crate::sound::sort_requested(&mut voices, |v| v.2, |v| v.1);
            for (i, (address, _, _)) in voices.into_iter().enumerate() {
                assert_eq!(
                    address,
                    cpu.word(list + 4 * i as u32),
                    "size {n}, pattern {pattern}, slot {i}"
                );
            }
        }
    }
}
