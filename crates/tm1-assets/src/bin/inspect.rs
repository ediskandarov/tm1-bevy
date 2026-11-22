use anyhow::{Context, Result, ensure};
use tm1_assets::{disc::Disc, scene::Dmd, texture::Vram};
fn main() -> Result<()> {
    let cue = std::env::args_os()
        .nth(1)
        .context("usage: inspect <disc.cue>")?;
    let disc = Disc::open(cue)?;
    for t in &disc.tracks {
        println!(
            "track {:02}: {} bytes, {}, sha256 {}",
            t.number, t.size, t.mode, t.sha256
        );
    }
    println!("ISO files: {}", disc.files.len());
    let dmd = disc.read("UADMD/XARENA1.DMD")?;
    let tms = disc.read("UADMD/XARENA1.TMS")?;
    let db = Dmd::parse(&dmd)?;
    let vram = Vram::from_tms(&tms)?;
    ensure!(db.version == vram.version, "DMD/TMS version mismatch");
    println!(
        "DMD: {} bytes, {} roots; TMS: {} bytes, {} TIM uploads",
        dmd.len(),
        db.roots.len(),
        tms.len(),
        vram.tim_count
    );
    let scene = db.extract_kind(1)?;
    println!(
        "arena root {:#x}: {} objects, {} polygons",
        scene.root_id,
        scene.object_ids.len(),
        scene.polygons.len()
    );
    let keys: std::collections::BTreeSet<_> =
        scene.polygons.iter().filter_map(|p| p.texture).collect();
    for key in &keys {
        for y in 0..=255 {
            for x in 0..=255 {
                vram.sample(*key, x, y)?;
            }
        }
    }
    println!("decoded {} texture/CLUT/window combinations", keys.len());
    for omitted in &scene.omitted_nodes {
        println!("OMITTED: {omitted}");
    }
    println!("Static asset inspection only; gameplay and renderer not validated.");
    Ok(())
}
