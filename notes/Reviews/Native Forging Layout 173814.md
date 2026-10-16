# Native Forging Layout 173814

Read-only review of `client-baseline-20261008T173814Z/forging-progress4.png`, the immutable ended-run checkpoint, recorded inputs and original terrain prop exports. **The coordinate drift does not move the recorded mouse points into different tiles. The server's recorded save error and session exit explain the frozen checkpoint; this screenshot mismatch does not prove a gravity bug.** No source, content, saves, native input or active-run files changed.

## Actual board calibration

The supplied screenshot is1279×960 pixels. Visually bounded tile-frame edges are approximately X475–823/Y218–566. Independent per-axis RGB gradient profiles support those border locations. An8×8 grid of that size has pitch **43.5 pixels** and cell(3,3) center approximately **(496.75,239.75)**. Use practical anchor **(497,240)** and step **(43.5,43.5)** for this captured layout, subject to fresh screenshot calibration after restart. Icon artwork can sit slightly off its cell center; this analysis does not claim a recovered subpixel native mouse transform.

The static guide predicted anchor(492.243490,232.526042), step44.791667. Across all64 centers the observed-frame comparison gives maximum absolute drift **4.536 pixels horizontally and7.224 vertically**. Every old guide center remains in its same visible tile. This conclusion also holds when the actual left edge varies474–476, top edge216–220 and either pitch varies43.3–43.7. All five late recorded drag paths from18:06:59 onward likewise map to the intended same-index cells under those bounds.

For example, the repeated `(492,456) → (492,501) → (492,546)` drag remains cells **(3,8)→(3,9)→(3,10)**. Drift alone is therefore insufficient to explain a wrong-cell chain under the original floor-to-cell rule. Timing, server availability and actual visible tile identities remain separate conditions.

## Frozen checkpoint and later native prediction

The saved active attempt has six chains, two collected hammers and score6600; all playable cells are marked idle. `character-after.json` filesystem timestamp is18:07:40.923402UTC; `forging-progress4.png` is18:08:11.181682UTC, approximately30.26seconds later. Two recorded drags occur between them. The screenshot also displays score6600, but visibly contains hammer icons at cells(5,5),(7,5),(3,9), whereas the frozen saved board contains hammer606 at(5,4),(3,5). The last first-column drag crosses visible blue ore/hammer/orange ore; the saved cells are402/402/402. These are observed differences between two times, not a simultaneous active-server comparison.

Crucially, `server-stdout.log` line282 records `session_ended` with **`Access is denied. (os error 5)`**. The independently finalized application decode has its last server application at frame10693 and later client applications without further server applications. The separate wire reviewer identifies later native OnChain traffic beginning at10743. The native movie can continue local prediction after the server stops, while the persistent board remains frozen. **Do not diagnose gravity, refill order or chain validation from this later screen/checkpoint mismatch.** Atomic-save handling is owned by the separate persistence investigation and needs its own regression/native acceptance.

## Anvil placement limitation

`anvil-approach3.png` visibly places the player in front of a large boulder while the **Anvil / PressX** prompt remains available. The anvil itself is not visibly clear. This is a native presentation/approach limitation despite interaction being discoverable; it is not evidence that the anvil is wholly unreachable.

The original community scene export identifies nearby `sg_boulder_02.adr`, object **1254768169**, at `[-2083.2632,-57.039143,190.98073]`, scale0.77000004. Its transformed render footprint spans approximately X−2086.958…−2080.046/Z187.599…193.821. The reconstructed dynamic anvil registration `[-2084,-55.6,190]` lies **inside** that convex ground-band render footprint. The nearby original static anvil is a different placement at `[-2078.2856,-55.983376,198.6523]`; no relocation has been selected or implemented.

This corroborates the visible boulder overlap using original geometry, but the community reader does not decode Bullet collision. A clear standing point, visible model and normal interaction must be checked before any placement acceptance. No speculative content edit, migration or deletion of original scenery was prepared.

## Preserved proof

`../lab/reports/native-forging-layout-173814/proof.json`, SHA256 `186553f4a4754c5deb36f437a246d275e01bda705078765d10c9d8c97eaef6e6`, preserves ten input hashes, all64 coordinate comparisons, the five recorded drags, pixel edge profiles, frame/timing limits and original boulder footprint. Reproducer `../lab/reports/calibrate-forging-native-173814.py` refuses existing output. The original screenshots remain unchanged.
