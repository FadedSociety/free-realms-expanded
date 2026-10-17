---
type: research
updated: 2026-10-17
task: W84
verdict: PARTLY PROVEN
---

# Item Catalogue

W84 supplies an offline catalogue generator and a byte oracle. **1,703/1,703 stored rows round-trip exactly**, representing **1,649 unique IDs** and 54 duplicate occurrences. Generated definitions cover **16,551/16,551 previously missing content reference occurrences**, including all seven definitions identified by W82. W84 itself changed no runtime content, saves, routing or shared modules; W166 later integrated the generated catalogue into the packaged runtime. A native grant/display gate remains open.

**Runtime correction (W166, `5d522cf`, 2026-10-16).** The packaged runtime loads the union of `inventory.json` and `item-catalogue.json`: **58,835 ids**, not the 1,649 in `inventory.json` alone. `Inventory::load` inserts the catalogue under the captured rows; `content_check.py` checks consumer references and ride coverage against that union; and `scripts/bundle/item_content.py` refuses a bundle whose catalogue is missing or fails the loader's rules. Every 1,649 figure below describes W84's stored-row fixture (1,703 rows with 54 duplicates), not the runtime catalogue.

These are separate claims: decoding and re-encoding the stored rows proves the wire layout; it does not prove that Legacy reproduces the captured values. Independent regeneration from Legacy plus shipped client starter data matches **419/1,703** stored rows. The other **1,284** rows have the mismatch classes below. Known rows retain their existing bytes and metadata; unknown IDs use the corrected source data. This is not a historically complete catalogue.

## Artifacts and source priority

- `scripts/build_item_catalogue.py`: Python stdlib only, run with `python -I`. Imports the existing `prepare-opening-base.py::legacy_definition` and `content_definition`, which use `prepare-gameplay-inventory.py::definition_bytes`. There is no second row encoder.
- `prototype/osfr-tools/tests/item_catalogue_roundtrip.rs`: ignored private oracle using `OSFR_ITEM_CATALOGUE`. Calls the existing `inventory_v2_fixes::encode_definition_batch_v2`; no duplicate batch encoder.
- Private output: `lab/reports/item-catalogue-20261015/`. `catalogue.json` contains 58,835 unique definitions (57,186 additions), serialized bytes, hashes, source tags and `reconstructed_fields`. It is the payload installed as `CONTENT/item-catalogue.json`; merged under `inventory.json` it gives the runtime's 58,835 loadable ids. The 1,703-row fixture behind the 1,649 batch ids is separate, not the runtime total.
- `roundtrip.json`: 1,703 independently decoded/re-encoded rows in original order. The Rust oracle reads the original input independently, verifies its SHA-256 and every row hash, then compares every row byte.
- `report.json`: input paths, hashes and lengths, coverage by source file, source mismatch counts, W82 cross-check. `source-mismatches.json` records every differing field for every stored row occurrence. `client-corrections.json` records all 302 starter-resource overlays.
- Native evidence stays in `native-*.txt` under that private directory. No client bytes, source tables or decompiled code are embedded in the repository files.

Inputs are hashed before analysis; no additional game inputs were acquired. The inventory baseline is `lab/reports/kart-weapons-probe-v20-20261010/content/inventory.json`, SHA-256 `0b95b0b1c032a2e0f864f8227bfe693fe58181863eaf182d35b6b9dbc4d9877a`. The task calls all 1,703 rows captured samples; this note calls them **stored baseline rows**, because previous preparers also added definitions. Their exact preservation does not establish a retail origin for each value.

Legacy data: `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`, 58,795 rows, SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`; checkout `9e082025391177d5a33ee18c714ddd06b457a695`, AGPL-3.0. Data is used under the owner's explicit 2026-10-13 policy. No Legacy source code was copied. Missing fields receive explicit reconstructed zero/false/empty defaults (R). The existing adapter's type-16 preview-disable rule is also reconstructed (R).

Client correction: `lab/working/OSFR.Client/OSFR Client/Resources/CharacterCreate/ItemDefinitions.txt`, SHA-256 `f9a5adf844bef0f10844fb3dd2485333b0d2783f6ceffc8649033dd47d935df7`, 302 rows and 51 named columns plus a trailing separator. Resources are retained in the preserved extracted archive tree under `lab/working`; `lab/extracted/client-assets` contains the separately extracted packed assets. The shipped text table overrides the corresponding Legacy fields. There are **zero conflicts between its mapped columns and overlapping stored baseline rows**; regeneration fails on such a conflict instead of silently keeping a contradictory baseline. Thirty-six client IDs and four baseline IDs are absent from Legacy, accounting for the 40 additional IDs.

Wield metadata for new rows reuses `lab/references/Sanctuary/src/Resources/ItemClasses.txt`, SHA-256 `73c2a9377477ff88e6f4966614930c3d375cd9d5904fddaa7544ec24f0c2f567`, through the existing content helper. Missing class mappings default to zero (R). Ten existing baseline wield overrides are preserved, rather than recomputed from that table.

## Decoded row layout and field provenance

Native evidence is from `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`, and read-only queries of `lab/native-map/run/map.sqlite`. The native map's speculative/unknown stat and extension labels were checked against disassembly. The reviewed [[../Build Guides/Gameplay/Items and Inventory|Items and Inventory guide]] supplies the datasource column cross-reference, including where a semantic name remains reconstructed.

All dwords are little-endian signed 32-bit containers; float fields retain their exact bits in those containers. Flags occupy one byte. Each string is an i32 UTF-8 byte length and that many bytes, with no NUL terminator. No alignment or per-row byte length occurs. With string byte lengths M, T, L, stat count S and ability count A, total row size is **147 + M + T + L + 17S + 16A** bytes. Empty row minimum is 147 bytes. The decoder requires complete consumption, canonical flags, bounded nonnegative lengths/counts and a positive definition ID.

The table lists **every field in stream order**. `+offset` refers to the native destination object, not the wire cursor. Compatibility aliases are retained to reuse the existing writer; an alias alone is not proof of semantics. `(R)` denotes a reconstructed meaning or value policy.

| Wire field / alias | Width | Native destination / meaning and provenance |
|---|---:|---|
| `Id` | 4 | +04; definition/cache key, `008e3760`; datasource `00c956c0` column 0 |
| `NameId`, `DescriptionId` | 4 each | +08, +0c; locale identifiers by existing serializer/client file mapping; exact semantic correspondence remains (R) in the reviewed guide |
| `IconData.Id`, `IconData.TintId` | 4 each | +14, +18; image-set ID and icon tint; `00720cd0`, datasource columns 1 and 0x0d |
| `Tint`, `Unknown5` | 4 each | +1c, +20; **unnamed dwords**, not proven tint/ability semantics (R); `008e3760` |
| `Cost` | 4 | +24; coin purchase price, datasource column 0x19 |
| `Class` | 4 | +28; item class ID, datasource column 0x22 and lookup `01048390` |
| `ProfileOverride` | 4 | +2c; name from source/file-order correspondence (R) |
| `Slot` | 4 | +30; equipment slot, datasource column 0x0a |
| `NoTrade`, `NoSale` | 1 each | +34, +35; first flag meaning (R); no-sale is datasource column 0x1c |
| `ModelName`, `TextureAlias` | length + bytes each | +3c and +4c strings; datasource columns 0x0b and 0x0c |
| `GenderUsage`, `Type`, `CategoryId` | 4 each | +68, +6c, +70; datasource columns 7, 0x23, 4; requirement text `00baec20`; client file columns corroborate |
| `MembersOnly`, `NonMiniGame` | 1 each | +74, +75; members-only is datasource column 5; second flag meaning (R) |
| `WeaponTrailEffectId` | 4 | +78; file-order semantic correspondence (R) |
| `CompositeEffectId`, `PowerRating`, `MinProfileRank` | 4 each | +7c, +80, +84; effect/power/rank, datasource columns 0x0f, 0x0e, 0x13 |
| `Rarity`, `ActivatableAbilityId`, `PassiveAbilityId` | 4 each | +88, +8c, +90; retain serializer aliases; the guide does not establish these exact semantic assignments (R) |
| `SingleUse` | 1 | +94; flag meaning from source/client file mapping (R) |
| `MaxStackSize`, `IsTintable` | 4, 1 | +98, +9c; datasource columns 0x25 and 0x16 |
| `TintAlias` | length + bytes | +58; third string, name from file order (R) |
| `ForceDisablePreview` | 1 | +9d; flag name from source/client file mapping (R) |
| `MemberDiscount` | 4 | +a0; name from file order (R) |
| `VipRankRequired` | 4 | +a4; requirement text, datasource column 0x17 |
| `RaceSetId`, `ClientEquipReqSetId` | 4 each | +a8, +ac; two requirement-set fields, `00baec20`/`00a142c0`; exact source alias assignment (R) |
| `ResellValue` | 4 | +f0; coin sell price/Worth, datasource column 0x1a |
| `Unknown36`, `Unknown37`, `Unknown38` | 4 each | +f4, +f8, +fc; unnamed native extension fields, `00919e10`; first two source aliases Param1/Param2 remain (R) |
| `ClientItemStatDefinitions` count | 4 | `00911a90`, stat list at +108; count followed by the rows below |
| stat key | 4 | `00911af8` reads the keyed-list key; `00911b08` inserts it |
| stat `Unknown`, `Unknown2` | 4 each | `008e88b0` reads record +04 and +0c; source interpretation ID/type, corroborated by stat display described in the guide |
| stat `Unknown3` | 4 bits | `008fc430` reads float at record +08 and flags NaN; decoder rejects nonfinite values |
| stat `Unknown4` | 1 | `008fc430` reads bool at record +10; semantic use unresolved (R) |
| `ItemAbilityEntries` count | 4 | `008e3c30`, following the stat list in `00919e10` |
| ability `Unknown`, `Unknown2`, `Unknown3`, `Unknown4` | 4 each | `008cc760` reads four dwords; meanings unresolved (R) |

Two Legacy rows, 18092 and 78397, encode stats as flattened `Stats.<id>.Id` / `.Value` properties. These are not silently discarded. The generator maps ID and float bits into the proved 17-byte layout; type 0 (additive) and final flag false are explicit **reconstructed defaults (R)**. Legacy supplies no ordinary `Stats` or `Abilities` collections. Empty absent lists are marked reconstructed, and missing captured stats/abilities appear as mismatch classes. Existing captured stat/ability rows retain every byte.

## Round-trip and independent-source mismatches

`decode -> existing definition_bytes`: **1,703 matches, 0 mismatches**. The batch encoder reduces the 54 equal duplicates to **1,649** IDs in sorted order. That is expected `Inventory::definitions_packet` behavior, not parity with the original ordered startup stream. It is also the stored-row fixture only; the runtime union with `item-catalogue.json` is **58,835** ids. The original 1,383-row startup stream is a different fixture, documented in [[Startup Catalogues]].

`Legacy + client starter overlay -> existing definition_bytes`: **419 exact matches, 1,284 differing row occurrences**. Every mismatch class and its count follows. A row may contribute to several classes, so counts do not sum to 1,284. Full per-occurrence field differences are in `source-mismatches.json`.

| Field / class | Occurrences | Field / class | Occurrences |
|---|---:|---|---:|
| CategoryId | 9 | Class | 170 |
| ClientEquipReqSetId | 323 | ClientItemStatDefinitions | 32 |
| CompositeEffectId | 1 | Cost | 468 |
| DescriptionId | 10 | ForceDisablePreview | 39 |
| GenderUsage | 1 | IconData.Id | 11 |
| IconData.TintId | 287 | IsTintable | 44 |
| ItemAbilityEntries | 120 | MaxStackSize | 14 |
| MembersOnly | 47 | MinProfileRank | 865 |
| ModelName | 5 | NameId | 8 |
| NoSale | 69 | NoTrade | 22 |
| PassiveAbilityId | 6 | PowerRating | 4 |
| ProfileOverride | 333 | Rarity | 6 |
| ResellValue | 1145 | SingleUse | 1 |
| Slot | 8 | TextureAlias | 4 |
| TintAlias | 2 | Type | 25 |
| Unknown36 | 1 | Unknown37 | 2 |
| Missing source ID | 4 | | |

The four baseline IDs absent from both sources are 6346231, 35463452, 163632473 and 81256176. They are retained solely from the baseline. Price, rank, profile and sell-value differences are not automatically treated as Legacy errors: native code proves their field positions, not the production value for every item. No class-wide inferred rule is applied to hide these differences.

## Content coverage and W82

Coverage reuses `content_check.py::Checker` traversal and item classification. Counts are **reference occurrences**, not unique IDs. The baseline reproduces the Ledger's exact **18,017 references / 16,551 missing**.

| Source content file | References | Missing before | Newly covered | Still missing |
|---|---:|---:|---:|---:|
| collections.json | 8 | 6 | 6 | 0 |
| dungeons/opening_cave.json | 4 | 0 | 0 | 0 |
| field_battles.json | 4 | 0 | 0 | 0 |
| harvest_nodes.json | 8 | 6 | 6 | 0 |
| pets.json | 19 | 1 | 1 | 0 |
| recipes.json | 169 | 90 | 90 | 0 |
| stores.json | 17,198 | 16,448 | 16,448 | 0 |
| vendors.json | 396 | 0 | 0 | 0 |
| vendors_unresolved.json | 211 | 0 | 0 | 0 |
| Total | 18,017 | 16,551 | 16,551 | 0 |

[[Job Starter Outfits]] appeared during this task and was cross-checked before handoff. All 77 proposed unique starter definitions are generated. Its seven missing IDs are covered: **19431, 13317, 12416, 12252, 4914, 29963, 13419**. Proposed weapons **75091 (Ninja), 29963 (Warrior), 4914 (Wizard)** are present. Ninja 75091 already existed in the baseline: its missing starter grant is a starter-list integration issue. W82's model/pack/CRC evidence is recorded in its note; this task does not claim to repeat its asset inspection or prove those selections historically correct.

## On-demand 35/34 plan (no route built)

The route already exists: `gameplay.rs` delegates 35/34 to `Inventory::handle`; `inventory.rs` accepts exactly 12 bytes (`u16 35, u16 34, i32 id, i32 zero payload length`), returns nothing for an unknown ID, and replies with `u16 35, u16 34, i32 id, i32 row byte length, row`. Native `00933501` calls wrapper `008f8180` / reader `008d7980`, which reads the header, **an ID dword and a length-prefixed blob**, then constructs one definition via `00919e10` and caches it via `00a3c290`. The older guide's shorthand omits the outer ID dword; do not copy that omission. Lead integration should supply the generated ID map to this existing lookup, preserve the current malformed-request rejection and unknown-ID no-op policy (R), and bound/rate-limit replies per session (R). Do not synthesize records or derive file paths from remote IDs. Load a separate bounded full-catalogue map; keep player-owned item limits separate. `Inventory::load` currently caps definitions at 4,096 rows and its whole manifest at 4 MiB; the generated fragment is about 68 MiB and the complete batch about 12 MiB. Do not inject it into startup unchanged. Gate: request a formerly unknown positive ID through the real gateway, verify exact response framing and unchanged inventory, then grant/display it natively and reload. No gateway change or native claim belongs to W84.

## Security and verification

Threat model: an operator runs an offline local tool against potentially malformed tables/row bytes. Inputs are read-only, bounded to 128 MiB, and hashed; no player names/chat, credentials or saves are read. Module imports use fixed repository filenames and disable bytecode writes. Output names are fixed under the assigned private report directory; resolved parent/symlink checks reject escape. Client IDs never become paths. No network listener, subprocess, dynamic evaluation or remote-player entry point was introduced. The Rust oracle is explicitly opt-in and reads private paths only.

Applied skills: Ponytail full (existing row serializer, Legacy adapter, metadata helper, content checker and Rust batch encoder reused); Caveman (concise worker communication); reverse-engineering (read-only native map and byte disassembly); secure-code-guardian (file/input boundaries); security-reviewer (automated source scan plus manual review of both code files). No new asset reader, art pipeline, installation or game automation was needed.

Security review: no open Critical/High/Medium/Low findings in the owned diff. Validation rejects truncated/trailing rows, oversized strings, negative collection counts, noncanonical flags and nonfinite stat values. Review fixes: bounded reads also recheck actual bytes read, executable validation uses errors rather than removable assertions, baseline metadata overrides are preserved, and captured/client conflicts fail closed. The manual pass confirmed fixed output filenames, private confinement, no secret literals and no added network/dev exposure. Local operator-supplied source overrides are trusted as file selections, not executed code. This does not certify a future network route or Legacy content accuracy.

Commands run from `project` (logs and exact toolchain details are in the private report directory):

```powershell
python -I scripts/build_item_catalogue.py --self-check
python -I scripts/build_item_catalogue.py
python -I scripts/content_check.py --out ../lab/reports/item-catalogue-20261015/content-check.json
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\w84'
$env:OSFR_ITEM_CATALOGUE='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\item-catalogue-20261015'
./scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','item_catalogue_roundtrip','--','--ignored','--nocapture')
```

The self-check reports roundtrip, a mutation canary and five malformed rows passing. At W84, the content checker reported **0 errors, 17,276 warnings** across 134 files. Its 16,551 `item_not_runtime` warnings were expected then: W84 did not alter runtime content. W166 later added the catalogue checks and reported **0 errors**. The bundle gate applies the merged 58,835-id set to the consumers and ride rows that a particular bundle ships; the current repository content covers **54/54 rides with 0 item-coverage errors**. The private Rust oracle passes 1 test; without the explicit ignored-test selection it remains ignored. A separate output copy with one name-ID byte flipped fails the oracle (exit 101), proving it detects a real row change; the unmodified output passes. No crate-wide formatting was run; only the owned Rust file was formatted.

Initial Cargo verification was blocked by an uncached concurrent `ed25519-dalek` dependency. A private-toolchain direct-rustc run against cached existing serde_json/sha2 libraries passed the isolated test and mutation canary. After that dependency appeared in the shared cache, the normal `scripts/cargo.ps1` retry also passed: **1 passed, 0 failed**, in the owned target directory. No dependency was added or manifest edited by W84.

Open gates: a native 35/34 request and reply for a formerly missing definition; native inventory display; starter grants; production correctness of omitted Legacy stats/abilities and all reconstructed defaults. The runtime loader/catalogue integration and bundle item gate are present (W166, `5d522cf`), so the old "only 1,649 definitions at runtime" reading is closed. The 419 independent matches must not be reported as 1,703 source-derived matches. Build Plan/Ledger and shared progress notes remain lead-owned.

Ponytail: reused the existing serializers, data adapter, content traversal and batch encoder. Deliberately omitted a second encoder, a new dependency, a general schema framework, gateway wiring and startup expansion.
