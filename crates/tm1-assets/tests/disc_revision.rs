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
        "XARENA1", "WH1", "WH2", "XFWY1", "XFWY2", "PARK1", "PARK2", "SUBURB1", "SUBURB2", "ROOF1",
        "ROOF2",
    ] {
        let data = disc.read(&format!("UADMD/{name}.DMD")).unwrap();
        let level = Dmd::parse(&data).unwrap();
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
        assert_eq!(intact.object_ids, db.extract_kind(kind).unwrap().object_ids);
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
