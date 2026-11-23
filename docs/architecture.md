# Architecture

## Simulation

Rust drives the original C vehicle, collision, weapon and AI routines through the `tm1-core` crate. Keyboard input is sampled at simulation steps. The core uses integer arithmetic, a shared random sequence and the original field-based timing. Some effects also update game state, so the host preserves their position in the update order. The fixed-point lookup tables are compiled into Rust and shared with C. [Math tables](../crates/tm1-core/src/math/tables.rs) · [Imported source and revision](../crates/tm1-core/native/upstream/PROVENANCE.md)

## Disc resources

The asset reader follows the CUE, indexes the ISO9660 data track and reads the requested assets on demand. Font textures and palettes come from the disc. Glyph rectangles, spacing and vehicle labels are compiled into Rust. Opening an image compares the tracks and executable against the Japanese reference. A different revision can pass this check with warnings and still have incompatible assets. [Disc reader](../crates/tm1-assets/src/disc.rs) · [Font layout](../crates/tm1-assets/src/screen/font_layout.rs)

## Presentation

The battle renderer retains the scene hierarchy and source geometry. For each camera it orders arena, vehicle and effect primitives together, then submits batches through Bevy meshes and materials. The macOS build uses Metal. Menu screens use sprites and CPU-composed textures. CD audio, VAB effects and STR movies come from the disc, with built-in Rust video and XA audio decoders. [Rendering and audio evidence](status/presentation.md) · [Decoder provenance](../src/cinema/video/NOTICE.md)

## Verification

Disc-backed tests execute selected routines from the original Japanese executable and compare car state, timers, random state and effects at corresponding steps. [Reference scenarios](status/comparisons.md) · [Known differences](status/gaps.md)
