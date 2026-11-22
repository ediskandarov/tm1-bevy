use tm1_assets::{disc::Disc, scene::Dmd};
fn main() -> anyhow::Result<()> {
    let disc = Disc::open(std::env::args_os().nth(1).unwrap())?;
    for path in disc.files.keys().filter(|p| p.ends_with(".DMD")) {
        let data = disc.read(path)?;
        let db = Dmd::parse(&data)?;
        println!(
            "{path}: {:?}",
            db.roots.iter().map(|r| r.1).collect::<Vec<_>>()
        );
        if path.contains("ARENA1") {
            for kind in [1, 3, 20, 50] {
                if let Ok(scene) = db.extract_kind(kind) {
                    let mut lo = [f32::INFINITY; 3];
                    let mut hi = [f32::NEG_INFINITY; 3];
                    for v in scene.polygons.iter().flat_map(|p| &p.vertices) {
                        for i in 0..3 {
                            lo[i] = lo[i].min(v.position[i]);
                            hi[i] = hi[i].max(v.position[i]);
                        }
                    }
                    println!("bounds {lo:?} .. {hi:?}");
                    println!(
                        "kind {kind}: {} objects, {} polygons, {} materials",
                        scene.object_ids.len(),
                        scene.polygons.len(),
                        scene
                            .polygons
                            .iter()
                            .filter_map(|p| p.texture)
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    );
                }
            }
        }
    }
    Ok(())
}
