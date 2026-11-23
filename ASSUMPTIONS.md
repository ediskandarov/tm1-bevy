# Assumptions and Architecture Validation

The hypotheses below record the initial position. Incremental implementation
evidence and current validation limits are tracked in [STATUS.md](STATUS.md);
the original hypotheses remain here for later evaluation.

## Results for the first arena

Keeping the C dynamics and using Rust for the host preserved the sampled retail updates. The differential fixtures caught concrete integration errors in ammo pickup, per-step shadow state, and RNG consumption by flash effects. Full layout comparison covers bytes whose source names are still unknown. It also requires the same initial state and the same caller sequence.

The taxi and police entities reuse source vertices, topology, colors, atlas UVs and vehicle poses. Each view computes opaque vehicle face depths from the current pose. An initial experiment drawing the taxi with a separate camera lost occlusion against walls, police and particles. The gameplay renderer now draws these elements with one perspective camera and depth buffer. The renderer sorts translucent packets within each material by ordering-table bucket, pass and insertion order. The experiment has not tested a browser backend.

See [acceptance runs and validation limits](STATUS.md) for the original reference, observed Metal timings and audio differences.

## Source observations

The `seems-ready` working directory was inspected on 2026-11-22. This was a
limited review, not a complete audit; see README for uncommitted changes.

- A search in `src/tm1` found no `INCLUDE_ASM`/`INCLUDE_RODATA`.
  This does not establish complete portable C coverage or replace a matching build.
- A subsequent audit found hand-written MIPS `geomProc` in the source
  configuration. It lies outside the earlier `src/tm1` macro search and must
  be replaced explicitly for this port.
- `RT.c` interleaves game updates and rendering preparation, calls `VSync`,
  and uses `FrameTimeToUpdateRate`.
- `TIMER.c` computes `gFieldsLastFrame`; `CARUPD.c` uses it directly in arithmetic.
  One three-field step must not be assumed equivalent to three one-field steps
  without verification.
- `DB.c` uses address `0x800188B8` and interprets values in the data as pointers.
- `HIER.c`, `HIER_SUB.c`, and `VIEW.c` handle matrices, geometry, and insertion
  of primitives into the ordering table.
- `SOUND.c` uses VAB, SPU voices, pitch, noise, reverb, and CD audio.
- `FILEIO.c`, `SCREEN.c`, and `MOVIE.c` contain waits/loops tied to the PS1 SDK.

## Proposed boundary

Original game logic retains ownership of simulation state. Bevy invokes it in
an explicit order, supplies input and controlled game time, and receives data
for rendering and audio. Initially, the game world is not split into
independently scheduled ECS systems.

The FFI uses explicit integer types, buffer lengths, and ownership rules.
Pointers and structures from the disc image do not directly become host
structures. PS1 addresses are converted to validated offsets/IDs or a separate
in-memory representation; the loader audit determines the choice.

The platform layer covers only operations actually needed. Individual SDK/GTE/SPU
replacements may be exact or high-level depending on their effect on gameplay.
Full CPU/PS1 emulation is not the selected execution approach.

## Hypothesis register

| ID | Hypothesis | Check and reason to reconsider |
| --- | --- | --- |
| A1 | The C core can be retained with limited adaptation | Build a representative loading and update path on macOS; measure the extent of ABI, pointer, SDK, and undefined-behavior changes |
| A2 | A primitive backend will deliver the first arena sooner than a full scene migration | Render actual geometry, textures, and effects; reconsider if this requires excessive GPU/VRAM emulation |
| A3 | Scene semantics for Bevy can be extracted from code/assets | Obtain a mesh, UVs, material, transform, and stable ID for one object before projection; missing data implies deeper changes to the geometry path |
| A4 | Physics can be preserved while separating game clocks from presentation | Compare state step by step, including variable step sizes; check rendering side effects and RNG calls |
| A5 | Assets from the selected JP revision are fully accessible through BIN/CUE | Inventory tracks, files, banks, and formats; verify address relocation against actual data |
| A6 | High-level audio can preserve required behavior | Check voice allocation, key status, loops, pitch, and durations; gameplay dependencies may require a more accurate voice model |
| A7 | The architecture can be ported to browsers | Later build a minimal C/Rust/WASM integration and exercise loading, a simulation step, and a frame; macOS alone does not prove this |
| A8 | macOS on Apple Silicon is sufficient for the first target | Test on a specific Mac and record its model, macOS version, and backend; Intel support is not implied |

## Path to regular Bevy rendering

Here, "Bevy primitives" means meshes, materials, transforms, and entities;
replacing game models with cubes or spheres is not required. Geometry comes
from the disc.

### Stage 1. Graphics command adapter

Decode the required PS1 primitives, texture pages/CLUT, and drawing state into
typed commands for the custom renderer. Initially support original drawing
order and preserve source colors and textures. Higher resolution is acceptable,
but it cannot recover precision already lost through coordinate rounding.

Commands should use logical texture/material references instead of spreading
raw packet formats throughout the application. Attach the source object ID
when available. Do not invent missing depth or object ownership from screen
coordinates.

### Stage 2. Scene extraction before projection

At asset loading and hierarchy traversal, obtain:

- Source vertices and topology, UVs, colors, and material parameters.
- Object world transforms and hierarchy, camera pose and parameters.
- Stable model and instance IDs.
- Dynamic geometry, visibility, and effects needed for the selected object.

Intercepting `DrawOTag` alone is insufficient: final PS1 packets may already
contain projected and rounded coordinates without source depth or scene
structure. Perspective-correct textures, a depth buffer, lighting, and shadows
require moving extraction earlier in the graphics pipeline.

### Stage 3. One-object validation

Move one vehicle or static arena object to a Bevy mesh/material. Keep other
objects on the primitive backend and allow switching the selected object's
rendering path.

Mixed rendering requires an explicit solution: original ordering tables and
a new depth buffer do not automatically produce correct shared ordering.
Check occlusion by walls, effects, and transparency. If depth cannot be obtained
for the remaining scene, compare separate frames rather than declaring the
mixed renderer complete.

Success means the object receives state from the same core, correctly uses
disc assets, renders through a mesh/material, and leaves the simulation trace
unchanged. Record the extent of additional changes and retained PS1-specific code.

### Stage 4. Remaining object classes

After validation, migrate the arena, vehicles, effects, and HUD separately.
An unlit or vertex-color material is acceptable initially. Lighting and PBR
require checking or generating normals and missing material parameters;
original textures may contain baked lighting and shadows.

Complete replacement of the primitive backend is a later goal, not an automatic
outcome promised by the first arena. A high-level scene representation should
make this migration measurable and incremental.

## Open decisions before implementation

- Exact image and hashes; arena, vehicle, opponent, and weapon set.
- Bevy/Rust/C toolchain versions and minimum macOS version.
- Reference emulator, trace collection, and state normalization methods.
- Simulation time-step policy: first replay recorded steps from the original,
  then separately validate the chosen real-time policy.
- Supported gamepad and button mapping.
- Minimum SPU model fidelity needed for effects and voice state.
- Pause/focus-loss behavior and measurable performance targets.

## Evaluating the approach after the experiment

For each hypothesis A1–A8, record whether it is supported in the stated scenarios,
refuted, or still untested. Include a reproducible command, versions, inputs,
and results. List discrepancies and required architecture changes separately.

Evaluate:

- How much original logic was retained and where semantic adaptation was necessary.
- Which simulation discrepancies were found and how they were resolved.
- Time and complexity spent on the disc, arithmetic, timing, rendering, and audio.
- CPU/GPU time, memory, and game-clock stability.
- What the primitive adapter contributed to the first arena and how much work
  was repeated when migrating one object to a mesh/material.
- Which data had to be extracted before projection and whether it is sufficient
  for the next object class.
- Which browser assumptions were actually tested and which remain architectural
  intentions only.

The outcome may support retaining C while also refuting the value of the
intermediate renderer. Evaluate these decisions independently.

(This text was generated via LLM.)
