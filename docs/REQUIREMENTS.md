# Requirements

## Full-game objective

Port the gameplay and player-facing functionality of the supported JP release.
The playable arena below is the first milestone. Subsequent work covers the
remaining vehicle roster and special weapons, levels and their interactions,
opponents and progression, the original front end, multiplayer modes and
endings. Verify behavior against the supported release as each path becomes
playable. Track implemented paths and outstanding limitations in the [implementation status](status/README.md).

## Initial arena deliverable

### R1. Platform and scope

- A Bevy desktop application for macOS.
- First supported architecture: Apple Silicon. Windows x64 is also a release build target.
- One arena selected from the supported disc.
- A player-controlled vehicle, at least one opponent using the original AI,
  collisions, damage, weapons, effects, and the ability to restart the arena.
- Direct arena launch through a minimal port interface is acceptable.
- Full menus, campaign, all vehicles/arenas, split-screen, and cinematics are
  outside initial acceptance. Their platform dependencies must still be
  covered by the audit.

The selected scenario is XARENA1, the taxi (kind 20), the police car (kind 50),
the machine gun and the fire missile.

### R2. Player-supplied assets

- The source is local BIN/CUE for one specific JP revision.
- The player selects a CUE and grants access to the BIN files it references.
- Compare track sizes and hashes against the JP reference manifest.
- Warn on JP reference mismatches and use the audio tracks declared by the CUE. Report unreadable files, invalid sector bounds and malformed disc structures as errors.
- The loader obtains geometry, textures, game data, and audio from the image.
- Installing a separate set of previously extracted assets is not required.
- An automatically generated local cache is acceptable if tied to the image
  and converter version. Do not modify the source image.
- Read music tracks according to the CUE; filesystem access alone does not
  constitute sufficient disc support.

### R3. Gameplay behavior

- Preserve the original physics, collision, AI, and weapon algorithms.
- The core may remain in C. An off-the-shelf physics engine must not replace
  the original simulation.
- Preserve relevant integer arithmetic, rounding, overflow, RNG behavior,
  and object update order.
- Preserve the semantics of `gFieldsLastFrame` and related timers.
- Presentation frame rate must not independently change game state.
- Visual interpolation must not write values back into the simulation.

Criterion: with identical initial state, input, RNG, and sequence of simulation
time steps, the compared states of the port and the original match after each
corresponding step. Investigate differences instead of hiding them behind
arbitrary tolerances. Introducing tolerances requires a separate explicit decision.

This validates specific scenarios; it does not prove whole-game equivalence.
Real-time game pacing is validated separately.

### R4. Rendering

- The initial backend translates original primitives/commands into GPU rendering
  inside Bevy. A software framebuffer is not the target renderer.
- Support the geometry, textures, transparency, effects, and minimal HUD needed
  for the arena.
- Initially preserve original command order wherever required for correct
  display. Pixel-perfect PS1 fidelity is not required.
- Render at window resolution, scale the HUD correctly, and use an aspect ratio
  consistent with the scene.
- Modern visual improvements are acceptable if they do not change simulation state.
- Do not make the packet decoder the only available scene representation:
  retain access to source geometry, objects, camera, and materials.

Modern lighting, shadows, PBR, new geometry, and new textures are not required
for the first arena. Their feasibility is evaluated through the subsequent
migration to regular Bevy rendering.

### R5. Audio

- Play the music and sounds needed for the selected arena: engine, shots,
  impacts, and explosions.
- Preserve significant pitch, volume, panning, looping, and voice lifecycle
  behavior. Check whether game logic reads audio state.
- Bit-identical PCM output is not required.
- Document differences in reverb, noise, and envelopes after the experiment;
  do not automatically accept them if they change gameplay behavior.

### R6. Input and time control

- Support keyboard input for the first arena. Physical gamepad testing is deferred by player decision.
- Map input to original PS1 buttons and sample it at simulation steps.
- Focus loss and pausing must not cause uncontrolled step accumulation or
  stuck button states.
- Restarting resets arena state, input, and timers predictably.

## Acceptance checks

| Check | Expected evidence |
| --- | --- |
| Disc import | Supported image hashes and tracks; correct asset loading; clear rejection of an invalid image |
| Vertical slice | Arena launches, vehicle is controllable, AI acts, weapons cause damage, and effects and music are audible |
| Physics | Original and port traces for acceleration, braking, turning, jumping, wall impact, vehicle collision, and weapon impact |
| Timing | Comparison across several simulation step-duration sequences; simulation independence from presentation frequency |
| State | Positions, velocities, orientations, health, weapons, RNG, and relevant timers; references normalized to object IDs |
| Operation | Keyboard play through the victory screen, restart, pause and focus loss |
| Performance | Simulation time, CPU/GPU rendering time, and memory measurements on an identified Mac; no sustained simulation clock lag |

Record step counts, seeds, input recordings, original/emulator settings, and
selected scenes before running comparisons. Numerical FPS and memory targets
have not been set and must not be implied to have been met.

## Later goals

- Browser support with the same game core and player-supplied assets. The first
  macOS version must not depend on direct access to a physical CD drive.
- Gradually replace the primitive renderer with Bevy meshes/materials/entities.
- Expand to other arenas and the full game flow after evaluating the experiment.
