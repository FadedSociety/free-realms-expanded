# Fishing service (W34)

October 13, 2026. Scope: `prototype/osfr-tools/src/fishing_service.rs`,
`src/fishing.rs` fixes, `tests/fishing_service.rs`, and
`data/content/fishing.json`. This implements a callable service seam. It does
not register a new gateway route, modify saves, launch the native client, or
establish historical fishing balance. The lead owns gateway wiring and review.

## Evidence boundary

The controlling specification is [[Build Guides/Gameplay/Fishing]] and its
[[Build Guides/Gameplay/Minigame Framework]] lifecycle. The worker contract and
the earlier [[Research/Fishing Restoration]] / [[Research/Fishing Service
Integration]] notes are retained as history; the older fork-derived Trout,
result-kind-1, candidate school and sub-15 assumptions were discarded.

Native fact provenance points to
`lab/decompiled/client/FreeRealms/pseudocode.c`; its extracted line offsets are
indexed in [[Research/Fishing Restoration]] lines 28-51. The network tables and
Fish Finder/area tables cited below are [[Build Guides/Gameplay/Fishing]] lines
116-219; catch-card and state behavior are lines 70-105. The build guide's
recovered addresses were byte-checked with `python -I
scripts/native_map/nmq.py dis <address> <byte-count>` during this task.

### Recovered client facts

- The fishing family is opcode 138 with a two-u16 header. Client writers at
  `00b6a400`, `00b6a380`, `00b6a460`, `00b6a4d0` and `00b6b510` write register,
  cast animation, cast, reel and handle-only special requests. Sub 15 is not
  implemented as a wider fork payload.
- Sub 3 reader chain `00b6acb0` -> `00b6a170` reads 16 player-config words, a
  length-prefixed zone string, five zone values, a counted u32 array and fish
  entries. Sub 4 reader `00b6ad20` -> `00b691d0` reads a counted fish-entry
  list. `00b673b0` establishes the fish-entry widths and order. The client
  expects the area's Fish Finder entries inside sub 3; sub 4 is for a later
  list refresh.
- Sub 8 reader `00b66a00` reads handle, u32 and two vec4s. Sub 10 reader
  `00b66a80` reads handle, u32 and two flags. Sub 11 reader `00b69130` reads a
  key, two vec4s, counted model triples, a counted u32 list and two u32s. Sub
  12 reader `00b68810` reads a school key.
- Sub 14 reader `00b66be0` reads the result fields in `fishing::ResultBody`;
  kind 5 records a catch, name id is at +0x20, icon id +0x24, weight +0x38,
  size grade +0x3c, time +0x40, difficulty/rarity/score +0x44/+0x48/+0x4c.
  Catch card code at `00cd0150` maps grades 1..4 to Small through Extra Large
  and prints the weight to two decimal places. Result and normal inventory
  update are separate messages.
- Fish Finder data sources are `FishTypes` at `00cd4420`/`00cd4620`,
  `FishingLures` at `00cd4b50`, and `FishScored` at `00cd44e0`/`00cd47b0`.
  Fish names and ids in the table came from client locale
  `OSFR Client/locale/en_us_data.dat`, records of type `ugdt`.
- The six area rosters are the client locale's area descriptions. Minimum
  Fisherman levels come from the wiki table copied into the build guide, not
  the client. The client names the four size grades. Lure text states a 10%
  better chance for three fish; lure-name pairing to each trio is wiki-derived.
- Client resource `MiniGameTypeData.txt` identifies fishing as non-Flash type
  21. Generic encounter lifecycle is owned by `MinigameRuns`; fishing is
  offered as a `RunMode::Encounter`, waits for the shared start and readiness
  transitions, and cancels on minigame leave/encounter close.

The client result reader label says “two length-counted blobs”, but its bytes
read three length-prefixed strings at +0x28, +0x54 and +0x64. `nmq dis
00b66be0 512` confirmed the three reads. The code builds all three fields.
Other low-confidence labels and state interpretations were checked against
bytes with `nmq dis` before use: `00b65880 512`, `00b67240 768`, `00b673b0
512`, `00b69130 512`, `00b66a00 512`, `00b67550 768` and `00b6a2b0 768`.
These checks establish read order and widths, not server field meanings or
values.

### Reconstructed server policy (R)

Every server-selected value lives in `data/content/fishing.json` with a source
tag. This includes bite/nibble/reel timers, catch and snap probabilities,
registration values, school positions, synthetic fish type ids, selection
weights, rarity/difficulty, weight and size distributions, item icon and lure
hint placeholders. The documented 10% lure bonus is applied as a 10% relative
increase to that lure's listed fish selection weights; that probability model
is R. All icon ids and lure-hint ids are zero placeholders because no mapping
was recovered.

The service validates the cast against reconstructed min/max distance from
the server-owned anchor. It selects an eligible fish at cast time, then emits
sub 10 for nibble and hook as server timers expire. Reel before hook, or after
the reel deadline, produces sub 10 escape flags and a sub 14 kind-0 result;
neither path grants an item or quest event. A valid reel after hook settles a
kind-5 fish result from server-selected species, size, weight and score data.
No client-supplied catch outcome is accepted.

On catch, `rewards::apply_grant` stages the item grant and display packet over a
cloned `RewardState`. The service commits that returned state only after the
grant succeeds, and returns messages in this order: fishing sub 14, reward
inventory update, reward display bundle. `Change.quest_events` emits
`QuestEvent::CollectItem` for the granted item definition and quantity one;
the gateway/quest owner must pass that event to `QuestEngine::on_event`.
Fishing-to-inventory definition ids are not recovered, so the content/import
layer must call `bind_item_definition(species_key, size_grade, id)` first.
Lure possession/activation also remains the trusted caller's responsibility
before `set_lure`.

The values in `fishing.json` are an intentionally playable slice, not original
balance. The current defaults make a valid hooked reel award a fish (catch
chance 1, snap chance 0); timing, probabilities, school positions, weights,
rarity and size weights all remain R.

## Content and conflicts

`fishing.json` contains all six area rosters and 40 unique species. It retains
the client's “Golden Scaled Nettler” in Brambleback's Bayou. The wiki omits it
there and lists Bitter Betta at levels 1 and 20; the content uses the client's
roster, keeps Bitter Betta at the first listed level, and marks Golden Scaled
Nettler's level 1 as R because no wiki level was supplied. The client spells
“Blubracuda”; the wiki also uses “Blubbercuda”. The client locale's fish name
is used. “Winter Piranha” is the locale spelling; the lure text has “Winter
Pirahna”.

Several bindings remain deliberately open: type-21 rows 595..601 do not have a
verified one-to-one area mapping; `game_id` and `encounter_id` are therefore
trusted inputs to `begin`. The first three fish-entry words appear to be
string references, but field semantics and the sub-3 integer list remain
unresolved. Sub-10 byte effects are visible in client code (first true sets
+0xd9; false/true sets +0xd8; false/false sets +0xda and escape text), while
their gameplay names are not recovered. This service's nibble/hook assignment
to those flags is R. School model triples, path behavior and the last two
sub-11 words are also unresolved; school packets use empty model/path lists
and zero trailing words as R placeholders.

The `00ccfdb0` label suggests a result presenter, but the observed code creates
a `ProxiedFishingBobber`; the native code wins over that label. `00cd0150` is
labelled a controller frame update but its caller passes proxied-player
records; this service relies only on the reviewed result/catch-card field
reads. Sub 18's fish-run reader (`00b6a2b0` / `00b67550`) still has unknown
record meanings, so this service does not fabricate a fish-run packet. Without
sub 18, actual underwater fish meshes and native tension/distance/reel meters
are not validated. No decoded native fishing datagram exists in the lab.

`notes/Build Guides/Gameplay/Fishing.md` also records the wiki/client conflict
that the bait quest giver Jonah is at Sacred Grove Shallows per client strings,
not Rainbow Lake per wiki; it does not affect this service.

## Interface for lead integration

1. Construct one `FishingService` in the player's session context. At the
   trusted portal/area entry call `begin(guid, game_id, encounter_id, area_key,
   fisherman_level, position, reward_state)` and send its 39/16 offer.
2. Route the shared minigame lifecycle to `handle_minigame_lifecycle`. After
   the encounter has completed startup, call `mark_started`; only then does
   fishing registration and opcode-138 activity proceed.
3. Resolve and bind actual inventory definition ids for all available fish
   grades. Route opcode 138 payloads to `handle(inner, guid, monotonic_ms)` and
   schedule `tick(guid, monotonic_ms)`. Send returned replies in order.
4. Feed returned `quest_events` to the session's `QuestEngine`. Persist the
   service's returned reward state together with the character save before
   sending grant packets; `apply_grant` already returns staged snapshots, but
   this module does not save them.
5. Route cancel/leave through the shared minigame lifecycle (or `leave`), and
   call `disconnect` on session loss. `disconnect` returns sub-12 school
   despawns for the caller to broadcast to nearby clients, then drops the
   fishing/minigame state with no reward.

The root already exposes `fishing_service`; this scoped change did not edit
`lib.rs`, the gateway, saves or launcher. The player flag that activates the
native fishing processor and the exact area/game/encounter selection still
need lead-owned integration.

## Verification record

- JSON parse check: `python -I -c "import json,pathlib; p=pathlib.Path('prototype/osfr-tools/data/content/fishing.json'); d=json.loads(p.read_text()); print('fishing.json parse ok; areas=',len(d['areas']),'species=',len(d['species']),'lures=',len(d['lures']))"` -> parsed, 6 areas, 40 species and 13 lure entries.
- Formatting: private toolchain `rustfmt --edition 2024` on only
  `fishing_service.rs`, `fishing.rs` and `tests/fishing_service.rs` -> exit 0.
- Requested command:
  `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W34'; scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test fishing_service --test fishing`.
- One intermediate run compiled the service and passed `tests/fishing.rs` (12
  tests); five of six then-current service tests passed. The one failure was
  the test helper reading reward family 50's one-byte subtype as a u16; the
  helper was corrected, and a lifecycle-cancel test was added afterward.
- Final retries did not reach test execution. Concurrent edits in shared files
  stopped compilation first at `src/bin/osfr-gateway.rs:437` (parameter
  `_options` referenced as `options`); current retries stop in shared
  `src/soccer_match.rs` (pattern/initializer still use `unknown_word` and
  `vectors` after the packet/StateRequest fields changed to `state_counter`,
  `action_direction`, `steering_vector` and `position_vector`) and
  `src/soccer_service.rs:150` (`request.vectors` no longer exists). These files
  are outside this task's ownership and were left untouched. The final test
  result is therefore **blocked by shared compile errors**, not a passing W34
  suite. The latest compiler output had no `fishing_service.rs` diagnostics.
- No native client launch or fishing capture was performed. Native visuals,
  packet parity, area activation, item-definition bindings and original balance
  remain unverified.

## Skills and workflow

`reverse-engineering` informed reader-by-reader schema work and the `nmq dis`
checks; Ponytail kept the implementation within the existing modules and APIs;
Caveman was used for brief progress updates. RTK was read and used for
condensed command output. Game-recon and web research were not needed because
the reviewed Build Guide and preserved client evidence already answered the
scoped questions. Asset-pipeline and game automation were not applicable: no
asset was generated and the native client was not launched.
