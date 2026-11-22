# Playable arena status

The macOS application launches XARENA1 from the verified JP BIN/CUE. The player selects one of twelve vehicles and fights a computer-controlled opponent on medium difficulty. The arena includes acceleration, reversing, steering, turbo, tire contact, bridges, collision damage, police pursuit and attacks, gun heat, missiles, pickups and regeneration. Health, selected ammunition and gun heat are visible in the chase view. Zero health ends the encounter and restart reconstructs the starting state. CDDA and decoded VAB voices play through Bevy.

The player completed a manual keyboard playthrough, heard game sounds and reached the **Arena cleared** victory screen. Keyboard play is the accepted input configuration for this milestone.

Sweet Tooth, Yellow Jacket, Darkside, Hammerhead, Outlaw, Crimson Fury, Warthog, Mr. Grimm, Pit Viper, Thumper, Spectre and Road Kill
load their own driving profile, collision geometry, three damage models,
special weapon and audio programs. Each selection was launched on the
Apple M3 Pro Metal backend and its chase-view frame inspected. Arena draws its opponent from the original card order with the BIOS random sequence starting at seed 1. Restart restores the same draw and simulation seed. The boss, remaining arenas,
campaign, front end and multiplayer are outstanding.

## Warehouse

Launch with `--level warehouse` to drive WH1 or WH2, selected by the original vehicle-dependent shell rules, against all three original opponents for the selected player. The game reads the level geometry, sky and textures from the disc, spawns the player at the original Warehouse start, and plays CDDA track 4. Warehouse uses its own trigger points, bridges, curbs, potholes and AI lane selection. The encounter can be restarted from the keyboard. The HUD shows the nearest surviving opponent and the number of rivals left. The round ends after all three rivals are defeated or the player is wrecked. Campaign progression is outstanding.

## Freeway

Launch with `--level freeway`. The selected vehicle determines the XFWY1 or XFWY2 database, textures and five opponents. The level uses 50 routing points, 18 bridge records, two potholes and CDDA track 2. Metal captures on the M3 Pro show Yellow Jacket and Outlaw on the highway. The keyboard restart fixture replays the same five-car lineup.

## City Park

Launch with `--level city-park` for the six-opponent encounter. Both park databases load with their corresponding textures, original player and opponent starts, 42 routing points, 54 bridge records, 16 curbs, the active pothole and the rectangular slick. Single-player City Park uses CDDA track 4. The M3 Pro Metal captures show Yellow Jacket on PARK2 and Hammerhead on PARK1.

## Reference execution

The test interpreter executes instructions loaded from the verified retail executable. The two vehicle records are copied to PS1-layout memory and compared byte by byte. The RNG starts at 1 for spawning and is then shared by both cars. Tests provide the BIOS random service, relevant GTE instructions, and explicit calls or stubs for presentation endpoints. These are controlled function-level scenarios. They do not execute an entire booted PlayStation or compare PCM and draw packets.

| Scenario | Observed result |
| --- | --- |
| Player dynamics | Three 360-step recordings, fixed three fields, repeating 3/4/5/3 fields, and a ramp. Full 0x4a4 bytes and RNG compared at each step. Ramp peak 164 with all four tires airborne. |
| Twelve player vehicles | For each vehicle the independently initialized 288-byte stat block and 360 mixed-timing movement steps match the disc executable. A separate firing scenario compares both car records, RNG and three steps of the special projectile trajectory for the six projectile-based specials. |
| Taxi and police together | 360 consecutive steps through both original update calls. Accelerate, hold the gun after step ten, steer left on steps 24–39. Cycle 4/5/3/3 fields starting at step one. Both complete car records and RNG agree after every step. |
| Opponent selection and initialization | Compared all twelve Warehouse lineups and 36 successive Arena draws with retail uaSelectOpponents, including deck resets and RNG. For each regular opponent, compared all 0x560 bytes after AICarInit and AICarSetDefaultWeapons, and the resulting RNG. For each of the twelve Warehouse player choices, initialized the full three-opponent group and compared each slot, reserved starting point, complete car record and final RNG with retail. |
| Group collisions and restart | Compared 250 Warehouse point hits with retail, covering separated and overlapping cars and pairs of excluded IDs. Replayed 180 steps through the keyboard frontend after a round restart and compared all three opponents, player health and RNG. |
| Battle selection | Checked the PlayGame scheduler against the JP instructions in 128 combinations of opponent count, occupied attack slots, surviving vehicles and elapsed fields. Compared the wait counter and every opponent record after selection. |
| Opponent radar | For every regular vehicle, checked contact color, screen position and all four edge directions against retail hudAddRadarSig at seven headings and fourteen offsets (1,176 cases). Warehouse Metal capture shows the three opponents in their original colors. V toggles the overlay. Active Warehouse recharge stations flash on the radar. |
| City Park pursuit and traction | All twelve player choices ran for 360 mixed-timing steps against the original six opponents. The replay checks all seven car records, active fragment positions and RNG every step. A separate 150-step steering and acceleration trace crossed the slick and regained traction, with the entire player record checked on every step. |
| Freeway pursuit | Twelve 360-step mixed-timing encounters compared every byte of the six vehicle records, active fragment positions and RNG after each step. The Road Kill encounter exercises a computer-controlled Hammerhead driving over another vehicle. |
| Warehouse pursuit | Twelve 360-step runs, one per player vehicle with its full three-opponent lineup. Compared starting position and heading, then all four complete vehicle records and RNG on every step. The reference executes the PlayGame battle scheduler and updates active flamethrowers and explosion particles. |
| Contacts | Original car-to-car and wall fixtures contact at steps 13 and 73. Both car records, RNG and normalized contact identities compared. 9,381 original world point queries included 2,742 solid contacts. |
| Warehouse recharge stations | Both DMD switches follow the gameplay state. A tire-contact fixture compared the entire player record with retail through both recharge cycles and compared both activation flags for 4,002 regeneration updates. The port plays VAB program 52 while charging. |
| Body damage | The renderer selects one of three retail DMD branches for each car. The stage matches the retail damage-update routines for every positive HP value, descending and ascending, for both cars (420 comparisons). At 75% health the body changes to damaged and below 11% it changes to critical. |
| Hammerhead crush | A moving Hammerhead reaches a stunned opponent at step 15, spends six charges and climbs over it. Both vehicle records and RNG match the disc executable through separation at step 19. With five charges the same approach leaves the crush inactive. |
| Flamethrowers | Compared Thumper firing forward at an opponent and into empty space, and the ordinary rear weapon firing backward. The 25-step forward and 12-step rear emitters produced 290 and 66 visible particles per activation. Both car records, RNG, particle coordinates and animation frame indices matched retail after every step. |
| Outlaw taser | Checked player firing at distances 647 and 648 and against a defeated opponent. Both car records, RNG and all sixteen arc points match the retail code on each of ten frames. A successful hit costs six charges and deals eight damage on the tenth frame. |
| Missile specials | Warthog launches three swarm missiles, Spectre a ghost missile and Road Kill a Deathspear. For each vehicle, tests compared locked and unlocked launches for 93 steps, including the random draws after impact. Compared both car records, missile poses and visible trail vertices, frame indices and colors against retail. |
| Weapons | A machine-gun round and all six regular missile types were fired at the police car. Compared missile movement through impact, car state, ammo and RNG. Fire, homing, power, singing and rear missiles dealt 7, 15, 25, 3 and 7 damage respectively. Freeze applied its status without HP loss. |
| Input and round | Keyboard and virtual gamepad reach identical sampled state at different presentation rates. Focus loss, twenty seconds paused, gamepad removal/reconnection and restart checked. In a simulation run the police destroyed the unattended taxi after 741 idle steps; the round froze and restarted cleanly. |
| Arithmetic | 8,192 matrix, 4,096 normalization and 4,444 square-root cases compared with the original executable. |

Reproduce from the port root with the supported absolute CUE path exported as `TM1_CUE`.

```sh
cargo test --release --offline --locked --manifest-path crates/tm1-core/Cargo.toml -- --include-ignored --nocapture
cargo test --offline --locked --manifest-path crates/tm1-assets/Cargo.toml -- --include-ignored --nocapture
cargo test --offline --locked -- --include-ignored --nocapture
cargo run --offline --locked -- "$TM1_CUE" --demo-screenshot local/arena-combat.png
cargo run --offline --locked -- "$TM1_CUE" --realtime-screenshot local/realtime.png
cargo run --offline --locked -- "$TM1_CUE" --object-mesh --demo-screenshot local/object-mesh.png
```

Inputs, initial conditions, original entry addresses and comparisons are in the tests. [Core output](evidence/drive-original.txt), [asset output](evidence/asset-tests.txt) and [frontend output](evidence/frontend-tests.txt) record the runs. Images under `local/` contain disc-derived material and stay out of version control.

## Native presentation

On the Apple M3 Pro (18 GPU cores, macOS 15.8.3) the native Metal capture showed the taxi, arena textures, projectiles and HUD in a 1280 × 800 logical window. With the normal 50 ms simulation clock and scripted input, the app presented 300 steps over 15.0046 seconds with 4.5956 ms remaining. The final frame showed the police colliding with the taxi and the taxi down to 66 health. Median / p95 timings were 0.0269 / 0.0447 ms per simulation step, 0.7122 / 0.7644 ms for CPU geometry preparation, and 0.8596 / 1.1361 ms for the GPU graph interval. There were 80 CPU samples after warmup, 79 valid GPU timestamp pairs and 310,704 KiB RSS at capture. Engine and music playback cursors advanced. [Native run output](evidence/metal-realtime.txt). The separate demo capture advances one simulation step per presentation frame to compare two render paths at the same point.

The arena, both cars and effects share a perspective camera and depth buffer during normal play. The HUD is an overlay. Each opaque vehicle body is submitted once as a retained mesh. The original arena enables headlight groups 24 and 54 on the taxi and police car. Their beam texture pages request additive blending. The renderer modulates light in the texture's display color space and adds it to the scene for STP texels. A black texel in the beam adds zero light. Non-STP texels replace the destination color. To reproduce the collision capture, run with `--demo-screenshot local/collision.png 300`.

Texture zero is transparent. Semi-transparent primitives honor the texture word's STP bit with an alpha approximation. The arena imports eight billboard instances with original center, extents and UVs; the primitive camera expands them in view space. The crowd display loads both textured DMD branches and changes pose every 16 simulation steps. Its selected child matched 512 successive calls to the retail HIER routine. The port clocks this cosmetic animation once per simulation step, including while off camera. The original clocks it when hierarchy traversal visits the node. Other PS1 blend modes, including subtraction, and exact ordering-table placement are incomplete. Flamethrower particles use the 16-frame Aburst animation read from XARENA1 and its additive texture pages. The port tracks flame bursts in a dedicated pool. The original shares that pool with other explosion types. Missiles use the arena DMD models (201 and 312) and the eight-frame contrail sprite set (652). Other combat effects still use replacement geometry. Camera and HUD are port-specific.

The arena plays CDDA track 3 and VAB programs for the taxi engine, gun, impact, explosion and missile launch. Pitched engine looping uses the ADPCM loop marker; shots retain bank stereo with five listener-relative pan positions. The port caps effects at 22 concurrent voices plus music and engine. SPU reverb, ADSR, noise and exact voice priorities differ. Original SOUND.c uses key status for voice management. The tested car and weapon updates agree while audio output endpoints are stubbed. Metal smoke reported live audio sinks. The player confirmed audible game sounds during play. Relative music and effect levels were not individually evaluated.

## Source baseline and scope

A fresh offline linux/amd64 build of pinned source `fb8f9dd4c3aff275893c2573c432a94f644e9970` with pinned maspsx produced ROM SHA-256 `32f36dbb4e9451d767869cd7c6f226d35fa97884fd39c0409ecdf4469f974b43`. The retail reference ROM hash is `95b57efc67c0625f98c76442d6c939b509e5e8b375aed9c344c858aaeccb5bc9`. Both ROMs are 610,304 bytes long, with 325,480 differing byte positions. The first difference is at file offset 12616 (zero based). Differential tests use the verified retail executable directly. The project's fixed source snapshot does not establish a byte-matching rebuild.

The native run reported zero gamepads. Physical controller testing is deferred. Full-game parity, other arenas, exact mixed rendering occlusion and exact SPU behavior are outside the evidence above.
