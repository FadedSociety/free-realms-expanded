# Pets service

Updated 2026-10-13 for Build Ledger item 24, pet half. This note separates shipped-client evidence, data-only reference material, and reconstructed server policy (R). No original pet traffic was available, and no OSFR pet records were used for this slice.

## Evidence and provenance

| Fact | Provenance | Use and confidence |
|---|---|---|
| Pet packet family is `0x35`; the client has pet handlers, request writers, and state readers. | `notes/Build Guides/Gameplay/Pets.md`, Network and Key functions; native map shards S0346, S0348, S0349. | Client fact. The game-state handler and message calls are present in the shipped x86 client. No pet datagram was decoded, so server replies and wire framing remain hypotheses. |
| Summon sender `00b60080` writes a pet id and action byte `1`; recall `00b601a0` writes pet id and action byte `0`. | `nmq dis 00b60080 128`, `nmq dis 00b601a0 128`; constructors/wrapper `00b50890`/`00b5c5c0`. | Client request bytes. `PetActive` subtype 9 as a server response is likely in the guide, not traffic-verified. |
| The PetActive reader consumes a u32 pet id and an eight-byte actor guid after the pet header. | `nmq show 00b556c0`; accepted likely label, pseudocode inspected. | Reader shape. The module's little-endian subtype-9 reply remains (R), since no server datagram exists. |
| PetStatus reader consumes one u32 and four floats. | `nmq dis 00b555a0 160`. Although the label is speculative, bytes show the dword followed by four guarded float reads. | Reader shape verified from bytes. Field order and the server's use of subtype 3 remain (R). |
| Pet record reader at `00912cf0` reads the fixed dword/byte fields, four floats at record offsets `+0x20..+0x2c`, counted lists/arrays, string blobs, four favourite dwords, and eight utility dwords. | `nmq show 00912cf0` plus `nmq dis 00912cf0 192`; S0240. | The function label is speculative. The bytes and pseudocode support the reader layout; meanings/order for unresolved fields are not inferred into Rust. |
| Feed sender `00b5f720` checks handler in-water byte `+0x1300` and owner flag bit `0x20` at `+0x143` before building the utility-feed request. | `nmq dis 00b5f720 160`; accepted-unknown label checked against bytes. Groom sender `00b5f8c0` is confirmed and has the same visible gates. | Client-side refusal gates. PetService mirrors the water gate for care; the integration adapter supplies minigame-busy state as (R). |
| Favourite sender accepts UI slots 1 through 4 and sends zero-based slots. | `nmq brief 00b5f670`; confirmed label and caller path. | Client fact; Rust stores four zero-based favourite entries and returns a reader-shaped echo as (R). |
| `00ccecf0`, `00ccf3c0`, and `00ccf260` are fishing controller functions, not pet grooming. | `nmq brief` and `nmq dis` for all three addresses; S0417. Grooming spots use `PetTactile` code around `00cce850` and `00cce960`. | The fishing functions were not used as pet evidence. |
| Client pet-name data contains 60 first-part rows and 250 second-part rows; the first first-part row is blank. | `lab/runs/client-baseline-20261010T064034Z/client/Resources/PetNames.txt`, SHA-256 `fe40e4146b15e1c2378b97a32ced822ea6f5b0059d83d8d84ab0199e9046f477`; loader `00b5aa40`. | Shipped client table. `pets.json` retains keys and order; rename validates zero-based selections against those arrays. Custom vanity names and paid rename are not implemented. |
| Gesture points define 23 trick ids across 147 waypoint rows. | `lab/runs/client-baseline-20261010T064034Z/client/Resources/PetTrickGesturePoints.txt`, SHA-256 `ac8efd2c263d11b5769880ec416e1c70b6484553f1927ec28cc63f349f85bde4`; loader `00cc9bb0`. | Shipped client table. The guide's 147-row count is correct. The generated table records each trick id and waypoint count. `PetTrickGestureSettings.txt` SHA-256: `a45f9bad0dbbf2bc59c011c50165195ebdb4a75d65344d6f62f475f3f750f822`. |
| The client exposes food, groom, play, mood, 4 favourite slots, 5 wearable/toy slots, and 8 utility slots. | `notes/Build Guides/Gameplay/Pets.md`, Data and formats and the client data-source/record tables; pet record reader above. | Client fields/slots. The Rust need scale and field order are explicitly reconstructed where the guide leaves them unresolved. |
| Simple Pet Food, Simple Pet Shampoo, and Basic Pet Conditioner map to item ids 1835, 1836, and 10521 in the reference item catalogue. | Data only from `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`, commit `9e082025391177d5a33ee18c714ddd06b457a695`, SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`. | Reference seed only; no Legacy code copied. Its item NameId values (426492, 426490, 24599) do not match the client locale ids listed in `notes/Build Guides/Content/Jobs/Pet Trainer.md` lines 244-248. Do not treat these item ids as client-confirmed. |
| The shipped client contains no species/catalogue table, item-id care map, or need-decay table in the four `Resources/Pet*.txt` files. | `notes/Build Guides/Gameplay/Pets.md`, Data and formats; inspected `PetNames.txt`, `PetEffects.txt`, `PetTrickGesturePoints.txt`, and `PetTrickGestureSettings.txt`. | The wiki-derived `species` array is now populated. Client species ids and exact per-entry model/item bindings remain unresolved; runtime definitions are withheld until those values are known. |

## Reconstructed policy in this slice

All values and rules in this section are (R):

- Need range is 0–100; new pets start full. Idle decay is `0.01` per second, summoned decay `0.02` per second. Mood is the mean of food/groom/play. Care restores 25 for the seeded food/shampoo entries and 35 for the seeded conditioner entry. Play with an equipped toy restores 20 play points.
- Pet ids increase per roster. Vendor grants deduplicate by grant id. Adopting an owned pet item consumes exactly one inventory unit through `Inventory::consume`.
- The roster is saved; active actor guid, follow target, and in-water state are session-only. Reload starts with no active pet because the old actor guid cannot be reused safely.
- A summon is rejected when the caller says a minigame is active. The lead adapter must derive this flag from the active `MinigameRuns` state. Care also refuses an active pet marked in water.
- The pet actor starts at the owner position. The server returns PetActive subtype 9; dismiss uses id 0/guid 0 as the unresolved none sentinel. Both are (R).
- Follow accepts any finite target for now and replies `can_go=true`; the native distance/time thresholds are unresolved. This provisional behavior needs a reachability rule before native acceptance.
- Care requires the item instance in an equipped utility slot, consumes one unit, then returns a PetStatus-shaped update. PetUtilityNotify begin/end ordering, grooming duration, item-tier effects, vitamin effects, and treasure rules remain open.
- `learn_trick` validates ids present in the gesture table. The caller must apply Pet Trainer job/level unlock rules before invoking it. Favourite slots must be 1–4 and refer to a learned trick.
- Inventory instance ids are stored in the five wearable/toy fields and eight utility fields. This mapping is (R), pending comparison with native pet record/item ids.

## Artifacts and verification

- `prototype/osfr-tools/src/pets.rs`: standalone service module, intentionally not registered in `lib.rs`. Message builders cite client readers in code comments. Bytes are inner pet-family messages, before gateway tunnelling.
- `prototype/osfr-tools/data/content/pets.json`: source-tagged client name and gesture tables, reference-seeded care item mappings, reconstructed effect/decay settings, and wiki-derived species/NPC-pet rows with unresolved client bindings.
- `prototype/osfr-tools/tests/pets.rs`: `#[path]` module tests for item adoption, valid/invalid rename, summon/dismiss bytes and presence events, follow/equipment, one-item care, decay, trick/favourite, reload, and minigame rejection.
- `prototype/osfr-tools/src/game_store.rs`: schema 6 adds a typed-by-caller optional `pets` payload while preserving the concurrent housing payload. Saves with no pet payload remain schema 5; writing pet state bumps that envelope to 6.
- `prototype/osfr-tools/tests/save_schema.rs`: appended schema-5-to-6 migration test checks pet reload, housing preservation, and instance counters.

Commands and results (from `prototype/osfr-tools`):

```powershell
& '..\..\..\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' --edition 2021 src\pets.rs tests\pets.rs
$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W26'
& '..\..\scripts\cargo.ps1' test --test pets --test save_schema
```

Final result: pets `7 passed, 0 failed`; save schema `5 passed, 0 failed`. Cargo reported one unrelated warning in the housing module: `Lot::instance_index` is unused. No native client was launched.

The acceptance baseline first failed as expected because `src/pets.rs` did not exist. During implementation, an initial schema assertion expected version 5; the pet payload correctly writes version 6. The final focused run above passes.

## Lead integration steps and open gates

1. Register `pets` and connect PetService to the player session after the local player object exists. Route the minigame and in-water state into its busy/care checks.
2. Resolve species id, model, adoption item, per-species trick set, and current-client item ids from reviewed client evidence. Replace the three Legacy reference mappings if they do not match the active item catalogue.
3. Route owner messages through the gateway wrapper. Translate `PetPresenceEvent` spawn/despawn/follow events into the actor replication path used by other players; the pet protocol guide does not establish those actor packets.
4. Replace the provisional follow `can_go=true` rule with a validated reachability check. Confirm dismiss sentinel, PetStatus field order, care utility notification sequence, name-request layout, and actor guid lifecycle from client evidence or captures.
5. Run the native two-player summon/follow/dismiss, care, and save/reload gates. These tests establish service behavior only; no client visual, actor motion, grooming interaction, or original-server parity is claimed.

The broader gesture drawing round, Pet Trainer XP/rank rewards, custom vanity naming, vitamins/treasure, store purchase, trial pets, and native pet visuals/movement are not implemented by W26.

## Skills used

Reverse-engineering was used to inspect the reviewed native map and verify speculative/unknown labels against bytes. Ponytail guided reuse of the existing inventory transaction and canonical save envelope. Caveman guided concise communication; this saved evidence note stays explicit. Game-recon and game-research-websearch were unnecessary because the Build Guide already identifies the client and the task used the local reviewed corpus. Asset-pipeline was unnecessary because no game asset was transformed.

## Species catalogue

`python -I lab/content/tools/cq.py wiki-cat Pets` returned 115 titles: the `Pets` hub and 114 pet pages. `pets.json` now has 113 species rows and one separate named NPC pet, Harley (owned by Wheelie Pete). The species rows cover 31 normalized kinds; dog breeds and Toyger are retained in `breed` while `species_kind` uses dog/cat.

Primary wiki source mix for the 113 rows: 84 SC Marketplace, 16 Cragara, 7 direct Trading Card Game rewards, 2 Daily Wheel, 1 membership promotion (Pinkeye), 1 quest/event reward (St. Bernard), 1 Blacksmith recipe (Mini-Mech), and 1 Clockwork Monkey item kit. Dot has an additional membership-gift path, so the JSON contains 114 acquisition-source records for 113 species. Gray Toyger's article describes its resemblance to Chester, the TCG pet, but lists Gray Toyger itself as a Marketplace pet; it is counted under Marketplace. Wiki `SC` template prices and member prices populate `price` and `member_price` when present. The values retain `[wiki]` provenance.

Client checks used `cq.py loc` for each species name, the 1,383 startup item-definition records (1,329 unique ids), `data/content/stores.json` (17,198 Legacy-derived reference rows), `catalog.tsv`, and Legacy `Resources/Npcs.json` as a data-only model cross-check. Nine species names have one exact locale id; 97 have multiple exact locale ids, and 7 have none. Ambiguous ids are kept as candidates, not selected as species bindings. Exact species-name locale ids did not match startup item-definition name ids; checked kit, recipe, and loot-card labels also had no item-definition match. All 114 acquisition records therefore have a null `store_item_id` with an explicit unresolved reason; no coin-store price is attributed to a pet without an item binding.

All 113 `species_id`, runtime `model`, and `model_files` values remain null with reasons. The client asset catalog contains pet-family `.adr` files, but neither it nor the reference NPC mapping binds a wiki pet to a species id/model record. `member_only` is false for 91, true for Pinkeye, and unknown for 21. `field_sources` tags every species field; adoption records also tag each field. The focused content test checks those tags and requires every adoption item id to exist in the client item catalogue or carry an unresolved reason. No player-chosen pet names were recorded.

R: `PetContent::definition` exposes a catalogue row as a runtime definition only when both `species_id` and `model` are populated. This keeps unresolved wiki rows from becoming spawnable content and preserves behavior for resolved definitions; it does not infer client bindings.

The W46 research pass used the local archived wiki and client-corpus workflow from game-research-websearch; a live web fetch was unnecessary. Game-recon, reverse-engineering, and asset-pipeline work was unnecessary because this pass changed no client runtime, binary behavior, or game asset.


## 2026-10-13 - W46c species/model/item bindings

This pass used the evidence W46b had not inspected. Before: 0/113 species IDs, 0/113 models, 0/113 model closures, 0 adoption-item IDs, and no icon or texture-alias fields. After: 113/113 reconstructed IDs, 26/113 models and closures, 16 active item IDs across 16 species, and 16 client icon references. Wiki-derived kind remains 113/113; 3 explicit breeds remain wiki-sourced. `adoption_source.store_item_id` remains null for all 114 acquisition records because marketplace/store product IDs were not bound.

### Category and item evidence

The client Build Guide observes the inventory top-level label `Pets`, but the shipped `Resources` folder has no `ItemCategories.txt` or `ItemCategoryGroups.txt`, and the W46c startup snapshot contains item definitions rather than category-tree packets. The data-only `FreeRealms-Legacy` reference `ItemCategoryGroups.txt` maps group ID 2 to category IDs 6, 7, and 200. Active client item definitions corroborate those leaves by type: 22 Type 3 pet-wearable records use category 6, 9 Type 4 pet-supply records use category 7, and 10 Type 2 pet records use category 200. Therefore this pass calls **group 2 the Pets group by reference-tree join** and **category 200 the pet-species leaf**; the numeric group label was not captured from the original server. The remaining 18 Type 2 records have `CategoryId=0` and are listed separately; do not silently treat category 0 as group 2.

The table gives every active startup item definition in the reference group-2 subtree (IDs 6, 7, 200), with its active-client model string, texture alias, and icon data. The label column is the Legacy `Comment` field for search only. It is reference data, not verified client display text; for example item 8 has reference comment `Earth Digger Helmet`, while its active client record is Type 4/category 7 with `bowl_basic.adr`.

| CategoryId | Type | Item id | Reference label | Model ADR/template | Texture alias | Icon ImageSetId/TintId |
|---:|---:|---:|---|---|---|---|
| 6 | 3 | 2203 | Kitty Island Lei | `pet_cat_<descriptor>_shoulders_lei.adr` | `hawaiian-flowers-P` | 2971/237 |
| 6 | 3 | 2206 | Kitty Horizon Sweater | `pet_cat_<descriptor>_chest_sweater.adr` | `casual-stripeshorizon-L1` | 2895/249 |
| 6 | 3 | 3996 | Doggy Paper Hat | `pet_dog_<descriptor>_head_newspaperhat.adr` | `freestyle-newspaperhat-P` | 4509/263 |
| 6 | 3 | 5081 | Doggy Hiking Backpack | `pet_dog_<descriptor>_chest_backpack.adr` | `explorer-backpack-P` | 2987/256 |
| 6 | 3 | 5104 | Doggy Island Shirt | `pet_dog_<descriptor>_chest_collarshirt.adr` | `hawaiian-flowers-P` | 2989/263 |
| 6 | 3 | 5333 | Doggy Fairy Wings | `pet_dog_<descriptor>_chest_wings.adr` | `fairy-stripes-M` | 3021/257 |
| 6 | 3 | 5613 | Doggy Fairy Shoes | `pet_dog_<descriptor>_feet_pointywcuffs.adr` | `fairy-tassles-M` | 3038/257 |
| 6 | 3 | 5753 | Doggy Fairy Antennae | `pet_dog_<descriptor>_head_antennas.adr` | `fairy-stars-M` | 3051/257 |
| 6 | 3 | 5906 | Doggy Ninja Cowl | `pet_dog_<descriptor>_head_hood.adr` | `ninja-plate-P` | 3064/249 |
| 6 | 3 | 5961 | Doggy Pirate Hat | `pet_dog_<descriptor>_head_pirate.adr` | `pirate-skull-P` | 3069/228 |
| 6 | 3 | 6138 | Doggy Fairy Collar | `pet_dog_<descriptor>_shoulders_collarwcard.adr` | `fairy-butterfly-M` | 3079/257 |
| 6 | 3 | 8364 | Doggy Racer Jacket | `pet_dog_<descriptor>_chest_racejacket.adr` | `racecardriver-zipper-P` | 2997/228 |
| 6 | 3 | 8539 | Doggy Racer Snaps | `pet_dog_<descriptor>_feet_fingerlessgloveshightop.adr` | `racecardriver-snaps-P` | 3672/228 |
| 6 | 3 | 8644 | Doggy Racer Goggles | `pet_dog_<descriptor>_head_racehelmet.adr` | `racecardriver-blackgoggles-P` | 3070/228 |
| 6 | 3 | 8868 | Doggy Pirate Collar | `pet_dog_<descriptor>_shoulders_necklacecoin.adr` | `pirate-gold-P` | 3088/263 |
| 6 | 3 | 8994 | Doggy Racer Scarf | `pet_dog_<descriptor>_shoulders_wrapscarf.adr` | `racecardriver-knitted-P` | 3098/228 |
| 6 | 3 | 34168 | Kitty Frog Skin | `pet_cat_<descriptor>_chest_animalshirt.adr` | `frog-spots-M` | 2871/256 |
| 6 | 3 | 34588 | Kitty Frog Legs | `pet_cat_<descriptor>_feet_animalboots.adr` | `frog-spots-M` | 2910/256 |
| 6 | 3 | 34763 | Kitty Frog Collar | `pet_cat_<descriptor>_shoulders_lei.adr` | `frog-flies-M` | 2970/256 |
| 6 | 3 | 77411 | Doggy Superhero Cape | `pet_dog_<descriptor>_chest_shirtcape.adr` | `superhero-star-M` | 2998/264 |
| 6 | 3 | 77412 | Doggy Superhero Boots | `pet_dog_<descriptor>_feet_bracersbootspointy.adr` | `superhero-rubber-M` | 3029/264 |
| 6 | 3 | 77413 | Doggy Superhero Mask | `pet_dog_<descriptor>_head_maskpointy.adr` | `superhero-pointy-M` | 3068/264 |
| 7 | 4 | 8 | Earth Digger Helmet | `bowl_basic.adr` | `bowl-bones` | 1764/228 |
| 7 | 4 | 25 | Green Paws Bowl | `bowl_basic.adr` | `bowl-paws` | 1767/257 |
| 7 | 4 | 28 | Purple Paws Bowl | `bowl_basic.adr` | `bowl-stars` | 1771/247 |
| 7 | 4 | 1835 | Simple Pet Food | `(empty)` | `(empty)` | 1766/232 |
| 7 | 4 | 1836 | Simple Pet Shampoo | `(empty)` | `(empty)` | 1768/244 |
| 7 | 4 | 37014 | Slicker Brush | `(empty)` | `(empty)` | 1756/231 |
| 7 | 4 | 37016 | Massage Brush | `(empty)` | `(empty)` | 1756/256 |
| 7 | 4 | 56778 | Heavy Duty Scrub Brush | `(empty)` | `(empty)` | 1756/253 |
| 7 | 4 | 62536 | Confetti Shampoo | `(empty)` | `(empty)` | 1760/228 |
| 200 | 2 | 2272 | Chicken | `chicken.adr` | `(empty)` | 6082/0 |
| 200 | 2 | 7890 | Snake | `snake.adr` | `(empty)` | 6088/0 |
| 200 | 2 | 10322 | Soccer Ball | `soccerball_m.adr` | `plain` | 6622/0 |
| 200 | 2 | 10323 | Soccer Ball | `soccerball_m.adr` | `reversecolors` | 6623/0 |
| 200 | 2 | 10407 | Fairy Boy | `sprite_basic_m.adr` | `(empty)` | 6860/0 |
| 200 | 2 | 10469 | Jack-O-Lantern | `jackolantern_m_01.adr` | `(empty)` | 6939/0 |
| 200 | 2 | 10497 | Marshmallow | `marshmallow_critter.adr` | `(empty)` | 6434/117 |
| 200 | 2 | 14184 | Thumper | `rabbit_thumper.adr` | `(empty)` | 7039/0 |
| 200 | 2 | 17557 | Snow Hog | `snowhog.adr` | `(empty)` | 6983/0 |
| 200 | 2 | 77460 | Cakeling | `cakeling_m_basic.adr` | `(empty)` | 6411/0 |

The following 18 active Type 2/category 0 item definitions are outside that reference group tree. Sixteen have a reference Comment that exactly matches a W46 wiki species display name and are stored as `adoption_item_ids`; item IDs remain client-verified, while the name-to-species join is tagged `R` and `reference` in `pets.json`. `Black Lab Whistle` and `Wolf` have no exact species-title join.

| CategoryId | Type | Item id | Reference label | Model ADR/template | Texture alias | Icon ImageSetId/TintId |
|---:|---:|---:|---|---|---|---|
| 0 | 2 | 2644 | Black Lab Whistle | `dog_large_lab.adr` | `lab` | 3198/191 |
| 0 | 2 | 4324 | Fritz | `spider_m.adr` | `furry` | 5944/343 |
| 0 | 2 | 9049 | Treeble | `treeble.adr` | `treeble_orange` | 6479/542 |
| 0 | 2 | 10392 | Ski | `dog_large_husky.adr` | `darkmark` | 4457/213 |
| 0 | 2 | 10481 | Phoenix | `dragon_m_phoenix.adr` | `0` | 6955/0 |
| 0 | 2 | 10992 | Chuck | `groundhog_m.adr` | `groundhog-basic` | 6254/455 |
| 0 | 2 | 10998 | Dot | `dog_large_dalmation.adr` | `dalmation` | 3206/172 |
| 0 | 2 | 11939 | Clockwork Beetle | `beetle_clockwork.adr` | `(empty)` | 7675/429 |
| 0 | 2 | 15493 | Wolf | `wolf_generic.adr` | `base` | 6873/539 |
| 0 | 2 | 17849 | Landshark | `groundhog_m_sharkfin.adr` | `groundhog-sharkfin` | 6620/260 |
| 0 | 2 | 23930 | York | `dog_small_fluffy.adr` | `lightmark` | 3211/202 |
| 0 | 2 | 23964 | Peccadillo | `dog_large_floppy.adr` | `spots` | 3199/179 |
| 0 | 2 | 55840 | Shadow | `dog_large_pointyghost.adr` | `darkmark` | 3804/213 |
| 0 | 2 | 70352 | Char | `dragon_m_basic.adr` | `goldtips` | 5606/282 |
| 0 | 2 | 70447 | Zax | `tyrannosaurus_m.adr` | `trex-dino` | 5857/387 |
| 0 | 2 | 76529 | Yule | `reindeer_m.adr` | `basic` | 5614/336 |
| 0 | 2 | 76906 | Lava Tiger | `tiger_m_lava.adr` | `lava-tiger` | 7058/0 |
| 0 | 2 | 78070 | Fire Pegasus | `pony_pegasus_m_fire.adr` | `fire` | 7080/290 |

**Item provenance.** `startup.json` SHA-256 `da3a4ac1654522e212e7364fcf3870f5315e055a45b66f7ecdb116c904c0f276` (1,911,797 bytes) contains 1,383 item-definition records. Parsing the client serializer yields Type, CategoryId, model, texture alias, `Icon.ImageSetId` and `TintId`. `catalog.tsv` SHA-256 `12bfd0f81236ffc02e6a9a661e1ffb14890a2bd4ac2306c14b20e9cbe445b1c9` (11,218,192 bytes) holds 108,547 extracted rows. The readable name comments came only from data in `FreeRealms-Legacy` commit `9e082025391177d5a33ee18c714ddd06b457a695`, `ClientItemDefinitions.json` SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`; no source code was copied. Category group 2-to-6/7/200 mapping is reference-only.

### Species joins and ADRs

- **16 item-backed joins:** York/23930, Zax/70447, Char/70352, Shadow/55840, Peccadillo/23964, Chuck/10992, Fritz/4324, Treeble/9049, Ski/10392, Dot/10998, Yule/76529, Phoenix/10481, Lava Tiger/76906, Landshark/17849, Fire Pegasus/78070, and Clockwork Beetle/11939. For each, the active Type 2 record supplies the model ADR, texture alias, and icon image-set/tint pair. Those 16 item records currently have CategoryId 0, not category 200.
- **10 reconstructed asset-only joins:** Coyote -> `wolf_coyote.adr`; Crackers -> `duckbill_m.adr`; Clockwork Monkey -> `monkey_clockwork_m.adr`; Duke -> `dog_large_dukey.adr`; Hydra Pet -> `hydra_basic.adr`; Mini-Mech -> `robgoblin_minimech.adr`; Pinkeye -> `bat_m_pinkeye.adr`; ROFLE -> `rofle_m.adr`; St. Bernard -> `dog_large_stbernard.adr`; Vulture -> `vulture_pet.adr`. Each row tags the actor-to-wiki association `R`; the name/kind match is a content reconstruction, not an original species table. Model-only rows have no adoption item, texture alias, or icon binding.
- **ADR closure:** each of the 26 selected ADRs was read with the pinned FR ADR Toolkit revision `5acfdf8d6006fa6f43d1d3605f076a6be7393e72`. `model_files` records the ADR, skeleton `.dsk`, mesh `.dme`, and material `.dma` names parsed from that actor; every filename exists in `catalog.tsv`. All 444 pet-named ADRs in the supplied inventory parsed without error. The active Type 2 item model strings are checked against the catalog as filenames (the 26 species model tests fail closed if a selected ADR disappears).
- **Icons:** bound item records preserve the client `ImageSetId` and `TintId` pair in `icon`. The catalog contains 336 `icon_pets_*.dds` rows, mainly named trick, care, and thought-bubble art. The available data does not map those DDS filenames to the item ImageSetId values; asset-only species icons stay null with a reason instead of taking an unrelated action icon.
- **Species IDs:** all 113 IDs use reconstructed namespace `1_000_001` through `1_000_113`, allocated by case-insensitive `wiki_title` rank. This is an R namespace for this content set; the client Pet record dword at `+0x08` remains unidentified. `PetContent::definition` now supplies the first bound adoption-item id when present and still withholds rows that have no model.
- **Locale IDs:** 0/97 multi-candidate names were disambiguated. None of the active Type 2 `NameId` values equals a retained candidate ID, and parsed ADR model/skeleton/mesh/material fields contain no locale ID. The 97 candidate arrays remain unchanged; 9 single-candidate names also remain unselected because no item/model reference binds their locale ID; 7 titles have no exact locale candidate. `client_name_string_id` remains null for all 113.

### W46c verification

Commands from `project/prototype/osfr-tools`:

```powershell
& '..\..\..\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' --edition 2021 src\pets.rs tests\pets.rs
$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W46c'
& '..\..\scripts\cargo.ps1' -CargoArgs @('test', '--test', 'pets', '--', '--nocapture')
```

The formatter exited 0. The Rust test target did not compile, so none of its nine tests ran after the final data update. Across retries, compilation stopped in concurrently edited, unowned `src/login.rs` with missing `gateway_binding_ticket`, moved `flags`, missing `migrate_legacy_account_store`, and missing `CharacterState` errors. The latest attempt reported missing `CharacterState` at line 480 and moved `flags` at line 242. W46c did not edit that file. A pre-data probe failed because `adoption_item_ids` had not yet been populated and is not final validation.

A separate read-only catalog validation passed. It parsed the `name` column (column 3) from `catalog.tsv`, checked every bound ADR and all four-file closure references against the inventory, checked every adoption item id against startup item definitions, and checked species-id uniqueness. Command, from repository root:

```powershell
@'
import json,pathlib
root=pathlib.Path(r'%USERPROFILE%\Documents\Projects\CartoonMMO')
project=root/'project'; lab=root/'lab'
rows=json.loads((project/'prototype/osfr-tools/data/content/pets.json').read_text())['species']
startup=json.loads((lab/'content/startup-snapshot/startup.json').read_text())
names={line.split('\t')[2].lower() for line in (lab/'extracted/client-assets/catalog.tsv').read_text().splitlines()[1:] if len(line.split('\t'))>2}
item_ids={int(x['id']) for x in startup['catalogues']['item_definitions']}
ids=[x['species_id'] for x in rows]
models=[x['model'] for x in rows if x.get('model')]
files=[f for x in rows if x.get('model') for f in (x.get('model_files') or [])]
adopted=[i for x in rows for i in x.get('adoption_item_ids',[])]
assert len(set(ids))==len(rows) and all(x.lower() in names for x in models+files) and all(i in item_ids for i in adopted)
print(f'PASS species={len(rows)} unique_ids={len(set(ids))} models={len(models)} model_file_refs={len(files)} adoption_item_ids={len(adopted)} invalid_models=0 invalid_files=0 invalid_items=0')
'@ | python -I -
```

Result: `PASS species=113 unique_ids=113 models=26 model_file_refs=104 adoption_item_ids=16 invalid_models=0 invalid_files=0 invalid_items=0`. Coverage is 113/113 kinds, 3/113 explicit breeds, 16/113 icons, 15 nonempty texture aliases, and one explicit empty alias. The Rust test parser was corrected to use the catalog `name` column rather than its pack-name column.



**Open gaps.** No original Pet packet or server species table; R IDs are not historical IDs. Active item NameIds still do not resolve pet display names; 97 locale candidate sets remain ambiguous. Marketplace/store acquisition records do not link to inventory Type 2 item IDs. Category group/category labels rely on Legacy reference rows rather than captured original category packets. Ten model-only associations are R. Eighty-seven wiki rows still lack one model; 97 have no adoption item ID or icon; 97 texture aliases are null and one is explicitly empty. Native pet spawn/appearance and inventory browsing remain untested by this worker.
