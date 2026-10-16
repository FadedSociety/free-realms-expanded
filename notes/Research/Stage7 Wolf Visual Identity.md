# Stage7 Wolf Visual Identity

Read-only investigation of native181715's cyan/orange shapes beside the tree. **Those photographed shapes match original static fairy flora, not a proven wrong wolf model. Later native183551 video now retains recognizable wolves and varying approach/lunge poses; the earlier image gap is resolved for this encounter.** No content, save, Rust source or native input changed by this review.

## Later native wolf rendering accepted in retained frames

Independent manual review of native183551 contact sheet and full frames at nominal video956,961 and964seconds confirms gray/brown four-legged Hooligan Wolves with canine silhouettes. At956, several labeled wolves approach the player with feet appearing at grass level. At961, a wolf visibly lifts its front in a distinct lunge pose;100damage text is visible. At964, nearby wolves are absent, chat lists defeat counts through5/5 and completion, and playerHP is525/1000. Cyan/orange flowers are separate static plants.

This supports actual visible rendering and pose changes for the observed encounter without replacing the original wolf model, skeleton or material. It does not assign each visible wolf a unique network GUID, prove every stationary-spawn height or accept all future terrain grounding. The combat wire/economy/quest audit remains separately owned; nominal video times are not embedded wall-clock timestamps.

Proof: [independent native visual review](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-native-wolf-visual-183551.json), SHA256 `492ef43f0decd13f3b909aeb028518d4cc8624bb8c731e500f46800d56fa073d`. Original308251391-byte recording SHA256 `6561d1cd149dbaea53d684ae423deb89891d571d00a1a20260261f3a57d10ca3` was independently rechecked. Seven source hashes bind the video, extraction manifest, contact sheet, three inspected full frames and review reproducer. [Visible lunge frame](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/runs/client-baseline-20261008T183551Z/analysis/wolf-video-945-35-1fps/frame-0017.png).

## Original identity and dependencies

Preserved Models.txt row3284 is `wolf_generic.adr`. Pinned Sanctuary NPC3156 independently identifies name5100200, Hooligan Wolf, model3284 and `wolf_generic.adr`. All five configured stage7 actors use this model with an empty/default texture alias. Combat reviewer independently reports the actual five native spawn packets retain the configured model/name identities; packet existence does not alone prove visible rendering.

Unmodified fr-adr-toolkit readers accept:

- ADR `wolf_generic.adr` → `wolf_generic.dme`, `wolf_generic.dma`, `wolf.dsk`.
- Material textures `wolf_tint.dds` and `WOLF.DDS`; the original diffuse texture depicts gray fur and yellow wolf eyes.
- DSK has48 named bones and48 bind poses. Original mesh has833 vertices; bounds span approximately1.452×1.543×2.715 before ADR modelScale0.78.
- ADR exposes33 animation slots, including stand→`wolf_loc_stand.gr2`, run→`wolf_loc_run.gr2` and attack→`wolf_com_h2h_attack_01.gr2`. The36 unique dependency files, including all selected GR2 references, match preserved catalog CRC and native manifest compressed lengths. GR2 bytes are verified, not decoded or animated here.

Original AnimationGroups/AnimationTypes IDs1/3/1001 select `loc_stand`, `loc_run` and `com_h2h_attack_01`, all present in the wolf ADR. The unrelated malformed full AnimationGroups XML prevents a whole-file standard parser; the analysis parses the three exact preserved complete XML elements and records their raw text without modifying the source. The current encounter packet template has uniformScale1, VerticalOffset−0.1, HasTiltfalse and MovementType2. Current spawning changes model/name/position/disposition and base animation override but does not substitute a human skeleton or cyan/orange material.

These facts support the existing original model mapping. They do not prove every native model load, shader, animation state or initial ground contact succeeds. See [[Stage7 NPC Grounding Semantics]] for the separate grounding boundary.

## Static flowers in the screenshot

`wolves-south-approach.png` shows green stems/leaves with cyan flower heads edged orange and dotted pale yellow. The preserved scene places `sg_fairy_flora_01_cluster03.adr`, object1254768278, at `[-2100.5867,-53.00537,425.19797]`, scale1.44—2.71 horizontal units from the reported player/wolf-area point `(-2098,426)`. Nearby original medium trees are at `(-2095.53,428.2731)` and `(-2095.6287,433.68762)`.

The flora ADR resolves to `sg_fairy_flora_01_cluster_03_lod0.dme`, `sg_fairy_flora_01_cluster03.dma`, `SG_FAIRY_FLORA_01.DDS` and its glow texture. Its original diffuse DDS shows the exact cyan/orange spotted heads and green leaf shapes observed. The model has758 vertices, no skeleton/animation bank, and height2.224 before sceneScale1.44. This explains the tall floral silhouettes without assigning them to network enemy GUIDs.

Independent cooking_restore review viewed the screenshot and both original texture previews, checked the original scene placement/scale, original model/community registration and all36 wolf plus5 flora dependency SHA values, and agrees with the narrow conclusion. Attribution to this exact scene object instance is inferred from nearby position and matching art; no camera-projection proof was performed. This distinction does not justify treating the visible flowers as enemy models.

`wolves-first-combat.png` is covered by the original news panel and cannot independently identify a wolf. No speculative model replacement, scale adjustment or material repair is justified by these two images. The clear native wolf image and ordinary chase/attack gates were subsequently retained in183551, reviewed above.

Proof: [wolf/flora identity](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-wolf-visual-v2/proof.json), SHA256 `03363625e43c24f36fb0d997dd0ce73f584c2177b346a37fa01e5263cbbe6f86`. Original texture previews: [flora](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-wolf-visual-v2/sg_fairy_flora_01.dds.png), [wolf](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-wolf-visual-v2/wolf.dds.png). Reproducer `lab/reports/trace-stage7-wolf-visual.py` uses pinned fr-adr-toolkit5acfdf8d6006fa6f43d1d3605f076a6be7393e72 and scene export from ForgeLightToolkit9c06714c5d79d9152c322814bc904abad944e06e. An incomplete initialv1 output is preserved and is not the accepted report.

## Starting respawn to pursued wolves

The first attempt died while idle; root reports ordinary auto-respawn at `(-1904.883,412.6024)`, all five enemy health180 and defeat count0. Pursued wolf positions were reported near `(-2098,426)`. The return guide proposes a different approach around the blocked tree, using expanded original geometry rather than a straight line:

| X | Render floorY | Z |
|---:|---:|---:|
|−1904.883|−39.752|412.6024|
|−1904|−39.810|412|
|−1942|−43.970|446|
|−2002|−43.869|434|
|−2072|−47.356|434|
|−2098|−49.341|444|
|−2108|−51.698|432|

Total226.725 units, maximum sampled centerline grade0.36203, corridor normalY≥0.91178 and selected ground-band render-footprint clearance≥1.58886. The expanded scene contains153 static objects and43 successfully exported original DME meshes. Three AGR appearance wrappers were excluded from the collision proxy; their origins are at least27.679 units from the proposed path, with positions and distances disclosed. Route-only `gl_cattail_01.adr` rawCRC matches, while re-encoded compressed length differs from native manifest; this is explicitly recorded and does not affect the raw geometry query. Wolf/flora closure compressed lengths all match.

The final point is11.66 horizontal units from the reported pursued wolves and is **not an idle safety zone**. Inspect current pursuit positions and use normal input promptly. Y is render floor, not an instruction to set altitude. All candidate segments need bounded actual walking and fresh screenshot/position checks: the community GCNK reader does not decode Bullet collision, and these conservative render-footprint measurements are not a player capsule guarantee.

Guide: [respawn route](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-wolf-visual-v2/respawn-route-guide.json), SHA256 `39991f33f90fff14afcb9dcac8c7619a70d4e6899960b82959622e67d3e32a6f`. Reproducer `lab/reports/analyze-starting-respawn-route.py` reuses the existing checked analysis mathematics and the unchanged original community reader. [[Stage7 Native Return Routes]] adds the tree-avoiding return proposal and recovered actual151602 Pappy climb; the earlier geometry reference remains [[Farnum Stage7 Route Geometry]].

## Short view approach to the later collapsed wolves

For root-reported player `(-2002,434)` and wolves near `(-2072,434…436)`, the short candidate `(-2002,434) → (-2030,434) → (-2050,434) → (-2060,434)` has maximum grade0.315460, corridor normalY at least0.934108, selected ground-footprint clearance at least2.479167 and no missing sampled floor. Original scaled wolf local mesh height is1.203749; this is an asset bound, not an observed native pose or pixel measurement.

With an explicitly assumed eye at floor+1.5 and target body at floor+0.8, the70m sight line from−2002 crosses a terrain ridge by about3.0m. Missing wolf pixels from that distant position therefore do not independently establish an asset failure. From−2050 (22m away) and−2060 (12m), those assumed sight lines clear the sampled floor by at least0.4133 and0.6667m respectively. A giant tree's full canopy AABB overlaps them; the proof deliberately does not treat a whole-tree box as opaque rendered triangle occlusion or claim clear actual camera LOS. Camera projection, actual wolfY/pose, alpha materials and visible rendering remain separate runtime observations.

Proof: [short approach/view assumptions](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/stage7-wolf-short-approach-v1/guide.json), SHA256 `8db7306f739df4baee8a79dcea5e100f7056235f8debc31f0eb5cacff19de2e5`. This checked existing original exports and their source hashes; no new native input, source, content or save changes. Root subsequently completed ordinary combat from this eastern side; the retained video reviewed above supplies the previously missing close-up evidence.
