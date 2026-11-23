#[path = "support/mips.rs"]
mod mips;
#[path = "support/targets.rs"]
mod targets;
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
    use tm1_core::drive::Vehicle;
    let mut first_layout_motion = Vec::new();
    for (vehicle, timing, ramp, layout) in [
        (Vehicle::YellowJacket, &[3u8][..], false, 1u8),
        (Vehicle::YellowJacket, &[3u8][..], true, 1),
    ]
    .into_iter()
    .chain(
        Vehicle::ROSTER
            .into_iter()
            .flat_map(|car| (0..4).map(move |layout| (car, &[3, 4, 5, 3][..], false, layout))),
    ) {
        let mut taxi = Taxi::for_vehicle(vehicle);
        // Construct the retail stat block independently from a cleared player slot.
        let stats = CAR + 0x138;
        cpu.put(stats, &[0; 0x120]);
        cpu.put(stats, &[1]);
        for off in [0x38, 0x4c] {
            cpu.put(stats + off, &1i32.to_le_bytes());
        }
        cpu.put(0x80170238, &3u16.to_le_bytes());
        cpu.put(0x8017023a, &20u16.to_le_bytes());
        cpu.call(0x80124d04, &[stats, 100]);
        cpu.call(0x80123310, &[stats, vehicle as u32]);
        let health = cpu.word(stats + 0x44);
        cpu.put(stats + 0x40, &health.to_le_bytes());
        for (offset, &v) in taxi.state()[0x138..0x258].iter().enumerate() {
            assert_eq!(
                v,
                cpu.ram[(stats as usize & 0x1fffff) + offset],
                "{vehicle:?} stats+{offset:#x}"
            );
        }
        if ramp {
            taxi.reset_at_test_ramp();
        }
        let mut peak = 0;
        let mut airborne = false;
        cpu.random_state = taxi.rng_state();
        cpu.put(CAR, taxi.state());
        cpu.call(0x80150318, &[CAR, u32::from(layout)]);
        taxi.set_pad_layout(layout);
        // Ask the JP input reader which physical button activates each action.
        let bindings = [29, 25, 27, 28, 31, 33, 34, 35, 36, 37].map(|offset| {
            let button = cpu.ram[0x20000 + 0xc8 + offset] as u32;
            (0..16)
                .map(|bit| 1u32 << bit)
                .find(|mask| {
                    cpu.call(0x800ffb98, &[2, *mask]);
                    cpu.call(0x800ffbe4, &[button, 0]) != 0
                })
                .unwrap()
        });
        for entry in [0x80125bc0, 0x8012cd90, 0x80130680, 0x80130a8c, 0x801312f8] {
            cpu.call(entry, &[]);
        }
        cpu.put(0x8018c3b4, &0u32.to_le_bytes());
        if layout == 0 {
            first_layout_motion.clear();
        }
        let steps = if ramp { 360 } else { 600 };
        let mut forward_speed = false;
        let mut backward_speed = false;
        let mut braking_turn = false;
        for step in 0..steps {
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
                cycle_weapon: step % 33 == 7,
                previous_weapon: step % 47 == 11,
                missile: step % 41 == 3,
                fire_guns: step % 13 < 6,
                turbo: (180..199).contains(&step),
            };
            let input = if ramp {
                Input {
                    accelerate: step < 100,
                    ..Default::default()
                }
            } else if step >= 360 {
                Input {
                    accelerate: step < 420 || step >= 550,
                    reverse: (420..560).contains(&step),
                    right: (420..430).contains(&step),
                    hard_turn: (420..430).contains(&step),
                    ..Default::default()
                }
            } else {
                input
            };
            let pressed = [
                input.accelerate,
                input.reverse,
                input.left,
                input.right,
                input.hard_turn,
                input.turbo,
                input.fire_guns,
                input.missile,
                input.cycle_weapon,
                input.previous_weapon,
            ];
            let buttons = bindings
                .iter()
                .zip(pressed)
                .filter(|(_, down)| *down)
                .fold(0, |bits, (mask, _)| bits | mask);
            cpu.call(0x800ffb98, &[2, buttons]);
            cpu.put(0x80170238, &(fields as u16).to_le_bytes());
            cpu.put(0x8017023a, &(60u16 / fields as u16).to_le_bytes());
            cpu.put(0x8017023c, &(step as u32 + 1).to_le_bytes());
            cpu.call(0x801125ac, &[CAR]);
            // Execute CarUpdate through the instruction following CarTransUpdate.
            cpu.call_until(0x8011125c, &[CAR], 0x801115c0);
            taxi.step(input, fields);
            if timing.len() > 1 {
                let motion = (taxi.position(), taxi.heading(), taxi.speed_mph(fields));
                if layout == 0 {
                    first_layout_motion.push(motion);
                } else {
                    assert_eq!(
                        motion, first_layout_motion[step],
                        "{vehicle:?} layout {layout} step {step}"
                    );
                }
            }
            if step >= 360 {
                forward_speed |= step < 420 && taxi.speed_mph(fields) > 0;
                backward_speed |= step >= 430 && taxi.speed_mph(fields) < 0;
                braking_turn |= taxi.state()[0xc8 + 0x10] != 0;
            }
            peak = peak.max(taxi.position()[2]);
            airborne |= (0..4).all(|i| taxi.state()[0x273 + 0x44 * i] != 0);
            assert_eq!(cpu.random_state, taxi.rng_state());
            let start = (CAR & 0x1fff_ffff) as usize;
            for (offset, &byte) in taxi.state().iter().enumerate() {
                assert_eq!(
                    byte,
                    cpu.ram[start + offset],
                    "{vehicle:?} layout {layout} step {step}, player offset {offset:#x}"
                );
            }
        }
        println!(
            "{vehicle:?}, timing {timing:?}, ramp {ramp}, layout {layout}, {steps} steps, max body height {peak}, all tires airborne {airborne}"
        );
        if ramp {
            assert!(peak > 96 && airborne);
        } else {
            assert!(
                forward_speed && backward_speed && braking_turn,
                "{vehicle:?} layout {layout}: forward {forward_speed}, reverse {backward_speed}, braking turn {braking_turn}"
            );
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn arena_replay() {
    use tm1_core::drive::{CollisionScene, PoliceCar};
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let data = disc.read("UADMD/XARENA1.DMD").unwrap();
    let roots = tm1_assets::scene::Dmd::parse(&data).unwrap().roots;
    let root = |id| roots.iter().find(|r| r.1 == id).unwrap().0;
    let run = || {
        let mut scene = CollisionScene::from_dmd(&data, root(1), root(20), root(50)).unwrap();
        let mut taxi = Taxi::for_vehicle(tm1_core::drive::Vehicle::YellowJacket);
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
    targets::register_retail_targets(&mut cpu, &roots);
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
