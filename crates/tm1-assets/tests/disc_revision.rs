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
    for kind in [10, 20, 30, 40, 50, 60, 80, 90] {
        let [intact, damaged, critical] = db.vehicle_bodies(kind).unwrap();
        assert_eq!(intact.object_ids, db.extract_kind(kind).unwrap().object_ids);
        assert_ne!(intact.object_ids, damaged.object_ids);
        assert_ne!(damaged.object_ids, critical.object_ids);
    }
    let scene = db.extract_kind(1).unwrap();
    assert_eq!(scene.object_ids.len(), 53);
    assert_eq!(
        scene
            .polygons
            .iter()
            .filter(|p| p.billboard.is_none())
            .count(),
        1273
    );
    assert_eq!(
        scene
            .polygons
            .iter()
            .filter(|p| p.billboard.is_some())
            .count(),
        8
    );
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
    assert_eq!(keys.len(), 61);
    for key in keys {
        for y in 0..=255 {
            for x in 0..=255 {
                vram.sample(key, x, y).unwrap();
            }
        }
    }
}
