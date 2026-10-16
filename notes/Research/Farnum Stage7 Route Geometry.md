# Farnum Stage7 Route Geometry

Read-only preparation for the native journey from Cleetus through the farm to the five reconstructed wolves. **The coordinates below are route candidates, not a completed native walk or collision acceptance.** No server source, content, saves, active runs or input changed. Shared progress files are left to the native operator.

## Evidence and limits

Unmodified ForgeLightToolkit readers at revision `9c06714c5d79d9152c322814bc904abad944e06e` read four preserved original FabledRealms terrain chunks, 41 original prop DME meshes and the original `FabledRealms.map` with 2,019 nodes. Unmodified fr-adr-toolkit at revision `5acfdf8d6006fa6f43d1d3605f076a6be7393e72` resolves the ADR mesh references. Every selected original mesh/ADR CRC matches the catalog; source/output hashes and transforms are recorded in the proof.

The community GCNK reader's `LoadCollision` is a `TODO Bullet Physics` placeholder returning true. **It does not decode Bullet collision.** This analysis uses render-terrain slopes and ground-band render-mesh footprints as conservative obstacle proxies. The original MAP adjacency is a road reference, not a proved player navigation mesh. Invisible collision, dynamic actors, bridges, player capsule dimensions and actual movement behavior remain unproved.

Main proof: `../lab/reports/stage7-route-geometry/stage7-route-guide.json`, SHA256 `404cd14a6c0a74c4d1b7ec2cd8a5a0be628663ccc2d4c9b7c4b3ec83954d6efd`. Focused departure, wolf-floor and road-return proof: `stage7-route-approaches.json`, SHA256 `478c9880f7a7819762cd0459f1e1ca0283fb63ec9a66e2119f0c58d790fb2104`. Overview: `stage7-route-map.png` in the same directory.

The analysis samples terrain along segments at intervals no greater than 0.5 world units, including offsets of 0.75 units in X/Z for slope checks. Prop footprints are convex hulls of original mesh vertices/edge intersections within terrainY−0.5 through terrainY+3. A 2-unit grid proposes candidate corridors, then the simplified route is independently sampled. All coordinates retain original game X/Z, without the community Unity preview's whole-world Z flip. Y in this guide is render-terrain floor, not an instruction to change character altitude.

## Farm to wolves

Use the current player's clear approach to Cleetus, then visually establish the staging point below. Cleetus's NPC registration at `(-2078.389,200.7649)` is close to a fallen log; the short registration-to-staging segment has only **0.314 units** of render-footprint clearance. It is not a proved safe player departure. Skirt the visible local log, forge and anvil from the already clear player position.

| Candidate X | Terrain Y | Candidate Z | Purpose |
|---:|---:|---:|---|
|-2082|-55.638|200|Clear staging near Cleetus; inspect departure first|
|-2084|-55.068|202|Turn clear of local props|
|-2080|-56.426|272|Pass between eastern farm fence and western corral/stump|
|-2084|-55.823|342|Approach campsite on its eastern side|
|-2088|-56.982|370|Clear campsite and resume north|
|-2108|-50.215|440|Approach wolves from their southwest|
|-2106|-48.125|448|Pause and inspect wolf ground contact|
|-2100.965|-46.034|456.7631|First wolf's original prepared X/Z|

The staging-to-approach route is approximately **252.4 units**. It has no missing sampled terrain floor, maximum sampled centerline grade **0.338** and minimum corridor normalY **0.914**. Minimum render-footprint clearance is **1.558 units**; the campsite segment narrows to **1.718**. These are measured geometry values, not a native capsule clearance guarantee. Walk in bounded segments, inspect fresh screenshots and actual position/altitude, and stop when a visible obstacle or unexpected movement contradicts the guide.

The final approach to the first wolf is 10.1 units, maximum grade 0.222, corridor normalY at least 0.962, with no selected obstacle within 4 units. The first-to-fifth wolf span is 20 units, maximum grade 0.125; its nearest selected tree footprint is 3.417 units away.

A direct Cleetus-to-wolves line intersects the fallen log near `(-2079.164,205.100)`, a stump near `(-2086.070,271.066)`, a tent near `(-2097.983,357.331)` and two medium trees near `(-2095.53,428.273)` and `(-2095.629,433.688)`. Crates/rocks sit around X−2089/Z338–344; tents occupy X−2094…−2100/Z352–360. Do not use a single straight walking target.

The original MAP has a connected northern road spine through nodes 1220→1218→728→727→726→725→724:

`(-2078.810,228.185) → (-2067.828,255.998) → (-2074.802,261.010) → (-2075.074,308.163) → (-2087.995,332.149) → (-2060.063,392.085) → (-2050.331,414.838)`.

Those adjacent road segments have sampled grade no greater than 0.314 and selected render-footprint clearance at least 3.375. They provide a longer eastern visual road reference if the shorter campsite passage is blocked. A complete road-to-wolves connector has not been accepted natively.

## Wolf altitude observation

Prepared `rust-opening-cooking-original-assets/encounters.json` places all five wolves at Y−45.98195 while terrain rises across their X/Z positions. This comes from `prepare-opening-encounters.py` copying the first actor position and offsetting only X/Z. No height change was made during this investigation.

| Wolf | X | Z | Terrain floor Y | Actor origin minus floor |
|---:|---:|---:|---:|---:|
|1|-2100.965|456.7631|-46.033704|+0.051754|
|2|-2096.965|459.7631|-45.652397|−0.329553|
|3|-2092.965|462.7631|-45.126369|−0.855581|
|4|-2088.965|465.7631|-44.715856|−1.266094|
|5|-2084.965|468.7631|-44.204653|−1.777297|

This proves a content-origin/render-floor discrepancy. It does not independently prove the mesh origin, native collision capsule, animation grounding or visible burial. Inspect all five in the actual client before accepting or proposing a Y-only repair.

Bounded follow-up [[Stage7 NPC Grounding Semantics]] proves that original physics movement targets can replace networkY with a downward collision-hitY when HasTilt is false, as in the current wolf template. Initial stationary-spawn grounding remains unproved. Do not infer visible burial from the table alone.

## Wolves to Pappy remains pending

Follow-up [[Stage7 Native Return Routes]] supersedes the old straight north connector after actual181715 stopped at the tree. It also recovers a successful151602 southern climb with normal walking, disclosed obstacle backouts and three recorded Space presses. The new wolf-to-farm/cross-farm connectors still need native acceptance; use the observed historical route rather than the unaccepted A* candidate below.

Pappy is near `(-2184.513,-31.537,328.111)`, on a plateau roughly 25 units above the farm valley. A direct southwest line cuts the cliff. The earlier native blocked-log area around `(-2140.949,306.417)` and nearby log `(-2130.866,311.398)` should be avoided; another large fallen log sits around `(-2141.301,264.095)`.

The proof contains a 285.6-unit A* return candidate via the campsite, `(-2136,298)`, `(-2166,278)` and a western climb toward `(-2192,300)`. Its centerline has floor and avoids selected footprints, but the 0.75-unit side corridor drops to normalY **0.770** near the cliff-edge zigzags. It is **not recommended as a blind return route**.

An alternate original road sequence 728→729→730→731→732→733 is also checked in the focused proof. Segment 732 `(-2168.178,245.223)` to 733 `(-2195.877,291.045)` intersects damaged corral piece 03, object `1360108822`; nearby fence clearances are only 1.03 and 1.49 units. MAP adjacency therefore does not establish an unobstructed walking path. A fresh screenshot/native inspection of a fence gap and western approach is required before choosing this return. The final `(-2192,300)` to Pappy section is gentle in the mesh check, but its approach remains unresolved.

## Reproduction

New private analysis files are `../lab/reports/prepare-stage7-route-geometry.py`, `../lab/reports/analyze-stage7-route.py`, `../lab/reports/check-stage7-route-approaches.py` and the wrapper `../lab/tools/stage7-map-reader/`. Original community parser sources and existing readers remain unchanged. Output/proof paths refuse overwrite; use a new versioned directory for a future rerun.

The raw original MAP is `../lab/extracted/client-assets/pack_14/006082.map`, SHA256 `edd298b4341a367a5d0e3e19d3e2de04f63911de4c12bc5dcacb1e75bae75565`, catalog CRC `325f9a98`. Four original terrain raw hashes are retained in the main proof's source inventory and `farm-floor-inputs.json`. The new map wrapper builds with the private .NET toolchain and no additional packages; the mesh analysis uses existing numpy/Matplotlib in `universal-modder-env`.
