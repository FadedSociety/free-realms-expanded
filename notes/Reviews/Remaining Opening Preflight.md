# Remaining Opening Preflight

October 8, 2026. Read-only preflight for local opening stages 13-17. It verifies bindings, identifies the exact acceptance gates and locates the existing private helpers. **It is not a native acceptance claim.** No save, content, source, service or user input was changed by this review.

## Verified bindings

- Queen source `lab/state/opening-after-native-queen-turnin-203139.json`, SHA-256 `3a9d38078449b7dd95fdd8263a733b9d77e44b24ffe740d1e0a44deff07e2681`. Stage 13 is unaccepted; stages 1-12 are completed and rewarded; discoveries 1-4; 259 coins; 30 item instances.
- Bound content `lab/references/opening-anvil-placement-v1/content`: opening.json `4e6ac15c…0ad1` equals the save's `content_sha256`; cooking `60d208f5…`, smelting `a06f9062…`, forging `20cc3ada…`, encounters `2d465c7e…`, profiles `d963fe7d…` all match the reviewed hashes.
- Live run `client-baseline-20261008T235041Z` (branch save `lab/state/opening-continuation-20261008T235029Z/character.json`): startup already reports the same 12/17 stage state, unlocks `[1,14,16,43,45]`, cooked/smelted/forged completion counters and 30 inventory instances. A canonical-JSON comparison against the Queen source with `character.position/rotation` normalized is byte-identical (`32ed5aa7…`), so the branch had only walked by 16:52 local.
- Field-level diff of the older `opening-journal-text-v2` content against the bound `opening-anvil-placement-v1` content: the **anvil position is the only difference** in the whole opening content. The stage5-17 guide remains valid except for that one coordinate.

## Stage 13-17 acceptance gates

Interaction reach is 12 units in 3D; one actual menu choice performs one action; objectives are ordered and rewards only exist at turn-in. Use the native Jobs menu and native equipment UI.

| Stage | Accept / turn in | Ordered actions | Expected inventory/economy |
| --- | --- | --- | --- |
| 13 Second Helping | talk Mae `…5060` again | Chef45 → harvest Rainleaf ×4 (actor `…5068`, resource1001) → Gummyfruit ×4 (actor `…5069`, resource1002) → win original Cooking (`2001` at table `…5070`) | accept grants Bat Meat2862×4 + Spice2900×4 once; each counted harvest grants 2884/2875 ×1; the win consumes 4×2862/2875/2884/2900 and grants Stew35×1; turn-in **+10 coins** |
| 14 Copper Collection | Cleetus `…5061` | Miner14 → mine Copper Ore ×10 (node `…5071`, resource1003) | each newly counted increment grants Ore2739×1 + Coal879×1; turn-in **+10 coins** |
| 15 Forging for Reinforcements | Cleetus `…5061` | Miner14 → win Smelting (`2002` at smelter `…5072`) → Blacksmith16 → win Forging (`2003` at the **moved anvil** `…5073`, `[-2088,-54.79,200]`) | accept grants once Turquoise2752×5, Planks55×5, Fasteners56×5 **and** Ore2739×5 + Coal879×3 (smelting inputs, not free bars); smelt consumes 2739×5+879×3 and grants Bars21785×10 (objective capped at 5); forge consumes 21785×10+2752×5+55×5+56×5 and grants Hammer30280×1; turn-in **+10 coins** |
| 16 Visiting the Job Center | accept at Queen `…5063`; turn in at Captain Sunspark `…5083` | talk Ellandra `…5080` → Maximus `…5081` → Scotty `…5082` → Sunspark `…5083` | acceptance unlocks Medic11 + Archer35 (and their starter items) before the tour; turn-in **+10 coins** |
| 17 Talk to Captain Sunspark | accept and turn in at Sunspark `…5083` | switching to Adventurer1 → Medic11 → Archer35 → Chef45, then equip Student Chef Hat `14794` while Chef | hat is owned (instance id 16) and **not** equipped on Chef45; turn-in **+10 coins** and 17/17 |

Server triggers: class select `26/13`, equip `42/3`/`42/8`, craft completion validated for job + station + ordered pending goal. Every reward also adds 25 XP to the active profile. Recipe ids/consumes are bound in the content files; fixed 10-bar smelt output, one-food cooking output and forged tint selection stay disclosed reconstructions.

## Findings that can block native completion

1. **Moved anvil.** Visibility and the ordinary job-gated interaction were accepted at `[-2088,-54.79,200]`, but the repeat forging has not been replayed there; stage15 needs that exact win.
2. **Medic/Archer switching is unverified natively** after the stage16 unlock; only packet-level simulation exists. Their starter-item definitions and the Archer slot-1 equipment row are present.
3. **Native hat equip is unverified.** Definition 14794 is the bound prepared variant (four byte offsets differ from the unmodified legacy resource), so this review does not claim it equals the original.
4. **Return travel is unaccepted.** Only Bartle→Queen has a route receipt; the farm→Sanctuary/job-center walk still needs native footing evidence.
5. **Persistence/duplicate check after stage17 is open**, including the ordinary crafting Quit path (separately owned by the other builder).
6. **Source drift note:** the repo has no git commits; `opening.rs` is unchanged since the guide (`08cf5bf3…`), while canonical `gameplay.rs` gained an 88-line `#[cfg(test)] mod ordinary_quit_routing` (hash `d076f963…` vs the pre-change snapshot `78845ac5…`). The diff is test-only, so stages13-17 release behavior is unchanged.

## Existing private helpers (reuse, do not reinvent)

- `lab/tools/navigate-local-game.py --run <lab/runs/…> --state <lab/state/…> --target X Z --expected-zone 0 [--radius 2.5 --turn-rate 1.45 --speed 8.5 --max-seconds 90 --max-distance 150] [--execute]` — default prints one proposal and sends no input; refuses targets beyond the distance bound (the Queen-side start is ~1,000 m from the farm, so the trip is segmented).
- `project/scripts/drive-local-game.py --run <lab run dir> "scanmode on" "hold 0x57 300" …` — recorded ordinary input; appends `external-inputs.jsonl` and regenerates `lab/tools/WinDrive-Dpi.ps1` (physical-pixel DPI).
- `lab/tools/fight-local-wolves.py --run … --state … --target-kind wolves|gloam|mac --modals-cleared [--max-seconds ≤30]` — ordinary-key combat on a reviewed clear route.
- `lab/tools/record-native-session.py <base> --seconds N --pid <FreeRealms pid>` — bounded recording; the adjacent `.stop` file ends it.
- `lab/tools/plan-forging-chain.py <snapshot.json> [--limit --max-length --anchor X Y --step DX DY]` — read-only legal-chain suggestion from a saved board.
- Offline oracles (no UI): `lab/tools/native-crafting-oracle`, `cooking-oracle`, `forging-oracle`, `smelting-oracle`, `login-oracle`, `self-identity-oracle`.
- Stage-by-stage route/action guide: `lab/reports/opening-stages5-17-native-guide.md` (SHA-256 `ad68bf99…`), valid except the stale anvil coordinate `(-2084,-55.6,190)` → use `(-2088,-54.79,200)`.

## Immutable checkpoints

| Checkpoint | SHA-256 |
| --- | --- |
| opening-after-native-queen-turnin-203139.json | `3a9d3807…e2681` (identical to the repeat and active anvil save) |
| opening-after-native-queen-discovery-203139.json | `0314f40f…4fd9c` |
| opening-after-native-queen-visit-203139.json | `b970e991…f0315` |
| opening-after-native-restart-203139.json | `b7265056…2e72c` |
| opening-after-native-mac-turnin-193318.json | `f2e3e2b8…6ec10` |
| opening-after-native-artifact-turnin-193318.json | `4db15a0d…f2489b` |
| opening-after-native-bartle-turnin-193318.json | `8006aef2…45bf8a` |
| opening-after-native-gloam-turnin-183551.json | `aff06523…edb3bc` |
| opening-after-native-wolf-turnin-183551.json | `c24b9d8d…d7347d` |
| opening-after-native-forging-turnin-181715.json | `e43e8d86…c4cdc49` |

## Evidence

Read-only verifier (43 checks on the Queen checkpoint, 42 on the live branch, all passing) and navigation proposals: `../../../lab/reports/opening-preflight-20261008T2351Z-flashmax/` (`verify-opening-preflight.py`, `verify-queen-checkpoint.json`, `verify-live-branch.json`, `nav-live-to-sunspark.json`, `nav-farm-to-mae.json`, `nav-farm-to-anvil.json`, `proof.json`). Live run: `../../../lab/runs/client-baseline-20261008T235041Z/server-stdout.log`.

See [[../Current Progress]] for live status and [[../Research/Opening Historical Coverage Ledger]] for the historical boundary.
