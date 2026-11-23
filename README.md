# Twisted Metal on Bevy

tm1-bevy is a desktop port of the original PlayStation game *Twisted Metal*, currently in development. You can drive all twelve regular vehicles through the six-stage single-player contest or play a local split-screen battle with two players on one keyboard.

The port uses the original C driving and combat routines. Rust and Bevy handle input, graphics, sound and disc access. The game reads its levels, vehicles, music and movies directly from a player-supplied BIN/CUE image of the game. The Japanese release is the verified reference disc.

## Quick start

1. Download the archive for **macOS (Apple Silicon)** or **Windows (x64)** when a build is published on [GitHub Releases](https://github.com/ediskandarov/tm1-bevy/releases).
2. Extract the archive. Place `tm1-bevy` (macOS) or `tm1-bevy.exe` (Windows) in the folder containing your disc image's `.cue` file. Keep the BIN files at the paths listed in the CUE.
3. Double-click the executable. At the title screen, press **Enter** for single-player. Use **Left** and **Right** to browse cars, confirm **SELECT** with **Enter**, then choose **START**.

To build the game yourself, follow the [build guide](docs/building.md).

## Keyboard controls

| Action | Single-player / Player 1 | Player 2 in split-screen |
| --- | --- | --- |
| Accelerate / reverse | W / S | Up / Down |
| Steer left / right | A / D | Left / Right |
| Hard turn / turbo | Left Shift / T | Right Shift / Slash |
| Fire machine guns | Space | Comma |
| Fire selected weapon | F | Period |
| Next / previous weapon | E / Q | L / K |

Press **P** to pause. In single-player, **C** cycles camera views, **B** toggles the rear-view mirror and **R** restarts the encounter.

[Playing the game](docs/playing.md) · [Implementation status](docs/status/README.md) · [Documentation](docs/README.md)
