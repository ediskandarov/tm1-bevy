use super::*;
#[path = "../tests/support/mips.rs"]
mod mips;
use mips::Cpu;

const PLAYER: u32 = 0x8019ee40;
const POLICE: u32 = 0x8019f7a8;
const TAXI_CS: u32 = 0x800bd000;
const POLICE_CS: u32 = 0x800bd200;
const WORLD: u32 = 0x800bd400;
fn put(cpu: &mut Cpu, at: u32, v: u32) {
    cpu.put(at, &v.to_le_bytes());
}

struct Reference {
    cpu: Cpu,
    player: Taxi,
    police: PoliceCar,
    scene: CollisionScene,
}
impl Reference {
    fn new() -> Self {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
        let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
        let exe = disc.read("SIPS_600.07").unwrap();
        let mut cpu = Cpu::new(&exe);
        let scene = CollisionScene::from_dmd(&data, root(1), root(20), root(50)).unwrap();
        let mut player = Taxi::from_executable(&exe).unwrap();
        let mut police = PoliceCar::spawn(&mut player);
        police.state[0x40] = 1;
        cpu.put(0x800188b8, &data);
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
            (TAXI_CS + 0x104, root(20)),
            (POLICE_CS + 0x104, root(50)),
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
            (0x8018c36c, 1),
        ] {
            cpu.put(at, &v.to_le_bytes());
        }
        unsafe extern "C" {
            fn tm1_pickup_address(scene: *mut std::ffi::c_void, i: i32) -> u32;
        }
        for i in 0..50 {
            put(&mut cpu, 0x8019ce2c + 4 * i, unsafe {
                tm1_pickup_address(scene.0.as_ptr(), i as i32)
            });
        }
        cpu.call(0x80102b48, &[0x800c1000]);
        cpu.call(0x80103a4c, &[0x800c3000]);
        // Isolated physics runs skip display/audio request endpoints on both sides.
        for at in [
            0x80146754, 0x80110d90, 0x801369b0, 0x80135f90, 0x8013721c, 0x8010461c, 0x8010767c,
            0x80107524, 0x80107bc0, 0x80142810,
        ] {
            cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        Self {
            cpu,
            scene,
            player,
            police,
        }
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
            tm1_use_scene(r.scene.0.as_ptr());
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
        r.police.update(&mut r.player, 3, false, r.scene.0.as_ptr());
        r.compare();
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn fire_missile_impact_original() {
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
    vec3(&mut r.police.state, 0x444, [2750, 3000, 400]);
    r.player.state[0xd2] = 1;
    r.player.state[0x460..0x464].fill(0);
    r.sync();
    r.cpu.call(0x80113858, &[PLAYER, 1]);
    r.cpu.call(0x8013148c, &[PLAYER, 1]);
    r.scene.phase(&mut r.player, &mut r.police, 1, 3);
    r.compare();
    assert_eq!(r.player.missiles(), 1);
    let before = r.police.health().0;
    let mut hits = 0;
    for _ in 0..30 {
        if r.cpu.word(m + 0x34) as i32 <= 0 {
            break;
        }
        r.cpu.call(0x80110210, &[m]);
        r.cpu.call(0x80137d2c, &[]);
        r.scene.phase(&mut r.player, &mut r.police, 2, 3);
        r.compare();
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
    assert_eq!(r.police.health().0, before - 7);
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn coupled_arena_original() {
    let mut r = Reference::new();
    r.police.state[0x40] = 0;
    for entry in [0x80125bc0, 0x8012cd90, 0x80130680, 0x80130a8c, 0x801312f8] {
        r.cpu.call(entry, &[]);
    }
    for at in [
        0x801417d0, 0x801466b0, 0x80146930, 0x801073cc, 0x8014b208, 0x80108604, 0x80116bcc,
        0x8014eff4, 0x801370d0, 0x80108378, 0x801083b0,
    ] {
        r.cpu.put(at, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    r.cpu.put(0x8018c378, &1u16.to_le_bytes());
    put(&mut r.cpu, 0x8018c3b4, 0);
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
        put(&mut r.cpu, m + 0x24, 0x800d0000 + 0x110 * i);
        put(&mut r.cpu, m + 0x100, 0x800d1000 + 0x110 * i);
        r.cpu.put(m + 0x30, &[i as u8]);
        put(&mut r.cpu, 0x800d0000 + 0x110 * i + 0xc0, 200 + i);
    }
    for i in 0..50 {
        let node = r.cpu.word(0x8019ce2c + i * 4);
        if node != 0 {
            let kind = r.cpu.word(node) >> 16;
            r.cpu.call(0x801356f4, &[node + 4, kind, i + 1]);
        }
    }
    r.sync();
    for tick in 1..=360 {
        let input = Input {
            accelerate: true,
            fire_guns: tick > 10,
            left: (24..40).contains(&tick),
            ..Default::default()
        };
        let fields = [3u8, 4, 5, 3][tick % 4];
        put(&mut r.cpu, 0x8017023c, tick as u32);
        r.cpu.put(0x80170238, &(fields as u16).to_le_bytes());
        r.cpu
            .put(0x8017023a, &(60u16 / fields as u16).to_le_bytes());
        r.cpu.call(0x800ffb98, &[2, input.ps1_buttons() as u32]);
        r.cpu.call(0x80110810, &[]);
        r.cpu.call(0x801344e0, &[0x800d8000, 0]);
        r.cpu.call(0x801333d0, &[0x800d8000, 0]);
        r.cpu.call(0x8010e3d8, &[]);
        r.cpu.call(0x80137d2c, &[]);
        r.cpu.call(0x8014ecd4, &[TAXI_CS, PLAYER]);
        r.cpu.call(0x8014ee14, &[POLICE_CS, POLICE]);
        r.cpu.call(0x80135784, &[]);
        r.scene.step(&mut r.player, &mut r.police, input, fields);
        if r.player.rng_state() != r.cpu.random_state {
            eprintln!("first RNG mismatch at {tick}");
        }
        r.compare();
    }
}
