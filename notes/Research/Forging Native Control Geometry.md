# Forging Native Control Geometry

Original-source coordinate guide for the next native Forging139 acceptance. **These are static candidates, not a native play pass.** No input, packets, source, saves, release or active-run files changed. The original movie and UI assets remain intact.

The official JPEXS26.3.0 XML exporter reads supplied `AssetsW_005.pack` entry788, `game_forging.gfx`, SHA256 `78b53823ad24337bba1a7b2103c3820c1baf7e13f3f025709326bf9daa663d99`. Its new private XML is byte-identical to the prior proposal export, SHA256 `eeb83bf2987673a6c6f800bea67b137feee2b6409f9b48076032866fcc4caf6e`. The guide also reads original AS2, native Lua, original title-bar/tutorial movies and UI window XML. It uses no new binary-format parser. Tool provenance and all input hashes are preserved in `../lab/reports/forging-controls-geometry.json`, SHA256 `685b77f157b9e78a212c6729aae6a5ebb168c7210a4c43e11fe0ac3357f50bd4`.

## Board at a1280×960 game client canvas

The original Forging movie is **650×768**, unlike the768×768 Cooking/Smelting movies. Original `OnGetResizeInfo` returns those actual dimensions; Lua still computes scale `min(1280/1024,(960-100)/768) = 215/192`, then centers the actual movie. Its viewport is727.864583×860, origin **(276.067708,50)**. Do not use the outer desktop/window border dimensions.

The original root `mcGame` is at(53.05,23). Runtime `mcGrid` offset(+240,+240) and its `tiles` child offset(−240,−240) cancel. Each tile is40×40 movie units; a center is `(53.05+40*x+20,23+40*y+20)` before viewport scaling. The prepared8×8 reconstruction occupies original14×14 indices **x3…10, y3…10**. Planner coordinates are these original indices; screenshot row0/column0 means original cell(3,3).

Candidate playable board edges are **(469.848,210.130)–(828.181,568.464)**. Cell(3,3) center is **(492.243490,232.526042)**; horizontal and vertical center spacing is **44.791667** client pixels. All64 centers and inverse floor-to-cell checks pass in the report. Compare the fresh screenshot's first tile, adjacent center and board edges before input, replacing anchor/spacing if actual rendering differs.

Native173814 later renders a slightly smaller grid: practical anchor **(497,240)** and step **(43.5,43.5)** in its1279×960 screenshot. All64 old guide centers still fall in the intended tiles, so this drift does not explain its failed late drags. The server had exited with a save error; the later screen/checkpoint mismatch is not a proved gravity bug. See [[Reviews/Native Forging Layout 173814]] for calibrated pixel support, timing and anvil/boulder overlap evidence. Fresh runs still require calibration.

| Original index | Client X center | Client Y center |
|---|---:|---:|
|3|492.243|232.526|
|4|537.035|277.318|
|5|581.827|322.109|
|6|626.618|366.901|
|7|671.410|411.693|
|8|716.202|456.484|
|9|760.993|501.276|
|10|805.785|546.068|

For cell(x,y), select X from rowx and Y from rowy. The original hammer mold6 frame1 art lies approximately(479.832,716.221)–(814.390,857.231), below the playfield. It displays progress; it is not a drag destination.

## Normal gestures and planner

1. Dismiss the visible tutorial and wait until the board is rendered, unpaused and idle. Use an atomic saved snapshot from the current attempt; the planner never sends input or packets.
2. After screenshot calibration, run:

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' '..\lab\tools\plan-forging-chain.py' '<current atomic snapshot path>' --limit 1 --anchor 492.243490 232.526042 --step 44.791667 44.791667
```

3. Read its snapshot hash/instance, phase and `wait` notice. Use **one** suggested chain. Hold left mouse at its first `screen_centers` point, trace every listed adjacent same-color center in order, then release at the last center. At least3 distinct cells are required. Do not skip intermediate cells or cross a diagonal: original mouse position is sampled on ticks and converted using floor(position/40). Give each successive center enough dwell for a tick; the original tile update interval is33ms, but actual reliable input timing remains a native check.
4. Wait for disappearance, fall, appearance, queue refill and any original reshuffle to finish; then reread the latest snapshot and plan again. Several choices from one snapshot are alternatives, not a batch to execute. The helper refuses nonidle selectable/hammer boards and terminal phases; it does not prove that a visible pause/tutorial overlay is absent.
5. Hammer606 is not a selectable chain tile. Clear matching tiles beneath its column until gravity carries it into the original dropzone. A real collection updates the quota/mold. Quota5 is disclosed reconstructed tuning; the original mold advances20 frames per collected hammer at that quota.

Release calls original `checkMouse`→`stopChain`→`verifyChain` and sends `OnChainMsg`; there is no separate Submit button. A one-tile quick release within500ms enters original click mode, where clicking the selected last tile can confirm. Use a deliberate drag for the initial acceptance to avoid that ambiguity.

The tile/random queues are internal arrays updated by the original network callbacks. There is no user queue button among original movie children. Let refill/ack happen automatically. A no-move board uses original reshuffle/rebuild; do not fabricate a move or completion.

The existing helper was independently imported read-only against a tiny synthetic three-cell board. It returned only the legal chain; nonidle and terminal variants returned no chain. This verifies planner/geometry integration, not the current native board or server/client synchronization.

## Separate native UI candidates

These UI windows use authored scale1, independently of the650×768 game viewport. Original `MinigameTitleBar.gfx` SHA256 `f484803b5272c406101948a485ce077fca350f1a235eb16c4768d5a4b840c316`; original tutorial shell SHA256 `dd95acdf4c391cd39db0a565c6d48abfb8f5eddbc37553dff8a1c8b379d0d417`. The original forging tutorial asset SHA256 `ec0c8cf4481ea41236ad0be4c90a1e35fd324169446d2a00e38c093706bc2856` has3 pages. Screenshot calibration remains required.

| Control | Candidate center | Original behavior |
|---|---|---|
|Tutorial Close/Play|(817.5,719)|Authored icon registration. Native `Close`→`TutorialClosed` starts first game; finishing all tutorial pages is optional. Dynamic FRButton art replaces placeholders, so this is not a claimed hitbox.|
|Tutorial Previous|(605.8,704.2)|Previous page; disabled on first page.|
|Tutorial Next|(692.2,704.2)|Next page; disabled on last page.|
|Reopen tutorial|(1212,65)|Lua places50×50 window at(1187,40). Confirm icon visually.|
|Top-left exit X|(466.04,23.65)|First click expands confirmation.|
|Pause/resume|(814.94,23.5)|Shown when Lua identifies the active Flash as1. Confirm visibility.|
|Expanded exit confirmation|(639.99,22.65)|After approximately0.4s animation, click within its3s timeout to emit `ExitGame`. Candidate bar bounds(445.405,3)–(834.580,42.3).|

Ordinary in-game exit invokes native `Ui.QuitMiniGame`, a separate cancellation path whose response acceptance remains pending. Do not use it to close a won results page. Terminal Forging uses the separate native ScoreScreen/type8 and actual26/42 Close; identify its fresh visible control instead of reusing the top bar coordinates.

## Proof and next native gate

Reproducer: `../lab/reports/extract-forging-controls-geometry.py`, SHA256 `cc48780fbdc66254d98896755fd2082ed4a8a6cc24ae3e1579074d2b85cbf840`. It reads only official XML, original script exports, prepared configuration and existing planner, writes this one new private proof, and refuses to overwrite it. All requested vector AABBs resolve with zero unsupported definitions. Empty/static marker concerns are not runtime failures: original Cooking Measure and Stir both worked in actual play despite earlier static concerns.

Next native gate remains original139 entry/tutorial, screenshot-calibrated normal chains, queue/gravity without desynchronization, hammer collection/mold progress, genuine quota win/delayed callback/results, actual Close, fixed ingredient consumption, exactly one output/quest update and restart. Original MatchThree server balance tables are still missing; reconstructed board/rules and tint/recipe disclosures remain in [[Original Forging]].
