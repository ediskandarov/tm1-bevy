# Playing Twisted Metal

Extract the release archive and place `tm1-bevy` (macOS) or `tm1-bevy.exe` (Windows) beside the disc CUE. Double-click the executable in Finder or File Explorer, or run it from a terminal with no arguments. The game opens the title sequence automatically. Keep the BIN files at the paths listed in the CUE.

The game looks for a single `.cue` file beside the executable first, then in the current working directory. Uppercase `.CUE` also works. If a directory contains several CUE files, pass the one you want explicitly. An explicit path takes priority over discovery.

```sh
./tm1-bevy "/path/to/Twisted Metal (Japan).cue"
```

```powershell
.\tm1-bevy.exe "C:\Games\Twisted Metal (Japan).cue"
```

The supported layout uses one raw BIN per track. The port reads each track from its declared INDEX 01. Differences from the [JP reference manifest](../data/jp-disc.tsv) and executable hash produce warnings. Replacement audio must use whole 2352-byte CD audio sectors.

## Single-player contest

The opening sequence plays the three JP presentation cards, Sony logo and title movie from the disc. Built-in stable Rust decoders stream the STR video frames and play the interleaved XA soundtrack at its native 37,800 Hz. The video decoder includes [host-adapted PSoXide components](../src/cinema/video/NOTICE.md). Enter skips the title movie or the Sony logo after its first two seconds.

Leaving the title idle cycles through the battle movie, contest history and vehicle showcase. During the history, Up and Down adjust the scrolling speed. Enter skips history or a showcase and returns through the title movie.

At default startup, press Enter at the title to open the garage at Sweet Tooth. Left and Right browse the twelve rotating vehicles in retail card order. Up and Down choose SELECT, CAR INFO or EXIT. SELECT opens the battle card for your vehicle. Choose START there to enter the arena, or OPTIONS to change difficulty, choose a control layout or enter a password. Press Enter again to close a driver biography. To start directly in the garage at a specific vehicle, pass `--garage --car outlaw`. D cycles through Medium, Hard and Easy before starting. Finishing Rooftops on Hard continues into Arena 5 after the credits with the remaining lives and inventory. On the selection screen, P opens the stage password editor. Use 1 for triangle, 2 for square, 3 for cross and 4 for circle. Left and Right move between slots. Enter accepts the code and Esc goes back. Space inserts a blank slot. Use 21422 to play Arena 5. The special passwords below can be entered before starting the contest.

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

## Campaign progression

Warehouse is playable against all three original opponents simultaneously. Drive onto either Warehouse recharge station to repair your vehicle. Both stations return after both have been used. The radar shows active stations as flashing crosses. Arena draws one opponent using the retail selection rules and a reproducible starting seed. Freeway features five simultaneous opponents, City Park features six and Cyburbia features eight. On Rooftops, the camera stays at the edge and tracks the car when it falls below the buildings. Defeating the first three rivals brings Minion into the same battle. Press R during the boss fight to replay the whole stage. The default run follows all six stages in order. Stage wins carry regular ammunition and the selected weapon forward, refill health and recharge the special. You start with three lives. Losing a life gives the player a fresh weapon loadout and leaves the rivals in the arena. Enter skips the five-second Minion introduction. Defeating Minion displays a password reward and the credits. Use Up or Down to change the scroll speed and Enter to continue. All twelve regular vehicles are selectable on each stage.

## Driving and camera controls

| Keyboard | Action |
| --- | --- |
| W / S, A / D | Accelerate / reverse and steer left / right |
| Shift, T | Hard turn and turbo |
| Space, F | Fire machine guns and fire the selected weapon |
| E / Q | Select the next / previous weapon |
| C / Z | Cycle forward / backward through camera views |
| B, Tab | Toggle the rear-view mirror and the overview, respectively |
| V, L | Toggle the opponent radar and list surviving opponents |
| P, R, Esc | Pause or resume, restart the encounter, and exit, respectively |

The camera cycles through chase, far chase and cockpit views. Press 8 during a fight to enter the helicopter unlock chord. On Arena, Rooftops and Arena 5, the helicopter view then appears between far chase and cockpit. Entering it switches off the 101 × 30 pixel mirror.

The cockpit uses the vehicle's disc artwork, steering wheel or handlebars and digital speedometer. The HUD reads weapon icons, digits and life markers from the level database. Windshield damage appears below 50%, 30% and 10% health.

The pause menu freezes the battle. Use the arrow keys and Enter to resume, quit to the title, adjust music and effects or select one of the four original controller layouts. Keyboard driving uses the same bindings in every layout.

On a gamepad the face buttons west/east control acceleration/reverse, north
activates turbo and south modifies the D-pad turns. During battle, Start pauses and Select restarts. In the single-player pause menu, Start, Select and the south button confirm the highlighted choice. On the keyboard, Enter or Space confirms. Release the controls between choices and before returning to battle.
Right shoulder fires the gun, left shoulder fires the selected weapon and the left analog trigger cycles inventory. The first arena was playtested on keyboard. Physical controller testing is deferred.

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
./tm1-bevy --level warehouse --car yellow-jacket --duel outlaw
```

## Direct launch examples

With the executable beside the CUE, you can also start a specific encounter. On Windows, use `.\tm1-bevy.exe` in PowerShell.

```sh
./tm1-bevy --level warehouse --car road-kill
./tm1-bevy --level arena-5 --difficulty hard
```
