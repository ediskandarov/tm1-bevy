#[path = "support/mips.rs"]
mod mips;
use tm1_core::drive::{Input, Taxi};

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn drive_trace() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let mut cpu = mips::Cpu::new(&exe);
    const CAR: u32 = 0x8002_0000;
    // Skip platform audio and visual endpoints for the isolated dynamics run.
    for entry in [0x80108378, 0x801083b0, 0x801417d0, 0x801466b0, 0x80146930] {
        cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
    }
    for (timing, ramp) in [
        (&[3u8][..], false),
        (&[3, 4, 5, 3][..], false),
        (&[3u8][..], true),
    ] {
        let mut taxi = Taxi::from_executable(&exe).unwrap();
        if ramp {
            taxi.reset_at_test_ramp();
        }
        let mut peak = 0;
        let mut airborne = false;
        cpu.random_state = taxi.rng_state();
        cpu.put(CAR, taxi.state());
        for entry in [0x80125bc0, 0x8012cd90, 0x80130680, 0x80130a8c, 0x801312f8] {
            cpu.call(entry, &[]);
        }
        cpu.put(0x8018c3b4, &0u32.to_le_bytes());
        for step in 0..360 {
            let phase = step % 120;
            let accelerate = phase < 63;
            let reverse = (78..115).contains(&phase);
            let left = (20..53).contains(&phase);
            let right = (57..99).contains(&phase);
            let hard_turn = (43..94).contains(&phase);
            let fields = timing[step % timing.len()];
            let input = Input {
                accelerate,
                reverse,
                left,
                right,
                hard_turn,
                cycle_weapon: false,
                missile: step % 41 == 3,
                fire_guns: step % 13 < 6,
                turbo: (180..199).contains(&step),
            };
            let input = if ramp {
                Input {
                    accelerate: step < 100,
                    ..Default::default()
                }
            } else {
                input
            };
            cpu.call(0x800ffb98, &[2, u32::from(input.ps1_buttons())]);
            cpu.put(0x80170238, &(fields as u16).to_le_bytes());
            cpu.put(0x8017023a, &(60u16 / fields as u16).to_le_bytes());
            cpu.put(0x8017023c, &(step as u32 + 1).to_le_bytes());
            cpu.call(0x801125ac, &[CAR]);
            // Execute CarUpdate through the instruction following CarTransUpdate.
            cpu.call_until(0x8011125c, &[CAR], 0x801115c0);
            taxi.step(input, fields);
            peak = peak.max(taxi.position()[2]);
            airborne |= (0..4).all(|i| taxi.state()[0x273 + 0x44 * i] != 0);
            assert_eq!(cpu.random_state, taxi.rng_state());
            let start = (CAR & 0x1fff_ffff) as usize;
            for (offset, &byte) in taxi.state().iter().enumerate() {
                assert_eq!(
                    byte,
                    cpu.ram[start + offset],
                    "step {step}, player offset {offset:#x}"
                );
            }
        }
        println!(
            "timing {timing:?}, ramp {ramp}, 360 steps, max body height {peak}, all tires airborne {airborne}"
        );
        if ramp {
            assert!(peak > 96 && airborne);
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn arena_replay() {
    use tm1_core::drive::{CollisionScene, PoliceCar};
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let exe = disc.read("SIPS_600.07").unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
    let run = || {
        let mut scene = CollisionScene::from_dmd(&data, root(1), root(20), root(50)).unwrap();
        let mut taxi = Taxi::from_executable(&exe).unwrap();
        let mut ai = PoliceCar::spawn(&mut taxi);
        let mut hits = 0;
        let mut trace = Vec::new();
        for frame in 0..1200 {
            let p = frame % 160;
            scene.step(
                &mut taxi,
                &mut ai,
                Input {
                    fire_guns: true,
                    accelerate: p < 138,
                    reverse: p >= 145,
                    left: (95..115).contains(&p),
                    ..Default::default()
                },
                3,
            );
            hits += usize::from(taxi.state()[0x25c] != 0);
            trace.push((taxi.position(), ai.position(), taxi.rng_state()));
        }
        println!(
            "arena replay: {hits} contact steps; first {:?}; last {:?}",
            trace.first(),
            trace.last()
        );
        assert!(hits > 0);
        assert_ne!(trace.first().unwrap().1, trace.last().unwrap().1);
        trace
    };
    assert_eq!(run(), run());
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn world_projectile_contacts() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
    let mut scene =
        tm1_core::drive::CollisionScene::from_dmd(&data, root(1), root(20), root(50)).unwrap();
    let mut cpu = mips::Cpu::new(&disc.read("SIPS_600.07").unwrap());
    cpu.put(0x800188b8, &data);
    cpu.put(0x800b0104, &root(1).to_le_bytes());
    cpu.call(0x80103a4c, &[0x800b0400]);
    let mut hits = 0;
    let mut samples = 0;
    for x in (0..6400).step_by(110) {
        for y in (0..6400).step_by(122) {
            for z in [18, 73, 150] {
                let point = [x, y, z];
                let hit = scene.world_point(point);
                for (i, v) in point.iter().enumerate() {
                    cpu.put(0x800b0200 + 4 * i as u32, &v.to_le_bytes());
                }
                cpu.put(0x800b0240, &[0; 36]);
                cpu.put(0x800b024a, &(-1i16).to_le_bytes());
                cpu.call(0x80102d00, &[0x800b0000, 0x800b0200, 0x800b0240]);
                let tags = cpu.word(0x800b0248);
                let expected = [
                    cpu.word(0x800b0240) as i32,
                    cpu.word(0x800b0250) as i32,
                    (tags >> 16) as i16 as i32,
                    cpu.word(0x800b024c) as i16 as i32,
                ];
                assert_eq!(hit, expected, "point {point:?}");
                hits += usize::from(hit[0] != 0);
                samples += 1;
            }
        }
    }
    println!("{samples} original HDP world queries, {hits} solid contacts");
    assert!(hits > 100 && hits < samples);
}
