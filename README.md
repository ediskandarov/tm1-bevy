# Twisted Metal on Bevy

The macOS build runs the arena driving prototype on Metal. It reads the JP
disc, simulates the selected player vehicle and its computer-controlled opponents, and renders a chase view.
The arena includes machine guns, missiles, pickups, contact damage, sound, music and round restart.
Initial decision date: 2026-11-22.

The goal is to port the full original game. The first playable milestone is
the arena encounter. Warehouse is playable against all three original opponents simultaneously. Drive onto either Warehouse recharge station to repair your vehicle. Both stations return after both have been used. The radar shows active stations as flashing crosses. Arena draws one opponent using the retail selection rules and a reproducible starting seed. Freeway features five simultaneous opponents, City Park features six and Cyburbia features eight. On Rooftops, the camera stays at the edge and tracks the car when it falls below the buildings. Defeating the first three rivals brings Minion into the same battle. Press R during the boss fight to replay the whole stage. The default run follows all six stages in order. Stage wins carry regular ammunition and the selected weapon forward, refill health and recharge the special. You start with three lives. Losing a life gives the player a fresh weapon loadout and leaves the rivals in the arena. Enter skips the five-second Minion introduction. Defeating Minion displays a password reward and the credits. Use Up or Down to change the scroll speed and Enter to continue. All twelve regular vehicles are selectable on each stage. With
`TM1_CUE` set to the path of the JP disc cue, run:

```sh
cargo run --locked --offline -- "$TM1_CUE"
cargo run --locked --offline -- "$TM1_CUE" --level arena-5 --difficulty hard
cargo run --locked --offline -- "$TM1_CUE" --car sweet-tooth
cargo run --locked --offline -- "$TM1_CUE" --level warehouse --car road-kill
cargo run --locked --offline -- "$TM1_CUE" --level freeway --car outlaw
cargo run --locked --offline -- "$TM1_CUE" --level city-park --car yellow-jacket
cargo run --locked --offline -- "$TM1_CUE" --level cyburbia --car outlaw
cargo run --locked --offline -- "$TM1_CUE" --level rooftops --car road-kill
```

## Local two-player battle

Choose **2P BATTLE** at the title. At **PRESS ANY BUTTON**, either player can claim the first pick. Player one uses WASD and Space in the garage. Player two uses the arrow keys and Comma. Move left or right to browse cars, choose **CAR INFO** to read the driver's biography, or confirm the car to pass the turn. The second player chooses from the remaining cars.

After both picks, choose **START** to play or **OPTIONS** to change the battleground or controls. Under **STAGE**, browse the six arenas with Left and Right and press Enter to confirm. Under **SELECT CONTROLS**, open the layout with Space for player one or Comma for player two. The upper viewport follows player one and the lower viewport follows player two.

| Action | Player one | Player two |
| --- | --- | --- |
| Drive / reverse | W / S | Up / Down |
| Steer | A / D | Left / Right |
| Hard turn / turbo | Left Shift / T | Right Shift / Slash |
| Machine guns / selected weapon | Space / F | Comma / Period |
| Next / previous weapon | E / Q | L / K |

Each split-screen view follows its car using that vehicle’s two-player chase camera.

Press P or the numeric keypad Enter to pause both views. Either player can navigate the pause menu. Both players must release their keys between choices and before driving resumes. Choose CONTROLLER OPTIONS with Space for player one or Comma for player two. AUDIO adjusts the shared music and effects mix. START resumes the round and QUIT returns to the title movie.

After the round, the score stays on screen while the same arena reloads. The next round starts automatically with both cars repaired and the score preserved. Enter skips the score display. Esc returns to the title. To launch a specific match directly, use the second car’s slug after `--duel`.

```sh
cargo run --locked --offline -- "$TM1_CUE" --level warehouse --car yellow-jacket --duel outlaw
```

## Single-player contest

The opening sequence plays the three JP presentation cards, Sony logo and title movie from the disc. Built-in stable Rust decoders stream the STR video frames and play the interleaved XA soundtrack at its native 37,800 Hz. The video decoder includes [host-adapted PSoXide components](src/cinema/video/NOTICE.md). Enter skips the title movie or the Sony logo after its first two seconds.

Leaving the title idle cycles through the battle movie, contest history and vehicle showcase. During the history, Up and Down adjust the scrolling speed. Enter skips history or a showcase and returns through the title movie.

With only the CUE argument, press Enter at the title to open the garage at Sweet Tooth. Left and Right browse the twelve rotating vehicles in retail card order. Up and Down choose SELECT, CAR INFO or EXIT. SELECT opens the battle card for your vehicle. Choose START there to enter the arena, or OPTIONS to change difficulty, choose a control layout or enter a password. Press Enter again to close a driver biography. To start directly in the garage at a specific vehicle, pass `--garage --car outlaw`. D cycles through Medium, Hard and Easy before starting. Finishing Rooftops on Hard continues into Arena 5 after the credits with the remaining lives and inventory. On the selection screen, P opens the stage password editor. Use 1 for triangle, 2 for square, 3 for cross and 4 for circle. Left and Right move between slots. Enter accepts the code and Esc goes back. Space inserts a blank slot. Use 21422 to play Arena 5. The special passwords below can be entered before starting the contest.

| Password keys | Result |
| --- | --- |
| 1, Space, 2, 4, 4 | Infinite Weapons refreshes weapon slots before firing. |
| 2, 1, 3, Space, 4 | God Mode protects player health from incoming damage. |
| 1, 3, 4, 2, 1 | Minion and Friends starts Rooftops with Minion joining three live rivals. |
| 4, 4, 1, 3, Space | Helicopter View unlocks H to switch the flying camera on Arena, Rooftops and Arena 5. |
| 1, 3, 1, 1, 4 | Preview Minion's introduction, then return to selection. |

With God Mode enabled, number keys during combat stand in for the original simultaneous controller presses. Tap the first digit, release it and tap the second within 30 simulation updates (about 1.5 seconds at the usual rate).

| Keys in order | Effect |
| --- | --- |
| 1 then 2 | Toggle protection from damage. |
| 1 then 3 | Toggle half damage when vulnerable. |
| 1 then 5 | Toggle infinite weapons. |
| 3 then 4 | Switch the collision strength and weight settings between the two cheat presets. |
| 6 then 7 | Stop or resume AI driving using the second controller sequence. |

The pause HUD shows the keyboard digits for the current stage. Explicit car and level arguments start the fight directly.

Available choices are `sweet-tooth`, `yellow-jacket`, `darkside`, `hammerhead`,
`outlaw`, `crimson-fury`, `warthog`, `mr-grimm`, `pit-viper`, `thumper`,
`spectre` and `road-kill`. Yellow Jacket is the default.
Press F to fire the selected weapon and E or Q to cycle forward or backward through the inventory. Hammerhead uses its special automatically when driving into an opponent with at least six special charges.

To capture the selected vehicle firing its initial special, append
`--demo-special-screenshot local/special.png 19` after the car choice.

Project documentation is maintained in English.

- [Current implementation status and evidence](STATUS.md)
- [Console agent handoff](HANDOFF.md)
- [Requirements and acceptance criteria](REQUIREMENTS.md)
- [Assumptions, risks, and architecture validation](ASSUMPTIONS.md)

## Agreed decisions

| Area | Decision |
| --- | --- |
| First platform | macOS |
| Later direction | Browser; outside initial acceptance |
| Game core | Retaining C is acceptable; Rust/Bevy provides the new platform layer |
| Assets | Local BIN/CUE for one specific JP revision, supplied by the player |
| First deliverable | One playable arena |
| Initial renderer | Custom GPU renderer for original primitives inside Bevy |
| Visuals | Pixel-perfect fidelity is not required; a modern presentation is desirable |
| Physics | Compare state against the original with identical inputs and timing |

## Source baseline

- Repository: https://github.com/ediskandarov/tm1_decomp
- Branch: `seems-ready`.
- Recorded baseline commit: `fb8f9dd4c3aff275893c2573c432a94f644e9970`.
- Local main repository: `../tm1_decomp`.
- Local branch worktree: `../worktrees/match-weapon`.
- Decompilation configuration: JP, executable `SIPS_600.07`.

The inspected worktree contained uncommitted changes in
`config/JP/SIPS_600.07.yaml`, `config/JP/report.json`, and `src/tm1/SOUND.c`,
and a modified `tools/maspsx` submodule. These are not part of the recorded
commit. The unchanged `TIMER.c` has been imported from the recorded commit for a host
integration probe. Further imports must explicitly select their snapshot.

`SIPS_600.07` alone does not identify the full disc. The supported nine-track
BIN/CUE layout is now validated against [jp-disc.tsv](data/jp-disc.tsv).

## Build and inspect

Bevy is pinned to **0.20.1**. The development toolchain is Rust **1.97.1**,
the minimum required by this Bevy release and its WESL dependencies.
The Bevy dependency graph has been fetched and built; see STATUS for runtime limits.
The two platform-independent crates have separate lockfiles so their tests can
run without fetching the renderer dependencies.

```sh
cargo test --locked --manifest-path crates/tm1-assets/Cargo.toml
cargo test --locked --manifest-path crates/tm1-core/Cargo.toml
cargo run --locked --manifest-path crates/tm1-assets/Cargo.toml --bin inspect -- /absolute/path/to/disc.cue
TM1_CUE=/absolute/path/to/disc.cue cargo test --locked --manifest-path crates/tm1-assets/Cargo.toml --test disc_revision -- --ignored
```

The real-disc test is opt-in and requires an absolute CUE path. No game assets
are included in the repository. The source headers and parser layouts are based
on the recorded decompilation revision.

Install the pinned compiler before building:

```sh
rustup toolchain install 1.97.1 --profile minimal --component rustfmt
cargo fetch
cargo run -- /absolute/path/to/disc.cue
```

Drive with W/S, steer with A/D, hold Shift for a hard turn and press T for turbo.
C cycles forward through chase, far chase and cockpit views. Z cycles backward. Press 8 during a single-player fight to enter the original helicopter unlock chord. On Arena, Rooftops and Arena 5 the helicopter view then appears between far chase and cockpit. Once the cockpit transition settles, the view shows the selected vehicle’s disc artwork, steering wheel or handlebars, and digital speedometer. B toggles the live 101 by 30 pixel rear-view mirror. Entering helicopter view switches the mirror off. The combat HUD reads its weapon icons, digits and life markers from the level database. Windshield damage appears below 50%, 30% and 10% health. P opens the pause menu over the frozen battle. Use the arrow keys and Enter to resume, quit to the title, balance music and effects, or select one of the four original controller layouts. Keyboard driving keeps the same bindings in every layout. P also resumes directly. R restarts the encounter, V toggles the opponent radar, L lists surviving opponents, Tab toggles the overview, and Escape exits.
Space fires the machine gun, E and Q cycle inventory in either direction and F fires the selected weapon.
Each camera draws arena geometry, cars and effects in a shared PlayStation ordering-table sequence.
On a gamepad the face buttons west/east control acceleration/reverse, north
activates turbo and south modifies the D-pad turns. During battle, Start pauses and Select restarts. In the single-player pause menu, Start, Select and the south button confirm the highlighted choice. On the keyboard, Enter or Space confirms. Release the controls between choices and before returning to battle.
Right shoulder fires the gun, left shoulder fires the selected weapon and the left analog trigger cycles inventory. The first arena was playtested on keyboard. Physical controller testing is deferred.

The driving comparison runs in release mode against verified player-owned bytes.

```sh
TM1_CUE=/absolute/path/to/disc.cue cargo test --release --locked --manifest-path crates/tm1-core/Cargo.toml -- --include-ignored --nocapture
```

## CI and GitHub Releases

Each push to `main` runs the application, asset reader and game core tests in
release mode. Tests marked as requiring the original disc are skipped on CI.

Pushing any Git tag runs the same tests and, on success, publishes a GitHub
Release for that tag. Download `tm1-bevy-macos-arm64.tar.gz` for Apple Silicon
or `tm1-bevy-windows-x64.zip` for 64-bit Intel or AMD Windows PCs. Each archive
contains the executable. Supply your own supported BIN/CUE files when playing.

## Experiment sequence

1. Verify the original matching build; identify the disc and reference scenarios.
2. Verify that the C core builds with Rust on macOS and assess structure and
   arithmetic portability.
3. Read actual arena assets from BIN/CUE and display geometry through Bevy.
4. Run the simulation, input, opponent, and weapons; compare state to the original.
5. Integrate effects and music; validate a complete gameplay session.
6. Move one visible object to a regular Bevy mesh/material and assess the cost
   of further migration. This is a separate architecture check after the arena.

This sequence is provisional. If dependencies require a change, record the
reason and the outcome of the relevant assumption check.

Do not distribute disc images, extracted game assets, BIOS, or the original
executable with the port. Before importing source code, review its license
terms and retain required notices.

(This text was generated via LLM.)
