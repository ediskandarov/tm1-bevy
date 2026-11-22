use tm1_assets::{disc::Disc, scene::Dmd, texture::Vram};

#[test]
#[ignore = "requires player-supplied JP BIN/CUE; set TM1_CUE"]
fn supported_disc_arena_and_audio() {
    let disc = Disc::open(std::env::var_os("TM1_CUE").expect("set TM1_CUE")).unwrap();
    assert_eq!(disc.tracks.len(), 9);
    assert_eq!(disc.files.len(), 159);
    assert!(disc.read("missing.file").is_err());
    assert!(disc.audio_track(1).is_err());
    let audio = disc.audio_track(2).unwrap();
    assert_eq!(audio.len() as u64, disc.tracks[1].size - 150 * 2352);
    let d = disc.read("UADMD/XARENA1.DMD").unwrap();
    let db = Dmd::parse(&d).unwrap();
    let vram = Vram::from_tms(&disc.read("UADMD/XARENA1.TMS").unwrap()).unwrap();
    assert_eq!(db.roots.len(), 24);
    assert_eq!(vram.tim_count, 192);
    assert_eq!(vram.version, db.version);
    for name in [
        "XARENA1", "XARENA2", "WH1", "WH2", "XFWY1", "XFWY2", "PARK1", "PARK2", "SUBURB1",
        "SUBURB2", "ROOF1", "ROOF2",
    ] {
        let data = disc.read(&format!("UADMD/{name}.DMD")).unwrap();
        let level = Dmd::parse(&data).unwrap();
        let smoke = level.smoke_frames(false).unwrap();
        let steam = level.smoke_frames(true).unwrap();
        for (smoke, steam) in smoke.iter().zip(steam) {
            assert_eq!((smoke.texture.page >> 5) & 3, 2, "{name}");
            assert_eq!((steam.texture.page >> 5) & 3, 1, "{name}");
        }
        let available = level
            .special_bullet_frames()
            .unwrap()
            .map(|group| group.is_some());
        let expected = match name {
            "PARK1" | "XFWY1" => [true, true, false, false],
            "ROOF1" => [true, true, true, false],
            "ROOF2" | "XFWY2" => [false, false, true, true],
            "SUBURB1" => [true, false, true, true],
            "SUBURB2" => [true, true, false, true],
            _ => [true; 4],
        };
        assert_eq!(available, expected, "{name}");
        let debris = level.fragment_frames().unwrap();
        assert!(
            debris.iter().flatten().all(|frame| frame.width > 0),
            "{name}"
        );
        assert!(level.flame_frames().unwrap().iter().all(|f| f.width > 0));
        assert!(
            level
                .spark_frames()
                .unwrap()
                .iter()
                .all(|f| f.width > 0 && (f.texture.page >> 5) & 3 == 1)
        );
        if name.starts_with("ROOF") {
            let boss = level.vehicle_bodies_with_lights(130, false).unwrap();
            assert!(boss.iter().all(|stage| !stage.polygons.is_empty()));
        }
    }
    let frames = db.explosion_frames().unwrap();
    assert_eq!(frames[0].uv, [[64, 96], [95, 96], [64, 127], [95, 127]]);
    for frame in &frames {
        assert_eq!(frame.width, 31);
    }
    assert_eq!(db.contrail_frames().unwrap()[0].width, 63);
    for (id, polygons) in [(201, 35), (312, 8)] {
        assert_eq!(db.extract_kind(id).unwrap().polygons.len(), polygons);
    }
    for kind in (10..=120).step_by(10) {
        let [intact, damaged, critical] = db.vehicle_bodies(kind).unwrap();
        for stage in [&intact, &damaged, &critical] {
            assert!(
                stage.polygons.iter().any(|p| p.car_shadow),
                "missing shadow for {kind}"
            );
            let wheels = stage.polygons.iter().filter_map(|p| p.wheel);
            if kind == 40 {
                let pivots = wheels
                    .map(|wheel| {
                        let tm1_assets::scene::Wheel::Hammerhead { pivot, axis } = wheel else {
                            panic!("expected a rotating wheel");
                        };
                        assert_eq!(axis, [1., 0., 0.]);
                        pivot.map(|v| v as i32)
                    })
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(pivots.len(), 4);
            } else {
                // XARENA1 has no tire switches in Mr. Grimm's bike.
                assert_eq!(wheels.count() > 0, kind != 80);
            }
        }
        let rest = db.extract_kind(kind).unwrap();
        let visible: Vec<_> = intact
            .polygons
            .iter()
            .filter(|p| match p.wheel {
                Some(tm1_assets::scene::Wheel::Tire {
                    initial, branch, ..
                }) => initial == branch,
                _ => true,
            })
            .map(|p| p.source_offset)
            .collect();
        assert_eq!(
            visible,
            rest.polygons
                .iter()
                .map(|p| p.source_offset)
                .collect::<Vec<_>>()
        );
        assert_ne!(intact.object_ids, damaged.object_ids);
        assert_ne!(damaged.object_ids, critical.object_ids);
    }
    let scene = db.extract_kind(1).unwrap();
    assert_eq!(scene.object_ids.len(), 63);
    let intact = || {
        scene
            .polygons
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                scene.damage_branches.iter().all(|(ptr, branch, range)| {
                    let offset = (ptr - 0x800188b8) as usize;
                    let initial = if d[offset] == 1 {
                        u8::from(d[offset + 20] == 0)
                    } else {
                        d[offset + 7]
                    };
                    !range.contains(i) || *branch == initial
                })
            })
            .map(|(_, poly)| poly)
    };
    assert_eq!(intact().filter(|p| p.billboard.is_none()).count(), 1273);
    assert_eq!(intact().filter(|p| p.billboard.is_some()).count(), 8);
    assert!(scene.omitted_nodes.is_empty());
    let show = |step| {
        scene
            .polygons
            .iter()
            .filter(|p| {
                !p.animation.is_empty() && p.animation.iter().all(|f| f.selected(step) == f.child)
            })
            .map(|p| p.source_offset)
            .collect::<Vec<_>>()
    };
    assert_eq!(show(0).len(), 3);
    assert_eq!(show(16).len(), 3);
    assert_eq!(show(0), show(15));
    assert_ne!(show(15), show(16));
    assert_eq!(show(0), show(32));
    let keys: std::collections::BTreeSet<_> =
        scene.polygons.iter().filter_map(|p| p.texture).collect();
    let intact_keys: std::collections::BTreeSet<_> = intact().filter_map(|p| p.texture).collect();
    assert_eq!(intact_keys.len(), 61);
    assert_eq!(keys.len(), 62);
    for key in keys {
        for y in 0..=255 {
            for x in 0..=255 {
                vram.sample(key, x, y).unwrap();
            }
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn ending_assets() {
    let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let fonts = tm1_assets::screen::Fonts::load(&disc).unwrap();
    for name in ["SCRNCALY", "SCRNCOP", "SCRNLARO", "SCRNROAD"] {
        let mut pixels =
            tm1_assets::screen::background(&disc.read(&format!("UASCREEN/{name}.TIM")).unwrap())
                .unwrap();
        let before = pixels.clone();
        fonts.centered(
            &mut pixels,
            tm1_assets::screen::Face::Text,
            70,
            "CONGRATULATIONS!",
        );
        assert_ne!(before.as_slice(), pixels.as_slice());
    }
    let data = disc.read("UADMD/CARSEND.DMD").unwrap();
    let db = Dmd::parse(&data).unwrap();
    for kind in (10..=120).step_by(10) {
        assert!(!db.extract_kind(kind).unwrap().polygons.is_empty());
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn vab_predictors_match_pcm_reference() {
    use sha2::{Digest, Sha256};
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let bank = disc.read("SND/UACORE.VAB").unwrap();
    // Independent signed-integer decoder, low nibble first; hash little-endian PCM.
    for (id, frames, digest) in [
        (
            0,
            44800,
            "60ebb79e4efa1a04b4b2055b1fd08eebbb36788479a06f1a8479c708f3c845cf",
        ),
        (
            13,
            31080,
            "e5c1e6b0d56f02d21d6e8ff7c791fee1165979a98122b838c5af4a082edddb0f",
        ),
        (
            44,
            51856,
            "e045f83d502498c3a7bed2a74367beb565faced7ac59725a7650035433ed36a1",
        ),
    ] {
        let tones = tm1_assets::sound::program(&bank, id).unwrap();
        let pcm = &tones[0].pcm;
        assert_eq!(pcm.len(), frames);
        let mut hash = Sha256::new();
        for sample in pcm {
            hash.update(sample.to_le_bytes());
        }
        assert_eq!(format!("{:x}", hash.finalize()), digest, "program {id}");
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn repeated_vab_blocks_keep_predictor_history() {
    use sha2::{Digest, Sha256};
    let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let bank = disc.read("SND/UACORE.VAB").unwrap();
    // Hash of a second pass through the compressed blocks with the decoder
    // history carried from the end of the first pass.
    for (program, expected) in [
        (
            0,
            "b75d7a244c461fab49c62caf1bc629a849a71c255fad6fc723133d6c8ac80a98",
        ),
        (
            1,
            "caa7a1022f3eb1bacd3c42657fabe580078183a318093f1f2d7f1563e9835eab",
        ),
        (
            4,
            "9b13358a10bd761663ffe17f9520a524d12e9ede97236e84c190128d0cb5edfa",
        ),
        (
            9,
            "c46215fcd60bc254ba3f83b09c00acce6b7c629d18449b73228d9a765c364860",
        ),
        (
            12,
            "d784f46014771dad16f14867ad6174d2f95c781fff6e97fb1cf3a62cc82cf371",
        ),
    ] {
        let mut tone = tm1_assets::sound::program(&bank, program)
            .unwrap()
            .remove(0);
        let first_end = tone.pcm.len();
        let period = first_end - tone.loop_start.unwrap();
        tone.prepare_loop();
        let start = tone.loop_start.unwrap();
        assert_eq!(tone.pcm.len() - start, period);
        let mut repeated = tone.pcm[first_end..]
            .iter()
            .chain(tone.pcm[start..].iter().cycle());
        for _ in 0..2 {
            let mut hash = Sha256::new();
            for sample in repeated.by_ref().take(period) {
                hash.update(sample.to_le_bytes());
            }
            assert_eq!(
                format!("{:x}", hash.finalize()),
                expected,
                "program {program}"
            );
        }
    }
}
