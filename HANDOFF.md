# Console Agent Handoff — 2026-11-22

This is the historical handoff. See [STATUS.md](STATUS.md) for subsequent progress.

## User intent and working rules

Continue the TM1 → Bevy experiment until the agreed playable-arena scope is
complete. The user explicitly requested sustained implementation, not only a
plan or asset viewer. This handoff is a transition from a restricted ChatGPT
Work session to a console Codex session; it does not change the goal.

- Reply to the user in Russian. Maintain project documentation in English.
- Work in this separate repository; preserve the original decompilation repo.
- Keep assumptions, requirements, implementation evidence, and limitations
  current so the architecture can be evaluated after the experiment.
- Do not substitute approximate physics or call a static preview a playable port.
- Do not repeatedly ask permission for already-authorized implementation work.
  Request execution approval when the environment actually requires it.
- No commits or PRs were requested. Do not commit automatically.
- No parallel agents were requested. Work without delegating unless authorized.
- Read applicable AGENTS.md files. Source-repo instructions prohibit generated
  assembly/report edits and automatic commits. The user explicitly authorized
  English documentation in this new repo.

Read `REQUIREMENTS.md`, `ASSUMPTIONS.md`, and `STATUS.md` after this handoff.

## Agreed scope

- First platform: macOS. Available machine: Apple M3 Pro, 18 GPU cores.
- One playable arena, one player vehicle, at least one original AI opponent,
  collisions, damage, weapons, effects, music, restart/pause/focus handling,
  keyboard and one tested controller. See requirements for acceptance details.
- Keep original C where useful; Rust/Bevy owns the platform layer.
- Player supplies one exact JP revision as BIN/CUE. Read resources from it.
  Do not bundle disc assets, original executable, or BIOS.
- Preserve original physics, collision, AI, weapon algorithms, integer behavior,
  RNG, timing, and update order. Compare original and port state per step with
  identical initial state, inputs, RNG, and time-step sequence.
- Initial graphics target: custom GPU renderer of original primitives inside
  Bevy. Pixel-perfect appearance is unnecessary; modern presentation is welcome.
- The user particularly wants an incremental path to regular Bevy meshes,
  materials, transforms, and entities. Preserve geometry and scene identity
  before projection; final PS1 packets alone lose necessary information.
- Browser is a later direction, outside first macOS acceptance. Full menus,
  campaign, cinematics, and split-screen are outside the first arena.

## Repositories and source provenance

| Item | Location / value |
| --- | --- |
| Workspace | `/Users/eduard/sandbox/tm1` |
| New port repo | `/Users/eduard/sandbox/tm1/tm1-bevy` |
| Source main repo | `/Users/eduard/sandbox/tm1/tm1_decomp` |
| Source branch worktree | `/Users/eduard/sandbox/tm1/worktrees/match-weapon` |
| Selected source branch | `seems-ready` |
| Pinned baseline | `fb8f9dd4c3aff275893c2573c432a94f644e9970` |
| Source upstream | `https://github.com/ediskandarov/tm1_decomp` |

The new repo is on `main`, with no commits or remote configured. All current
files are untracked. Preserve them; this is the accumulated implementation.

The source branch worktree already had uncommitted changes in
`config/JP/SIPS_600.07.yaml`, `config/JP/report.json`, `src/tm1/SOUND.c`, and
the `tools/maspsx` submodule. Do not overwrite them or silently import them.
Use `git show <pinned-commit>:<path>` for source imports. Imported `TIMER.c`
and `MATH.c` are unchanged baseline files, with the MIT notice retained.

## Disc and selected resources

Local CUE:

```text
/Users/eduard/sandbox/tm1/tm1_decomp/disks/Twisted Metal (Japan)/Twisted Metal (Japan).cue
```

- Nine separate BIN files: Track 1 MODE2/2352, Tracks 2–9 AUDIO.
- All track hashes and sizes are in `data/jp-disc.tsv`.
- Executable: `SIPS_600.07`, 612352 bytes; SHA-256
  `552dbdb7afbcdaced9be90074036d59a2c3eb6fe31ade623679842346b3ffa36`.
- Track 1 SHA-256:
  `c96fef832c03b651f809360343822b1af0ce108bd735c1ce84c4ea7255839758`.
- ISO contains 159 files. Preview arena resources are `UADMD/XARENA1.DMD`
  and `UADMD/XARENA1.TMS`; sound bank is `SND/UACORE.VAB`.
- DMD has 24 roots; environment root kind 1 at `0x8009f7a0`.
  TMS contains 192 TIM uploads. DMD address base is `0x800188b8`.
- The arena preview choice is implemented. Player/opponent/weapon acceptance
  scenarios have not been selected. Make reasonable concrete choices and record them.

## What exists and what has passed

### Build

Bevy is pinned to **0.20.1**, as the user asked for an update. Rust is pinned
to **1.97.1** in `rust-toolchain.toml`; Bevy/WESL require it. The user installed
the toolchain and fetched dependencies through ordinary Terminal.

`cargo check --locked --offline` and `cargo build --locked --offline` passed.
Two Bevy API issues were fixed: disambiguating `anyhow::Context` from Bevy's
trait and binding the mutable asset handle as `mut`.

Root dependencies enable the minimal 2D renderer. Audio/gamepad features are
not enabled or fetched. Root and each standalone crate have separate lockfiles;
the inner crates use their own `[workspace]` declarations intentionally.

### Assets (`crates/tm1-assets`)

- Strict supported-layout CUE parsing, path checks, all-track SHA-256 checks,
  Mode 2 Form 1 sector reads, bounded ISO traversal, executable verification.
- DMD pointers are checked offsets, not native pointer casts.
- Static scene extraction retains source object IDs and polygon offsets.
  Current subset: 42 object visits, 754 polygons, 57 texture keys.
- TMS/TIM uploads into 1024×512 CPU VRAM; CLUT, texture window, transparent-zero
  decoding. Texture bit-15 semi-transparency behavior is incomplete.
- Raw CDDA track bytes are accessible after INDEX 01. Playback is absent.
- Six asset unit tests and one opt-in real-disc test passed earlier.
- Static traversal omits rotation nodes, sprites, and visibility controllers;
  uses nearest LOD and initial animation/switch selections.

### Core (`crates/tm1-core`)

- `TIMER.c`: unchanged C, controlled host timer harness, two tests passed.
  This is not integrated gameplay or hardware interrupt emulation.
- `MATH.c`: unchanged C, explicit 32-bit host types and ABI size assertions;
  compiled with `-fwrapv`. Rust exposes matrix operations and checked normalize.
- `SquareRoot0` is reconstructed from the SDK algorithm. Its 192 table values
  equal `isqrt((index + 64) << 18)`; no original table is distributed.
- `tests/support/mips.rs` is a **test-only instruction subset**, not a PS1
  runtime. It loads executable bytes from the verified player disc and executes
  original functions, with branch/load delay slots. Unknown instructions fail.
- Differential test passed: **8192 matrix cases, 4096 normalization cases,
  4444 square-root cases**, deterministic seed `0x544d3101`. A synthetic CPU
  branch/load-delay test also passed.
- Normalize rejects vectors whose true squared norm exceeds `i32::MAX`.
  Original negative `SquareRoot0` inputs can read before the SDK table; that
  behavior is not ported. The exported SDK shim currently aborts for such input.
  Resolve the necessary domain before wiring arbitrary game callers into it.
- These checks establish sampled arithmetic agreement, **not game physics
  equivalence**. There is no vehicle simulation, collision, AI, or weapon loop.
- `clang-format` was not found. New compatibility headers have not been
  clang-formatted; original imported C must remain byte-identical.

### Frontend (`src`)

- CPU perspective projection and average-depth painter ordering; GPU triangles
  through Bevy Mesh2d/ColorMaterial, using a texture atlas of decoded resources.
- Orbit preview controls: arrow keys, W/S zoom, Escape quit.
- Window title explicitly says resource preview, not gameplay.
- `--screenshot <path.png>` requests GPU readback after 120 frames and exits.
  The parent output directory must already exist. No screenshot was obtained.
- No validated original ordering table, near-plane clipping, backface culling,
  semi-transparency modes, full hierarchy, animation, or gameplay HUD.
- Current world-axis mapping is `[x, -y, z]`. **Recheck it:** `CARUPD.c` uses
  `pos.vz` for height and `pos.vx/vy` for ground coordinates. Visual validation
  has not happened; do not assume the preview's axis mapping is correct.

## Environment issue that caused the handoff

The previous agent ran inside a ChatGPT Work session with local shell tools,
not the ordinary console Codex permissions workflow. Its permissions were
workspace-write, restricted network, with sandbox escalation auto-rejected.

- Ordinary user Terminal could access crates.io; the agent shell could not.
  This was not a crates.io outage. Fetching in Terminal populated the cache.
- The compiled Bevy app failed at device selection: `Unable to find a GPU`.
- `system_profiler SPDisplaysDataType` reported Apple M3 Pro with Metal support.
- A minimal Objective-C Metal probe **in the same restricted shell** returned
  `Metal devices: 0` and `Default device: nil`, independently of Bevy.
- This establishes the process-access discrepancy, not the specific sandbox
  rule responsible. Do not recommend installing GPU drivers or changing Bevy
  merely on this evidence.
- Docker socket access was also denied in that session.
- The user opened a console agent to get a normal execution/approval path.
  Check your actual permissions; do not assume those restrictions persist.
  If needed, request approval for the specific GPU smoke run.

Do not use UI automation or another tool to evade a sandbox denial. Do not
alter global security configuration or relaunch an unrestricted agent yourself.

## Reproducible next commands

Run from the port repo. No copyrighted files should be copied into tracked paths.

```sh
cd /Users/eduard/sandbox/tm1/tm1-bevy
rustc --version
cargo check --locked --offline
cargo test --locked --offline --manifest-path crates/tm1-assets/Cargo.toml
cargo test --locked --offline --manifest-path crates/tm1-core/Cargo.toml
TM1_CUE='/Users/eduard/sandbox/tm1/tm1_decomp/disks/Twisted Metal (Japan)/Twisted Metal (Japan).cue' cargo test --locked --offline --manifest-path crates/tm1-core/Cargo.toml --test math_original -- --include-ignored --nocapture
TM1_CUE='/Users/eduard/sandbox/tm1/tm1_decomp/disks/Twisted Metal (Japan)/Twisted Metal (Japan).cue' cargo test --locked --offline --manifest-path crates/tm1-assets/Cargo.toml --test disc_revision -- --ignored
mkdir -p local
cargo run --locked --offline -- '/Users/eduard/sandbox/tm1/tm1_decomp/disks/Twisted Metal (Japan)/Twisted Metal (Japan).cue' --screenshot local/arena-preview.png
```

Only rerun checks useful to establish your current baseline; do not repeatedly
run complete inherited suites after unrelated small changes. Keep screenshots
and other asset-derived artifacts in ignored `local/`.

## Important source findings for the remaining implementation

- About 38k lines in 43 game C modules. There are no `INCLUDE_ASM` macros under
  `src/tm1`, but **`geomProc` is handwritten MIPS** in `asm/tm1/geom.s` and
  the source configuration. It must be replaced explicitly.
- Actual game-car dynamics are in `CAR.c`, `CARUPD.c`, `CAR_INIT.c`, plus
  `AI_CAR.c`, `AICARUPD.c`, `AICARINI.c`, and environment interaction modules.
  Do not confuse the generic `VEHICLE.c` controls with complete car physics.
- `RT.c` interleaves missile/effect updates, view, bullets, explosions, HUD,
  hierarchy, and `PlayGame`. Rendering preparation may mutate shared state.
- Timer rounds elapsed ticks using 70618 ticks/field, uses 3 fields when the
  rounded result is below 1, caps at 5, and sets update rate to `60 / fields`.
  The main loop waits for update rate <= 20. Do not introduce arbitrary 60 Hz
  physics or split one three-field step into three single-field steps.
- LP64 macOS `long` and pointer size differ from PS1 ABI. Headers, pointer-bearing
  resource structures, pointer arithmetic, and casts need an explicit audit.
- Hardcoded addresses include DB base `0x800188b8`, scratchpad `0x1f800000`,
  shell/load/sound buffers, and dashboard/menu image buffers. Native structs
  cannot simply be overlaid onto disc bytes.
- For a rotation node, collision traversal in `HD.c` applies
  `local = R * (world - t)`. Rendering `HIER.c` transposes node rotations when
  composing matrices. Candidate visual local-to-parent transform is
  `t + transpose(R) * local`, then compose with parent. This is a source-based
  inference, not yet implemented or validated across full hierarchy/GTE rounding.
- Vehicle roots are kinds 10,20,30,40,50,60,70,80,90,100,110,120; see `UA.H`.
- Math function addresses: `0x800fe9c4`, `0x800feaf8`, `0x800fec2c`,
  `0x800fed60`, `0x800feedc`. SDK `SquareRoot0`: `0x8015bf74`, table `0x80184728`.
- SDK `rand` delegates to BIOS A0/2F. RNG compatibility is not verified.
- Sound requires VAB/SPU decoding/voice behavior plus CDDA. Only raw CDDA file
  access exists. Do not report audio complete based on reading track bytes.
- Existing source `build/SIPS_600.07.rom` matched its reference artifact, but
  a fresh matching build of the pinned baseline has **not** been performed.
  `/Applications/DuckStation.app` exists; no original game state-trace harness
  is implemented yet.

## Recommended continuation

1. Verify the native Bevy window with GPU access and inspect the captured image.
   Fix obvious projection/axis/asset issues and validate omitted transforms.
2. Establish original reference execution/state capture and a fresh source
   baseline. Use the arithmetic oracle as a narrow tool, not a substitute for
   gameplay traces.
3. Port the real car/update/loading dependency chain with controlled ABI,
   memory, RNG, time, and input. Preserve source function/module traceability.
4. Connect primitive rendering, original AI/weapons/collision, audio, controller,
   pause/restart/focus behavior; run the requirements' arena acceptance scenarios.
5. Validate one object's path to regular Bevy meshes/materials and record how
   much of the primitive renderer can be retired. Do not claim mixed depth/OT
   occlusion is solved without testing it.

The agreed goal is still incomplete. The current deliverable is a compiled,
unvalidated resource preview plus asset and arithmetic foundations.

(This text was generated via LLM.)
