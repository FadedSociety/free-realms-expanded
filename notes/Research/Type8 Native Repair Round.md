# Type8 native repair round

Status 2026-10-09: Harvesting111 baskets now use the recovered original produce
contract (`value = 400 + produce type` -> `farmingValue401..416` name and
`SelectableTile1..16` icon), the shared Type8 engine counts selectable-valued
goals on accepted chains, and the writer refuses the old 600-block baskets that
rendered `##farmingValue60…`. Focused and full Rust suites pass. **No native
acceptance is claimed here**; the lead owns the in-game run.

Owned artifacts: `prototype/osfr-tools/src/mining.rs`,
`prototype/osfr-tools/src/harvesting.rs`,
`prototype/osfr-tools/tests/type8_native_repairs.rs`, this note and
`../lab/reports/type8-native-repair-20261009/`. Mechanically updated existing
harvesting test files are listed under "Test updates".

## Symptom and root cause

Native run `client-baseline-20261009T202011Z` (original Chef Harvesting111
board, screenshot
`../lab/reports/delegated-native-audit-20261009T200750Z/harvesting-placeholder-goals.jpg`)
showed four required baskets with unresolved `##farmingValue60…` text and blank
produce icons.

The client resolves a basket label and icon from the basket's tile value:

| Step | Recovered original | Effect |
| --- | --- | --- |
| Name | `GameClient.as::OnQuotaInitMsg` calls `LookupString(GameData.sGAME + "Value" + value)`; `SoeStringLookupFreeRealms.GetStringByName` returns `"##" + name` when the host never supplies the string | `value 601..604` -> `##farmingValue601…`, because the shipped client has no 600-block farming key |
| Icon | `Bin.SetQuota` builds the icon with `value` through `core/TileGenerator.as::SetTileInfo(value, index)`, which resolves selectables as `"SelectableTile" + index` and collectables as `aCollectableNames[index * 2]` | The farming movie labels only `SelectableTile1..16` (JPEXS frame inventory of `game_farming.gfx`); a 600-block value has no farming frame, so the icon stays blank |
| Counting | `GameLevel.QuotasInit` sends one quota per spawn entry with `quota != 0` (selectables first); `HandleSelectableTile` counts every collected chain tile and sends `OnQuotaUpdateMsg` | A farming basket must be a *selectable* produce value, not a collectable |

The writer emitted mining-style collectables: `selectable = [401..404]`,
`collectable = [601..604]`, all four goals on 601..604. That is the defect
repaired here.

## Recovered original contract

The client's own `Resources/CodeStringMappings.txt` (working copy
`../lab/working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt`,
lines 4784-4799) maps exactly sixteen farming values, and the stock
`locale/en_us_data.dat` resolves those ids through the pinned community reader
(`../lab/references/FreeRealmsLocaleTools`, strict `LocaleFileInfo`):

| value | code string | locale id | resolved name | movie frame |
| --- | --- | --- | --- | --- |
| 401 | `farmingValue401` | 117115 | Gummyfruit | `SelectableTile1` |
| 402 | `farmingValue402` | 69328 | Snowberry | `SelectableTile2` |
| 403 | `farmingValue403` | 5398 | Spiralmint | `SelectableTile3` |
| 404 | `farmingValue404` | 117118 | Moonbean | `SelectableTile4` |
| 405 | `farmingValue405` | 117119 | Choychoy | `SelectableTile5` |
| 406 | `farmingValue406` | 69333 | Sunleaf | `SelectableTile6` |
| 407 | `farmingValue407` | 117121 | Honeytwig | `SelectableTile7` |
| 408 | `farmingValue408` | 117122 | Punkin | `SelectableTile8` |
| 409 | `farmingValue409` | 69334 | Briarmelon | `SelectableTile9` |
| 410 | `farmingValue410` | 69332 | Rainleaf | `SelectableTile10` |
| 411 | `farmingValue411` | 117125 | Stickfruit | `SelectableTile11` |
| 412 | `farmingValue412` | 117126 | Starnut | `SelectableTile12` |
| 413 | `farmingValue413` | 69329 | Stainberry | `SelectableTile13` |
| 414 | `farmingValue414` | 69330 | Chugaroot | `SelectableTile14` |
| 415 | `farmingValue415` | 69327 | Bumbleberry | `SelectableTile15` |
| 416 | `farmingValue416` | 117130 | Tumtumnip | `SelectableTile16` |

The order is identical to the bundle locale ids `23285..23300`
(`{n}Bundle of <produce>`) and to the module's `PRODUCE` table
(item definitions 2875..2890), so one value carries the name, the icon and the
bundle. The region's four produce become four selectable values, e.g. game 111
Farnum's Farm (stickfruit, stainberry, tumtumnip, spiralmint) uses
411, 413, 416, 403.

Cross-check: the pinned wiki screenshot of Lakesfield (`94153`) shows bins
Punkin/Rainleaf/Sunleaf/Gummyfruit with their icons; the recovered names/icons
cover it. The screenshot's middle two slots are swapped relative to the locale
prose order, so the original *slot* order stays an unrecovered gap (below).

Evidence commands: the locale dump and probing live in
`../lab/reports/type8-native-repair-20261009/` (`locale-en-us.json` plus
`logs/locale-probe.txt`); the frame labels come from the pinned JPEXS 26.3.0
export `../lab/decompiled/minigames/harvesting/game_farming.xml`.

## Repair

`src/harvesting.rs`

- New recovered helpers `produce_value`, `produce_index` and
  `produce_value_for` implement `value = 400 + 1-based produce type`; the old
  `REGULAR_TILE_VALUES = [401..404]` placeholder is gone.
- `board_spec` emits exactly the region's four produce values as both the
  selectable spawn set and the goals/outputs; the tile list carries four
  category-400 kinds whose `value - category` is the movie frame; `collectable`
  is empty and `RECONSTRUCTED_COLLECTABLE_PERCENT` is 0 (no `farmingValue`
  names exist in the 600 block).
- `validate_content` now refuses a goal outside the recovered produce values,
  a goal whose selectable kind is not registered, an unnamed 600-block
  collectable, or a selectable set that is not one-per-produce.
- `evaluate` reports produce values; the rule notice cites the recovered
  `farmingValue401..416` / `SelectableTile1..16` contract.

`src/mining.rs` (shared Type8 engine)

- `validate_spec` accepts a goal (and its output) on a registered selectable or
  collectable value; the collectable list may be empty.
- `validate_chain` returns the chain's matching value; the chain path counts
  every collected tile into a matching goal (`collect_chain_goals`) and emits
  `OnQuotaUpdateMsg`, mirroring `GameLevel.HandleSelectableTile`. Wildcards
  count as the adopted chain value (`ServerGrid.doMatch3` sets the chain value
  before collection). Selectable collection adds no per-tile score, as in the
  original handler.
- The completion/reshuffle tail is shared (`settle_after_collect`), so a chain
  that completes every quota stops the level exactly like a fall does.
- Mining18 behaviour is unchanged: its goals are 600-block values, a 400-block
  chain value never matches them, and falls still score and complete.

Repaired revisions: `src/harvesting.rs` `9e4927bc…`,
`src/mining.rs` `ddc22715…`; every touched file's full SHA-256 is in
`../lab/reports/type8-native-repair-20261009/final-hashes.txt`.

## Test updates (mechanical contract flip, disclosed)

These existing harvesting test files asserted the 601..604 writer contract and
were updated to the recovered produce values while keeping every guard's
intent: `tests/harvesting.rs` (contract mirror, 401/410/415/407), 
`tests/harvesting_border.rs`, `tests/harvesting_review.rs`,
`tests/harvesting_runtime_tiles.rs`, `tests/harvesting_tile_registration.rs`.
The registration/border/runtime guards themselves and the mining starter are
untouched. `rustfmt` normalized those files, plus two pre-existing unformatted
powerup hunks in `src/mining.rs` (lines 1103, 1978) that predate this repair
(identical in the pinned mining-clickable harness copy).

## Checks

| Check | Result | Log |
| --- | --- | --- |
| RED before the repair | `type8_native_repairs` 7 failed / 1 passed, naming the 601 goals, the empty collectable expectation, the `OnTileInfoMsg` set and the engine refusals | `logs/red-type8_native_repairs.txt` |
| GREEN after the repair | `type8_native_repairs` 8 passed / 0 failed | `logs/green-type8_native_repairs.txt` |
| Harvesting family | 17 + 3 + 3 + 4 + 4 + 2 + 3 + 9 passed, 0 failed | `logs/focused-harvesting-family.txt` |
| Mining + gateways | 21 + 3 + 6 + 3(+1 ignored) + 22 + 13 + 4 + 3 + 10 + 11 passed, 0 failed | `logs/focused-mining-gateway.txt` |
| Gameplay/economy bindings | 18 + 11(+2 ignored) passed; `gameplay` 0 executed (1 ignored, pre-existing) | `logs/focused-gameplay-piece018.txt` |
| Full suite | 169 test targets, 0 failures | `logs/full-suite.txt` |
| Scoped formatting | `rustfmt --edition 2024 --check` clean on all eight touched Rust files | `logs/fmt-check.txt` |
| Content probe | Writer emits region 111 goals 411/413/416/403, tiles 411-11/413-13/416-16/403-3, outputs 2885/2887/2890/2877, and the engine emits the same four `OnTileInfoMsg` pairs | `logs/content-probe.txt`, `content-probe.json` |

## Lead native integration (exact)

1. Content: copy
   `../lab/reports/type8-native-repair-20261009/native-package/content/harvesting.json`
   (SHA-256 `bfe6897fe00aac118b0c9c92c8528e91cc4ef713e1e3b0fdd61f1690e8e42532`)
   over the `harvesting.json` in the campaign content directory the gateway is
   launched with. Keep the campaign's `inventory.json`, `opening.json`, save and
   every other file; the four bundle definitions (region 111: 2885, 2887, 2890,
   2877) must stay present in that inventory.
2. Do **not** reuse a 601-era `harvesting.json`: the repaired writer refuses
   those goals and the client cannot resolve their names.
3. Launch as before, select Chef, enter Harvesting111. Expected basket labels
   `Stickfruit / Stainberry / Tumtumnip / Spiralmint` with produce icons and no
   `##farmingValue` text, and four `OnTileInfoMsg` pairs
   `411-11, 413-13, 416-16, 403-3` in the capture.
4. A valid three-tile chain of one produce must emit `OnQuotaUpdateMsg` for its
   basket (`0`-based index in send order) and increment the counter; completing
   every quota (15 tiles, 5 per bundle) stops the level and then the normal
   results/`FRServer_GameClose` path releases the run.
5. Recorded chain input remains an unresolved tool limitation from the audit;
   the engine still requires at least three orthogonal same-value tiles
   (`GameData.nMINIMUM_CHAIN_LENGTH`), and this repair does not relax it.

## Leave / Quit investigation (no code change)

Decompiled farming scripts: `GameClient.as:94` wires the in-game Leave button to
`SoeInterface.Quit()`, and `SoeInterfaceFreeRealms.as` turns that into the host
command `fscommand("FR_event","Quit()")`. It is not a server message, so the
engine has nothing to answer; the results close (`FRServer_GameClose`) and the
reported `FRServer_GameQuit` are already handled by the shared engine
(`src/mining.rs`, `FRServer_GameClose`/`FRServer_GameQuit` arms). The failed
native clicks stay a client/tool-side limitation, not a demonstrated server
defect. Escape/`realm exit` behaviour is unchanged.

## Remaining gaps (labelled, not claimed)

- Basket *slot order* follows the recovered locale description order; the wiki
  screenshot shows the middle two Lakesfield produce swapped, so the original
  level table order is still unrecovered. Names and icons per produce are
  unaffected.
- The wildcard-count nuance (an adopted wildcard counts as the chain value) is
  implemented from `ServerGrid.doMatch3`/`HandleWildcardTile` but not natively
  verified.
- Original quotas, timer, spawn percentages and reward grants stay
  reconstructions; the 600-block collectable removal is a reconstruction
  choice, not a historical spawn-rate claim.
- Native acceptance (basket labels/icons, chain input, win, ordinary
  results/Close and reload) remains the lead's gate.

Superseded wording: [[Harvesting Restoration]] ("Goal tiles are `601+slot`"),
[[Harvesting Native Content v2]], [[Harvesting Tile Registration]] and
[[../Reviews/Delegated Harvesting Readiness]] describe the pre-repair
601..604 contract and should be read through this note.

Cross-links: [[../Reviews/Delegated Native Acceptance Audit]],
[[Type8 Mining Gateway Integration]], [[Harvesting Border Fix]].
