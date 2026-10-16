# Collections, achievements, titles and claims (W49)

Updated 2026-10-13. Scoped module and content; no gateway, save, or native-client integration.

## W49b determination

Collection, achievement, and title definitions are server-sent; assign replacement IDs locally as reconstructed policy (R). The client ships category/trophy shells, not these wiki-derived definitions. Collections: family 38/subtype 8 is dispatched by `009392c0`; reader `009099d0` reads the keyed start record, including the reward bundle and keyed rows. Achievements: family 150 dispatcher `00badec0` switches across subtypes 2–8; bytes at `00badf53` call the header reader, and `00badf69` bounds the subtype to 2–8 before its jump table. Subtype-2 add constructor is `00bab710`; subtype-4 initialize constructor/list reader are `00bad510`/`00bade40`. Titles: receive handler `00b4a3a0` handles 152/1 add and 152/3 update-all; add reader `00b4a280` reads four dwords. These handlers require the definition records from the server. `CollectionCategories.txt`, `AchievementCategories.txt`, and `AchievementTrophies.txt` remain client-defined resource rows; `DiplomacyTitles.txt` and `DiplomacyRanks.txt` are header-only. Native map and packet details: [[Build Guides/Gameplay/Rewards and Collections]].

## Source boundary

The requested note did not exist at this path when W49 started. This note uses the reviewed gameplay guide and the existing [Piece 074 collections note](Pieces/074%20Collections%20and%20collectibles.md). The project wiki corpus is read-only. Wiki text is transformed into content data with CC BY-SA attribution and pinned revisions.

Evidence precedence remains client, then reviewed video, then wiki. The wiki fills lists only where the client does not ship world collection or title definitions. Unrecovered original IDs and non-unique locale matches stay `null`; replacement IDs and counter bindings are marked (R).

## Recovered client facts

### Reward bundles

- Client reader `008e7930` consumes the shared reward bundle; entry construction is `0103c070`. The existing Rust grant boundary is [`rewards.rs`](../../prototype/osfr-tools/src/rewards.rs): grant messages mutate wallet, profile XP, and inventory; family 50/subtype 1 is display-only.
- The supported client entry IDs are `1, 3, 6, 7, 8, 10–15, 17–20, 22, 23`. Never serialize entry IDs `2, 4, 5, 9, 16, 21`: the client creates no reader for them and consumes no body. `progression_extras::validate_reward_rows` rejects them, in addition to the existing `rewards::apply_grant` validation.
- Client provenance: `Resources` reward reader and constructors in `notes/Build Guides/Gameplay/Rewards and Collections.md`, “The reward bundle” and “Entry types”; entry factory `0103c070`; bundle reader `008e7930`.

### World collections

- First-party `CollectionCategories.txt` has ten rows: IDs 1–10, locale IDs and icon IDs. Its pinned SHA-256 is `c3a06887eccb1f6f0e0fd64c773e9979a82ef4fcad6579cd53691dd71e4fbd37` (182 bytes, CRC32 `88c0f5c2`). The exact rows and resolved locale names are in Piece 074. Client category loader: `009fd4e0`.
- Client Lua `ScriptsBase.lua` defines `BaseClient.CollectionCategory` as three columns and `BaseClient.CollectionData` as 17 columns: IDs/name/description/image, entry count, tint, category, and eight collected flags. Client pseudocode `FUN_009fce20` returns three category columns; `FUN_009fce30` names them; `FUN_009fd050` hashes category IDs modulo ten. Piece 074 records these facts and the lab source hashes.
- World progress packets are client-update family 38. Subtype 8 starts a collection using a length-prefixed record containing its reward bundle and keyed rows; subtype 10 adds an entry; subtype 11 removes an entry. The subtype-10 record is 33 bytes including its two-`i16` header. Subtype 9's body was not recovered. Guide provenance: `009392c0`, `009099d0`, `008fb360`, `008d8570`, and `008d8670` in “Collections” and “Network”.
- Reward bundle entry types 10 (`CollectionAdd`) and 11 (`CollectionEntryAdd`) can represent collection changes. Which path the original server used for each collection is unknown. Harvest, quest, and bundle-entry sources in the new module are event origins, not recovered trigger mappings.
- `startup_selfdata.rs` parses a 7-dword prefix, reward bundle and keyed rows while preserving unnamed bytes. `tests/startup_selfdata.rs:317` checks 253 records in its self-data fixture. Those records are not decoded into collection definitions here.

### Achievements and trophies

- Client resource `AchievementCategories.txt` has 23 rows; `AchievementTrophies.txt` has ten rows. Pinned SHA-256 values are `c1a38aba479dc5d30796e19586f856d6a2a4b3819398b2ca6ef2b76a83885cb8` and `947dd769f799ce3fd5d8bced6625290d878b9dc0e7e78efb93489681f71abd02` respectively. Trophy IDs 1–10, locale IDs, image sets, point values and content IDs are copied as client data; achievement-to-trophy assignments are not.
- Family 150 (`0x96`) has subtypes 2–8. Native dispatcher `00badec0` calls `00baaeb0`; disassembly at `00baaeb0` reads an `i16` followed by a 32-bit subtype. Constructor `00ba97d0` stores family `0x96` and the supplied subtype. Constructor `00ba9b70` stores subtype 7's three dwords and an `f32`; `00ba98c0` stores subtype 3's two dwords and byte. The new encoder uses those widths. Progress ratio, field meanings, event bindings and award timing are reconstructed policy (R).
- The achievement records and objectives come from the server; no achievement IDs or gameplay bindings are shipped in these client resource tables. The client's notification handler is `00a95940`; dispatch is `00badec0`. Guide provenance: “Achievements and trophies” and “Data and formats”.

### Titles

- Family 152 (`0x98`) uses a `u16` plus `u32` header. Client receive handler `00b4a3a0` handles add (1), remove (2), and update-all (3). Add `00b4a280` reads four dwords. Client requests select (4) with one title ID via `00b4a880`; refresh request (5) has no body via `00b4a910`.
- Lua exposes title ID at data-source column 0 and selected at column 3. The mapping of that selected flag into the four packet dwords and the update-all count width were not recovered; the new title writer marks both as (R).
- The title picker in `AboutPlayer:Show` is filled only under `Ui.IsChinaEnvironment()`. Shipped `DiplomacyTitles.txt` and `DiplomacyRanks.txt` have header rows and no data. This is why the global client's title picker may be hidden. Guide provenance: “Titles” and “Data and formats”.

### Claim and promotional packets

- Claim family 108 uses `u16` opcode and `u16` subtype. Client sends 108/1 to list; server answers 108/2. Claim request 108/3 contains a tracker ID and counted 8-byte claim IDs; server answers 108/4 with the tracker ID and success flag. The client then requests definitions with 108/5 and accepts 108/6 only for the matching tracker. Client entry reader `00a040e0` is labelled likely; claim row middle fields are not fully decoded.
- The new module accepts claim IDs only after trusted server code queues them with `issue_server_grant`. Grant application uses the shared reward transaction and a stable batch key. Claimed IDs are exposed for save integration. Claim record bytes remain opaque; the tests use synthetic bytes, not client captures.
- Opcode 114 subtype 1/2 is promotional/store-originated traffic. The module only frames an already serialized body; it does not implement code redemption. Provenance: `00a08e60`, `00a08d70`, `00a08b10`; guide “The Claim window” and “Network”.

## Wiki-derived content

Queries used the local corpus through `python -I ..\lab\content\tools\cq.py wiki ...`:

| Page | Revision | Use |
| --- | ---: | --- |
| `List of Collections` | 27852, 2013-07-13 | 240 set-list names in eight categories |
| `Exploration (Collections)` | 46070, 2022-03-21 | Detailed exploration sets and entries |
| `Job Specific (Collections)` | 27853, 2013-07-13 | Detailed job sets and entries |
| `Toys (Collections)` | 31048, 2013-08-21 | Detailed toys sets and entries |
| `Achievements` | 48170, 2023-11-06 | 318 achievement table rows |
| `Titles` | 47673, 2023-07-24 | 36 title names and acquisition notes |

URLs, revision hashes and source tags are present in each content file. Collection page hashes and the one detailed set absent from the list (`Ninja Dolls`) come from `lab/reports/piece-074-20261009T041951Z/collections.json`. The category-6 label differs between client locale (“Robgoblin Junk”) and wiki (“Various Robgoblin Junk” / “Robgoblin Junkpile”); content keeps both labels.

- At W49 baseline, `data/content/collections.json` had ten client categories, 241 wiki rows and 100 detailed sets; original collection/entry/image IDs, triggers and rewards were null where unrecovered. Wiki rows in Quests and Minigames are absent from the list corpus.
- At W49 baseline, `data/content/achievements.json` had 23 client categories, ten client trophy rows and 318 wiki rows; achievement IDs, category joins, event keys, trophy joins and server rewards were null.
- At W49 baseline, `data/content/titles.json` had 36 wiki names; original title IDs and packet metadata were null. Diplomacy resources were empty and the China-only picker condition was recorded.

## Implementation and checks

`prototype/osfr-tools/src/progression_extras.rs` reuses the existing Piece 074 collection ledger and `rewards::apply_grant`. Collection progress stages before grant and commits only after the reward succeeds. Achievement completion and claims also grant through `rewards`; display bundles do not grant state. A repeated event, completed achievement or claimed ID cannot award again within the supplied state.

Known packet encoders cover collection 38/8–11, achievement 150/3 and 150/7, title 152/1–5, claim 108/1–6, and opaque promotional 114/1–2. Collection subtype 9 and claim item rows remain caller-supplied opaque bytes. The collection ledger and achievement/title states have state serialization; claim history is exportable as claimed IDs but is not connected to an account save.

| Command | Result |
| --- | --- |
| `python -I ..\lab\content\tools\cq.py wiki 'List of Collections'` | Page read from local wiki corpus; 240 names recorded in generated content. |
| `python -I ..\lab\content\tools\cq.py wiki 'Achievements'` | Page read; 318 table rows normalized. |
| `python -I ..\lab\content\tools\cq.py wiki 'Titles'` | Page read; 36 names normalized. |
| `python -I scripts/native_map/nmq.py dis 00baaeb0 80` | Bytes show signed 16-bit read, cursor advance by two, then a dword read. |
| `python -I scripts/native_map/nmq.py dis 00ba97d0 96` | Constructor stores family `0x96` and subtype. |
| `python -I scripts/native_map/nmq.py dis 00ba9b70 96` | Subtype-7 constructor stores three dwords and an `f32`. |
| `python -I scripts/native_map/nmq.py dis 00ba98c0 96` | Subtype-3 constructor stores two dwords and a byte. |
| Private rustfmt via `rustup run stable-x86_64-pc-windows-gnu rustfmt --edition 2024` on the two W49 Rust files | Passed; only owned Rust files formatted. |
| `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W49'; ..\..\scripts\cargo.ps1 test --test progression_extras` from `prototype/osfr-tools` | Passed: 6 passed, 0 failed. One unrelated `housing.rs` dead-code warning was emitted by the shared crate. |

W49 tests used synthetic collection/achievement/title IDs and a synthetic claim row. They verified once-only collection reward application, achievement threshold award and packet, title ownership/selection, tracker-matched claim and definitions response, and rejection of bundle IDs 2, 4, 5, 9, 16 and 21.

## W49b follow-up — ids, locale strings and bindings

### IDs and locale coverage

| Catalog | W49 original IDs | W49b local (R) IDs | W49b `name_id` | W49b `description_id` |
| --- | ---: | ---: | ---: | ---: |
| Collections | 0/241 | 241/241 | 238/241 (2 ambiguous, 1 unmatched) | 0/241; no wiki descriptions |
| Achievements | 0/318 | 318/318 | 187/318 (129 ambiguous, 2 unmatched) | 226/318 (91 ambiguous, 1 unmatched) |
| Titles | 0/36 | 36/36 | 22/36 (14 ambiguous) | 0/36; no wiki descriptions |

The 595 assigned IDs are globally unique. The replacement namespaces are collections `0x40000001–0x400000F1`, achievements `0x41000001–0x4100013E`, and titles `0x42000001–0x42000024`. Persist each key-to-ID mapping; allocate from the next unused number, never renumber or reuse, and retain removed IDs as tombstones. These are server-compatible local keys, not recovered original IDs.

`lab/reports/locale-id-to-text.json` has 85,932 entries and SHA-256 `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`. Its `id` field is the integer referenced by resource `NAME_ID` values: `93636 → Exploration`, `4988 → Bronze`, and `510213 → Wonderous`. Exact text was matched first; then NFKC, case-folding, punctuation-to-space and whitespace normalization. Only unique matches were assigned. Ambiguous and unmatched definition rows, candidate locale IDs and text are preserved in each content file's `locale_match_report.unresolved`.

Achievement category resource rows 6 and 8 have native `NAME_ID=0`; all 23 category rows have `DESCRIPTION_ID=0`. Those zeros are absent from the locale table, so JSON stores them as null and preserves each raw value as `source_name_id` or `source_description_id`.

The first-party `AchievementTrophies.txt` rows already provide all 10 `image_set_id` values. `reward.client_trophy_id` now joins all 318 wiki reward rows by unique point value (5, 10, 15, 25, 40) or by the exact Wonderous label plus client locale name for 0 points. This is a wiki-to-client-resource crosswalk, not a recovered per-achievement packet assignment. Per-achievement `image_id`/`image_set_id` remain null because the server-sent `IconId` values were not captured. Collection `image_id` stays null because no client collection-definition table or captured row supplied it. Title images stay null because both diplomacy tables have headers only.

### Reconstructed counter bindings (R)

`counter_key` is filled for 85 individual job-rank achievements (`job_rank:<job_slug>`) and five quest-count achievements (`quest_completed_count`). Job events count promotions from the starting rank 1, so the threshold is target rank minus one. Quest thresholds are 10, 50, 100, 200 and 400 completed quests. Rust evidence is `profiles.rs::experience_packets` (rank update family 38/18), `restoration/piece_069.rs` (completed-quest-count update), and `AchievementProgression::on_event`. These are reconstructed keys (R), not native event names. The other 228 achievements stay unbound: Combat 112, TCG 40, Quests 25, Items 12, Minigames 9, Housing 7, Snow Days 7, Spooktacular 7, Events 4, Jobs 4 and Festival of Hearts 1. Their keys, names and wiki sections are listed in `achievements.json` → `counter_binding_report.unbound_rows`.

### W49b checks

| Command | Result |
| --- | --- |
| `python -I scripts/native_map/nmq.py dis 009099d0 120` | Bytes read the collection key and record fields, then call the reward-bundle and keyed-row readers. |
| `python -I scripts/native_map/nmq.py dis 00badf34 512` | Achievement bytes call `00baaeb0`, read the subtype, subtract 2, bounds-check through 8, then dispatch via a seven-entry table. This verifies the incomplete `00badec0` map summary's subtype range. |
| `python -I scripts/native_map/nmq.py dis 00b4a280 180` | Title add reader consumes the shared header then four dwords. |
| Private rustfmt on `src/progression_extras.rs` and `tests/progression_extras.rs` | Passed; the global `cargo fmt` command was not used. |
| `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W49b'; ..\..\scripts\cargo.ps1 test --test progression_extras` from `prototype/osfr-tools` | Passed: 6 passed, 0 failed. One unrelated `housing.rs` dead-code warning remains. |

The expanded source-content check verifies all 595 IDs are present and unique, each non-null name/description locale ID resolves to non-empty client text, all 318 trophy joins resolve, and the expected locale and counter-binding coverage remains stable.

## Open gates

- No original server implementation or capture of these packet families exists. The client readers do not establish server ordering or the reconstructed counter, claim, or title policies.
- No original collection/entry IDs, node placements, trigger radii, collection rewards, achievement IDs, or original event bindings were recovered. Wiki rows remain catalogue leads, not executable client definitions. The 253 opaque self-data rows are not mapped onto these wiki names; W49b's definition keys and selected counters are local (R).
- Original title IDs remain unverified. W49b used local (R) keys; W57 adds Legacy Id candidates without packet confirmation. The global title picker is hidden outside its China-only UI branch.
- The exact claim-row writer and definitions source remain unimplemented. Claim once-per-account behavior requires the caller to persist `claimed_ids` with the reward state. The tracker map is session state.
- The new module is not registered in `lib.rs` and is not wired to harvest, quest, reward-bundle, account-save or gateway handlers. No native client was launched; no native or full-game acceptance is claimed.

## W57 follow-up — Legacy title and collection rows

### Evidence classes and input provenance

This pass added only data rows from FreeRealms-Legacy revision `9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0); no Legacy source code was copied. Input SHA-256 values:

| Input | SHA-256 | Use |
| --- | --- | --- |
| `src/Resources/PlayerTitles.json` | `0235992f5fb18f3e7df169bc4d5e020cc08825ab9f615d2af7b31d0a087d6e65` | 90 title rows, Id/Type/NameId/IconId |
| `src/Resources/Collections.json` | `d2c511fbb95702dacb517d1b146aaa94b73d009b911957a40ca9cddcecc1fbcc` | One collection row and its eight entries |
| Client `Images/Images.txt` | `0f79b8016c035b1d155d3ee71c63c61c7f3fcdb5aa29a04ac7131be3e2ef3443` | Title and collection image-ID cross-check |
| Client `CollectionCategories.txt` | `c3a06887eccb1f6f0e0fd64c773e9979a82ef4fcad6579cd53691dd71e4fbd37` | Category 10 cross-check |
| `lab/reports/locale-id-to-text.json` | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` | Numeric locale-ID to text cross-check |
| `ClientItemDefinitions.json` | `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025` | Collection entry item-ID cross-check |

The client title reader at `00b4a280` consumes four dwords after the shared family-152 header (W49b disassembly evidence above); this identifies the record shape, not the meanings of each dword or any Legacy ID. A byte search over 14 checked-in PCAP/PCAPNG files found no family `0x98` subtype 1–7 signature in decoded SOE reliable payloads or reassembled reliable fragments. The scan did not unpack SOE MULTI (`0x03`) envelopes. Therefore no title ID in this import is packet-verified; Legacy Ids are candidates only, and server use remains (R).

| Capture under `lab/references/OpenSourceFreeRealms/ofrserver/Packets` | SHA-256 |
| --- | --- |
| `p1.pcap` | `f5fa8a40a552291ef81aaee909ade4c52df9c62a19144b0b40accce02a72ef60` |
| `p12.pcap` | `99309eb5d3287bb458bf3e79332efcfc7ded3ba742077bb7258ab34a8addfe97` |
| `p2.pcap` | `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108` |
| `p3.pcap` | `ddb557ed8d132955e17790036a70cc348e0b467605cdc91ed2142550ff48475a` |
| `packets_1.pcapng` | `817d9a565c1ff6508b5938ce81e04a7949e47d467dd41e30cc124d10ab47380c` |
| `packets_1_just_gameserver.pcapng` | `4387d0b55cfce81aff83095a99d83ce4f2afc73419c52c65f39e9cb0c2a25102` |
| `packets_2.pcapng` | `221b8d548d0c01bea5f84bb0eadece91e7902ec0c81ffddd989cdea6e0775f37` |
| `packets_3.pcapng` | `c56341ca756984602b369bbad3a8a410e627e94a1346c8852aa46b17e7fff41a` |
| `packets_4.pcapng` | `5734593a76f08a79013378d2f5c71c40356e6b54d41d2ad245d7f7a8af8013d5` |
| `packets_demoderby_minigame.pcapng` | `9306cec9fae98ee6752af7db2d8f774421903aaed8aa592ebdbccb03572a6fae` |
| `packets_racing_minigame.pcapng` | `4a289592f8105f7df6857cf030071bb9a0d797bd192cd8d5f68e2e7efc9b5986` |
| `packets_soccer_minigame.pcapng` | `9e609bb526fbf7aa00164afc694ddd4d27680e55477492127012c8ff493ee8` |
| `possible_character_data.pcap` | `4ebc3e70c5a09a9122fd85b7d692d6d61695d5b848565be60e3cc84d8e34781f` |
| `possible_character_data.pcapng` | `dda23a7f43326fdf0c71c3dec8a73cd282829b89b7b032598d1daa79ed6ad4f0` |

### W57 coverage and remaps

| Catalogue | W49b baseline | W57 content | Cross-check result |
| --- | --- | --- | --- |
| Titles | 36 wiki rows; 22 wiki name IDs; local (R) IDs | 90 Legacy rows; all 36 wiki rows reconciled and 54 added; all 90 NameIds resolve | 53 raw IconIds present: 52 occur in client Images.txt, 37 rows have no IconId, and raw IconId 3141 is unresolved. Runtime `image_id` is null for that unresolved value. |
| Collections | 241 wiki rows; 238 name IDs; 1 description ID; 240 local (R) IDs | 241 definitions; eight Legacy entries merged into `wiki:3:69` | NameId 17054 and DescriptionId 17055 resolve; entry NameIds 17056–17063 resolve; ItemDefinitionIds 11081–11088 exist in the pinned item definitions. |

The 36 matched title rows now carry Legacy Id candidates and their previous local IDs are recorded in `titles.json` → `legacy_id_remaps`. The other 54 Legacy titles were added with their Legacy Id candidate. Candidate status is retained in each row's `source_tags`; `coverage.legacy_id_candidates_packet_verified` remains 0. The prior W49b locale match report is retained as `w49_locale_match_report`.

The merged collection is Briarwood Mushrooms: candidate Id 17054; client-verified CategoryId 10 (`Minigames`); existing wiki category 3 retained separately; locale-resolved DescriptionId 17055; eight entries (Legacy entry IDs 41–48). The previous local definition ID 1073741893 → candidate 17054 mapping is in `collections.json` → `legacy_id_remaps`. The other 240 definition IDs remain local (R). Legacy IconId 2124 is not in client Images.txt, so the raw value is preserved as `legacy_icon_id` while runtime `image_id` remains null. The client category row is `10^392298^1319^`; locale ID 392298 resolves to “Minigames”.

### W57 check

| Command | Result |
| --- | --- |
| `python -I -` with an inline standard-library preflight over the three JSON files, locale report, Images.txt, Models.txt and ClientItemDefinitions.json | Passed: titles=90, title NameIds=90, verified title images=52; collection definitions=241, Legacy entries=8; node types=2, pools=2, spawns=76; models=2 and loot item IDs=8 exist. Unresolved image references: title 3141 and collection 2124. |
| `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W57'; ..\..\scripts\cargo.ps1 test --quiet --test progression_extras --test progression_extras_content --test harvest_nodes` from `prototype/osfr-tools` | Passed after updating the old W49 title/count assertions: 6 progression tests, 1 content cross-reference test, and 1 harvest-node test; 0 failures. |

The exact merged rows and per-field evidence tags are in `prototype/osfr-tools/data/content/titles.json` and `collections.json`. The focused test also verifies unique candidate IDs, all present locale references, valid runtime image IDs, unique collection IDs, and collection item references. The test reads the local lab source tables directly; it does not claim a server handshake or native UI acceptance.
