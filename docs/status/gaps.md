# Verification limits and full-game audit

## Source baseline and scope

A fresh offline linux/amd64 build of pinned source `fb8f9dd4c3aff275893c2573c432a94f644e9970` with pinned maspsx produced ROM SHA-256 `32f36dbb4e9451d767869cd7c6f226d35fa97884fd39c0409ecdf4469f974b43`. The retail reference ROM hash is `95b57efc67c0625f98c76442d6c939b509e5e8b375aed9c344c858aaeccb5bc9`. Both ROMs are 610,304 bytes long, with 325,480 differing byte positions. The first difference is at file offset 12616 (zero based). Differential tests use the verified retail executable directly. The project's fixed source snapshot does not establish a byte-matching rebuild.

The native run reported zero gamepads. Physical controller testing is deferred. The campaign and duel evidence covers individual stages and selected encounters. Whole-game equivalence remains open.

## Known differences

| Confirmed gap | Evidence and player impact |
| --- | --- |
| Arena video monitors | Each UADMD/XARENA1, UADMD/XARENA2 and UA2PLAY/XARENA1 database contains four kind-810 objects. The original initializes their UVs and selects the current display buffer’s texture page in `UAeffectUpdateTVs`. The port imports object faces through `tm1-assets/src/scene.rs` and draws the battle texture atlas without routing the display buffer to those faces. |
| Title input and idle demo | `screenDisplayTitle` polls the controller and waits for release after navigating. In `src/contest.rs`, a held unmapped host key resets the title idle timer, and a newly pressed Enter can confirm while the previous navigation key is still held. The title demo and the showroom release handling themselves are implemented. |
| Remaining audio voice behavior | Explicitly stopped final looping effect voices can release in campaign and duel. Pre-rendered one-shots and engine shutdown still bypass that envelope. The hardware reverb and noise paths are absent. The audit did not establish an audible use for either in normal JP play. |
| Presentation differences | Battle blending uses the host destination color and rasterizer. Transparency over a nonblack background, clipping and pixel coverage can differ from the console. The chase framing and HUD layout have port-specific elements. Stage loading does not reproduce physical CD seeks. |

Verification still needed includes a continuous keyboard contest through Minion and the ending, the Hard continuation through Arena 5, and comparison of a complete retail battle frame’s state, draw packets and sound requests. Existing targeted replays and forced-transition captures remain useful evidence for their covered scenarios. The renderer does not need pixel-perfect output for initial arena acceptance under R4. Physical gamepad testing is deferred by player decision. The Windows x64 release has since been cross-compiled on macOS. A Windows playthrough remains open. Browser support and migration to Bevy meshes/materials are later project goals.
