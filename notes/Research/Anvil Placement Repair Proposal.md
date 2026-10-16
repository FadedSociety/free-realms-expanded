# Anvil Placement Repair Proposal

**Placement approved; new private package prepared, active migration pending.** The initial read-only proposal below was reviewed and root authorized a new variant plus guarded migration helper. No active save, native run, Rust source, release or input changed. This is a local reconstructed placement using original farm geometry, not a recovered historical quest coordinate.

Native run173814 shows the Anvil interaction prompt while a large blue boulder obscures the model. Its existing X/Z `(-2084,190)` lies within original boulder02 object1254768169's render footprint, approximately X−2086.958…−2080.046/Z187.599…193.821. The prompt proves access remains possible; this is a visual and approach limitation, not proof that the activity is unreachable. See [[Reviews/Native Forging Layout 173814]].

## Exact proposed change

Change only `opening.json` `/actors/16/position`, after checking GUID5715144128442925073:

```json
[-2084.0,-55.6,190.0] → [-2088.0,-54.79,200.0]
```

The actor retains definition3600, name36164, original model273 `sg_anvil_stump_01.adr`, heading metadata−1.007059, interaction radius12, recipe2003, Blacksmith/profile16 gate and every other field. Every other opening field and actor remains equal. The proposal script reverses this one delta and asserts whole-document equality.

At the proposed registration, preserved original render terrain has floorY−54.844223 and normalY0.99795134. NetworkY−54.79 is floor+0.054223. Its move is 10.7703 horizontal units. Original mesh local bounds are X[−0.438217,0.406804], Y[0,0.989298], Z[−0.408804,0.439719], scale1. Support-footprint floor spans −54.937171…−54.827163; network base is 0.037163 above the highest sampled support floor.

Current spawning copies the combat template direction, yaw−0.023182, rather than serializing actor heading metadata. Both remain unchanged. Clearance uses a conservative all-yaw horizontal radius0.443669: nearest selected static footprint clearance is 5.37337 to fallen log01, then 6.35866 to fallen log02, 6.59426 to the boulder and 6.64990 to the original forge. Dynamic center distances are 9.641 to Cleetus, 15.620 to furnace, 17.464 to ore and 23.594 to cooking table. The new point is separate from the original static scene anvil at `[-2078.2856,-55.983376,198.6523]`, object1360103621.

The unmodified community GCNK reader does not decode Bullet collision. Ground-band render footprints and slopes are obstacle proxies. Preserved network VerticalOffset−0.1 and initial native grounding may affect visible contact; neither is changed or declared accepted here.

## Provenance and reproducible proof

Proof: [proposal JSON](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/anvil-placement-proposal-v1/proposal.json), SHA256 `91584f4d257f1529fecdfba324beed32c777f4ba92b052a876e74bf1fe9d5bc5`. Reproducer: `lab/reports/propose-anvil-placement.py`; it refuses an existing output directory. All22 direct inputs and64 inherited route inputs were hash-checked again after generation. The old opening bytes still hash to `3e6f4228efc76460b8b4ff5c9ecefb36264e2caf308da7d0cc7fc3ea8e34229e`.

Original model registration is the preserved Models.txt row273; Sanctuary NPC3600 independently identifies model273/name36164. The original five-file ADR/DME/DMA/DDS/CDT closure matches catalog CRC and native manifest compressed lengths. Unmodified fr-adr-toolkit ADR/DMA/DME readers accept applicable files; CDT is preserved CRC-verified raw data, not a newly decoded collision format.

Pinned community revisions:

- ForgeLightToolkit `9c06714c5d79d9152c322814bc904abad944e06e` for original terrain/MAP/DME geometry.
- fr-adr-toolkit `5acfdf8d6006fa6f43d1d3605f076a6be7393e72` for ADR/DMA/DME dependencies.
- Sanctuary `4f028dcc274f628b65ef665fd59b184f5f3d3e3e` for NPC identity.

## Future content and migration boundary

Only a root-approved **new** content/package copy may apply this proposal. The proposed canonical opening hash, computed in memory, is `4e6ac15cd19b5785a04ac7eb944bf0235f4a0592f4c42df755f89c47896b0ad1`.

Alongside the exact actor-position change, update only operational `opening_sha256` bindings in `forging-campaign.json`, `cooking-campaign.json` and `journal-text.json`. Replacing the mapping's one existing hash preserves all other bytes and yields mapping SHA256 `ed985d533add4261c8179176f3803a35cb0281296aafe9b644c0f8bfb94538f3`. A new locale overlay manifest must pin the new opening, mapping and approved source-content path; preserve locale DAT/DIR, roundtrip and proposed strings exactly. Character-select overlay remains identical.

Do not rewrite historical `encounters.json` provenance, prior-opening snapshots or ancestral repair proofs. Enemy content is unchanged; changing its provenance would unnecessarily change the encounter content hash.

A future migration may change only `/gameplay/opening/content_sha256` from the current opening hash to the proposed one. No source snapshot is selected. Require all services stopped, exact immutable version2 snapshot/hash, pinned source/new content, and no active Cooking/Smelting/Forging/encounter. Preserve every other serialized field, including unknown fields, character position/identity, progress, discoveries, wallet, item IDs/gear/rentals, profiles/XP, activity history, health and enemy state. Write a new destination only and retain old snapshot/content/overlay together for rollback. The early Cooking migrator requires cef512 plus untouched stage4 onward and **cannot migrate this progressed placement change**.

## Prepared version and migration checks

New package: `lab/references/opening-anvil-placement-v1`; use its `content` subdirectory as the harness `--content` and its package root as `--journal-text`. Root keeps current181715 on old content until the next safe boundary. Package marker `anvil-placement.json` SHA256 is `b78532891c30ee5e12dea8b6c7facc7132c4f8fde158065cdf2efb5efd96d86f`.

New helper [anvil_placement.py](%USERPROFILE%/Documents/Projects/CartoonMMO/project/scripts/anvil_placement.py) exposes `prepare`, `verify` and `migrate`; the latter accepts `--direction forward` or `rollback`, an explicit immutable source SHA and a new destination. It checks stopped native/gateway processes twice and rejects active activities, inconsistent progression, duplicate JSON keys, nonfinite constants and ambiguous repeated hashes. Forward migration replaces exactly one64-character hash in original snapshot bytes; reverse equality proves every other byte, including unknown future fields, survives. Existing identical output/proof is idempotent; mismatched output/proof is rejected before writes. The helper never rewrites the source.

Package validation rederives all36 exact expected output files from a pinned35-file original source inventory, SHA256 `c851cedc5ac6e5ad379c51bcb301ef2ab11adff604c4dfdd1403ee97b4e69d7b`. It rejects missing, additional or changed files and preserves old package bytes. The original `validate_journal_text` launch validator accepts the paired new package/content.

Offline proof: [package/migration checks](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/anvil-placement-proposal-v1/offline-package-migration-proof.json), SHA256 `b430028c614148c07746038f799092af94b80d17e417ab2dc2683798016be6f2`. [check-anvil-placement.py](%USERPROFILE%/Documents/Projects/CartoonMMO/project/scripts/check-anvil-placement.py) passes a six-completed-stages fixture, exact byte rollback, isolated filesystem retry/source-preservation/race/proof guards, locale launch validation and44 rejection cases. Filesystem tests explicitly stub stopped-process checking only inside disposable synthetic fixtures; actual guard rejection is tested separately. No campaign snapshot was read or migrated. Independent helper/package review and Rust runtime restore acceptance are recorded separately from this offline proof.

Independent cooking_restore review reports no blocking findings: [review proof](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/anvil-placement-independent-review.json), SHA256 `7d7c2ffa918108f8d5d9736dae82ef23997238409ccc02f39267201f9538270c`. It verifies the exact36/35-file transformation, original locale/encounter preservation, independently reruns44 rejection checks, and adds17 negative state cases plus raw-byte roundtrip with unknown fields and absent optional components. Real snapshot migration, Rust runtime restore and native render/approach remain pending.

Root subsequently applied the migration at the safe boundary. Independent review [actual migration proof](%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/anvil-migration-independent-review.json), SHA256 `d84783afa0232868d12c3eff2d286d6653ccd86d4aca575c62c52c2b448215ec`, verifies source3a3b159d→output0cc3496c is exactly one raw hash replacement with byte-exact rollback and unchanged identity/progress/economy. Existing Rust restore and full17 mapped campaign gates pass10/10. Native placement/interaction and walking acceptance remain separate.

## Native walk candidate to stage7 wolves

Use a fresh screenshot and actual player position before departure. The old173814 endpoint `(-2083.8594,194.05472)` is only0.272 units from the boulder render footprint. Its candidate exit via `(-2085,197)` to `(-2085,200)` is not certified collision clearance from that initial contact. Move away north/west in short normal steps, then establish the clear standing point.

The proposed interaction standing point `(-2085,200)` has render floorY−55.240972, 3D distance3.034 to the proposed anvil, and places the furnace outside its12-unit radius. Its connector to `(-2084,202)` has maximum sampled grade0.077223, corridor normalY≥0.974064 and selected log clearance2.65078. Continue the existing guide:

| X | Render floorY | Z | Purpose |
|---:|---:|---:|---|
|−2085|−55.241|200|Clear standing/interaction point|
|−2084|−55.068|202|Join checked stage7 route|
|−2080|−56.426|272|Between farm fence and corral/stump|
|−2084|−55.823|342|Eastern side of campsite|
|−2088|−56.982|370|Clear campsite|
|−2108|−50.215|440|Southwest wolf approach|
|−2106|−48.125|448|Pause and inspect wolves|
|−2100.965|−46.034|456.7631|First wolf approach|

Y values describe original render floor, not an instruction to set player altitude. The inherited main route has ≥1.558 selected render-footprint clearance; inspect the campsite and stop if native collision contradicts it. See [[Farnum Stage7 Route Geometry]] and [[Stage7 NPC Grounding Semantics]]. The wolf-to-Pappy return remains unresolved at the western cliff/fence approach and is not promoted to a safe blind route.

## Acceptance still required

- Visible anvil clear of boulder, grounded sensibly and approachable normally.
- Normal X/menu/Blacksmith gate and genuine Forging139 start, with no duplicate material grant.
- Independent completion of full Forging win/results/player Close/quest turn-in/restart after the Windows save-lock repair.
- Actual walking from clear camp staging through the wolf route. No geometry-only claim substitutes for this.
