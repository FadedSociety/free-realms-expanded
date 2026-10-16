# Stage7 NPC Grounding Semantics

Bounded read-only follow-up to [[Farnum Stage7 Route Geometry]]. **The original native client can replace a physics NPC's network target Y with a collision-hit height during movement. The five copied spawn heights therefore do not prove visible burial.** Initial stationary-spawn grounding remains unproved; actual wolf visibility, chase and combat contact are still native gates.

## Original and community evidence

The preserved original executable is SHA256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`. Original standalone movement125 reader/handler `008e5940`/`0090de90` leads to `00963840` for MovementType2. That function retains the received position at entity+0xf50, copies it into a target, and calls `010ba6e0` unless entity+0x145 bit8 is set. It then passes the potentially adjusted target to queue function `0095e890`.

`010ba6e0` casts vertically at the incoming X/Z from **Y+2 down to Y−2000**, using collision query `00fd4290` with filter `0x143`. On a hit it replaces only targetY with the returned collision-hitY. On no hit it returns0 without changing Y; its caller does not branch on that result. Constants 2.0 and2002.0 and the actual call/conditional/store instructions are independently checked against original PE bytes with Capstone. The exact query's surface-selection/filter semantics have not been exhaustively traced. Render terrain is not assumed identical to collision.

Native AddNpc reader `00921a10` places the corresponding boolean at packet-structure offset0x113; AddNpc dispatch `0092f460` sets entity+0x145 bit8 from that field. Unmodified pinned Sanctuary writer revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e` calls the field **HasTilt**. Five new private compiled vectors distinguish HasTilt from adjacent Unknown42 and the separate VerticalOffset and MovementType fields. The empty334-byte wire offsets are HasTilt175, Unknown42174, VerticalOffset94 and MovementType233. String/list lengths shift later offsets; these constants are only for the verified empty template.

Current `rust-opening-cooking-original-assets/encounter-template.bin` has HasTilt=false, Unknown42=false, TerrainObjectId0, MovementType2 and **VerticalOffset−0.10000000149**. `Encounter::spawn` supplies actor XYZ and explicit locomotion/speed while preserving those tilt/offset values. HasTilt is not a proposed fix switch. Original debug function `008df1bb` names entity+0x670 `GetVerticalOffset`; AddNpc dispatch assigns that field separately from XYZ. Its complete per-frame rendering effect was not resolved by this bounded task, so it must not be substituted for a measured ground-contact correction.

All five previously measured wolf render floors fall within the native target ray's vertical range. Even wolf5's higher floor is approximately0.223 below the common ray start. This supports a plausible native grounding mechanism, conditional on the actual collision hit, actor state and movement path; it is not an observed raycast or visible acceptance.

## Initial spawn and authoritative state limits

Initial AddNpc creation `0092f460 → 00985340 → 00968c80` passes the network XYZ into model/controller creation and includes tilt-dependent creation booleans. The complete implementation behind those creation interfaces was not traced. **Do not claim the initial stationary spawn is automatically grounded, or that the differing Y values are harmless in every state.** Observe the wolves before their first chase as well as during movement.

The Rust encounter AI computes 3D distance and interpolates all three authoritative coordinates toward the player. The client's collision-height adjustment does not rewrite that server state. Correct visible grounding would therefore not by itself prove correct server hit reach, pursuit terrain handling or saved heights. No such defect is asserted without a native distinguishing result.

## Possible repair boundary, only after a native defect

- If stationary wolves visibly intersect terrain and the actual contact/target evidence identifies spawn placement, review a new versioned content-only Y correction for the affected wolves. Preserve X/Z, GUIDs, models, packet flags, health, objective counts and behavior. Select collision-consistent heights and the established model offset from evidence; do not assume a generic floor+0.05 rule.
- If appearance is correct but server reach or pursuit height is wrong, review authoritative encounter-state height separately. A tilt flag or visual offset change does not repair a proved 3D distance error.
- Existing snapshots/content remain immutable. Any future content correction requires a separately authorized compatibility/migration review, especially for already moved or defeated enemy states. **No migration has been prepared.**

## Reproducible private proof

`../lab/reports/stage7-npc-grounding/proof.json`, SHA256 `de90cf11e77ea680d6d611711134d11a84abb14046d8415c95473d735bfd9e45`, pins20 inputs, five compiled original-writer vectors and287 original instruction bytes. `native-grounding-functions.json` and `native-grounding-dispatch.txt` preserve bounded original excerpts with source line numbers. New helper `../lab/reports/prove-stage7-npc-grounding.py` asserts the source relationships, PE constants/instructions and exact template fields, refusing to overwrite existing proof. New private `../lab/tools/npc-grounding-oracle/` references the existing assembly built from unmodified community writer sources; original serializers were not rewritten or edited.

No game/server source, content, save, native input, active-run file or shared progress file changed. This is static source/compiled-writer evidence only.
