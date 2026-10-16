# 081 Inventory instance lifecycle

Status: closed for review, 2026-10-09 05:30 UTC. This piece recovers the **instance id lifecycle**
around the existing item rows and reports the one unrecoverable case. It is a typed module, a
checker and evidence; it is not a playable claim. Stack limits, merge, atomic add/remove and the
38/2, 38/3 and 38/4 replies already exist in `src/inventory.rs` and are exercised here, not
reimplemented.

Owned paths: `prototype/osfr-tools/src/restoration/piece_081.rs`,
`prototype/osfr-tools/tests/piece_081.rs`, `scripts/pieces/piece_081.py`,
`tests/pieces/test_piece_081.py`, this note and `lab/reports/piece-081-20261009T051824Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched,
and no commit, release or native input was made. Hashes for every source, input and log are in the
private receipt `lab/reports/piece-081-20261009T051824Z/receipt.json`.

## The gap

Production allocates a new instance inside `Inventory::grant_with_tint` as
`max(existing item id) + 1` (`src/inventory.rs`), and `Inventory::consume` deletes a row when its
count reaches zero. Retiring the highest row therefore frees its id: the next grant hands the same
id out again. A restart repeats the same derivation because nothing durable records the id.

Two characterisation tests pin both halves: `production_reward_reuses_a_retired_top_instance_id`
shows the current allocator reusing id 3 after it was consumed, and
`counter_supplied_ids_close_the_production_reuse_gap` shows the counter agreeing with production on
the first grant and diverging exactly at the reuse. Both live in the owned test file and are the two
tests the lead should retire when `inventory.rs` takes its id from the counter.

## Recovered original facts

Item row. The prototype gateway and the client carry the same row: definition, tint, instance id,
count, consumed count, last cast time, a rental flag and an ability count
(`Gateway.Player.PlayerCode.SendPlayerUpdateItemDefinitions` in
`lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs`; the pinned community mirror is
`lab/references/Sanctuary/src/Sanctuary.Packet.Common/ItemInstance.cs`, whose `Rental` field is still
a `TODO`). The current campaign's live rows on disk are exactly that shape
(`lab/state/opening-mmo-continuation-20261009T023433Z/character.json`, 34 rows).

Stack limits. `BaseItemDefinition.MaxStackSize` is an original definition field
(`lab/decompiled/server/Gateway/Gateway/BaseItemDefinition.cs` line 59) and is written into the 35/37
catalogue. The prepared original catalogue holds 1383 definition rows with `max_stack` in
{-1, 1, 10, 50, 100, 1000}; 1194 definitions declare -1, which production already reads as
"unbounded" (`definition.max_stack <= 0 || count <= max_stack`).

A declared limit is a merge policy, not a stored invariant. The preserved original character holds
one live row above its definition's positive maximum: id 3650, definition 9119, count 10, declared
max stack 1 (`lab/references/rust-gameplay/inventory.json`). The checker therefore reports that row
rather than rejecting it, and the module never rewrites stored rows.

Instance id space. The same preserved self holds **1386 live instance ids inside 2..=4595 with 3209
gaps** (lowest id 2, highest 4595, no duplicates). The current campaign branch already shows the
same shape at small scale: 34 live ids in 1..=35 with id 22 retired. A compacting allocator would
not leave those holes, so the captured data is evidence for ids that are handed out once and never
refilled. It is evidence, not proof: the original server's allocation routine is not in hand.

Community implementations reuse ids. Both pinned checkouts allocate a new row from the live
maximum, so the pattern this piece replaces is real code, not a hypothetical:

| Site | Pattern |
|---|---|
| `FreeRealms-Legacy`/`Sanctuary` `src/Sanctuary.Gateway/Handlers/BaseCoinStorePacket/CoinStoreSellToClientRequestPacketHandler.cs:103,:127` | `NextId = x.Items.Max(i => i.Id)` then `Id = dbQuery.NextId + 1` |
| `FreeRealms-Legacy` `src/Sanctuary.Game/Trading/TradeTransferPlanner.cs:282,:330` | `nextItemId = party.Items.Max(item => item.ItemId)` then `itemId = ++nextItemId` |
| `CommandPacketInteractRequestHandler.cs:107,:117` | `DefaultIfEmpty(0).Max() + 1` |
| `PacketInGamePurchasePlaceOrderPacketHandler.cs:118,:149` | `lastItemId = … Max(i => i.Id)` then `Id = ++lastItemId` |

All flagged sites are listed by the checker and in
`lab/reports/piece-081-20261009T051824Z/campaign-and-community-report.json`; AGPL-3.0 provenance is
unchanged and nothing was copied from them.

## Reconstructed

`InstanceCounter` owns one durable document holding one integer, `next_instance_id`, with
`schema_version` 1 and `deny_unknown_fields` (`instance-counter.json`, bound 4 KiB).

* `open(path, live_instance_ids)` reads the document, floors it to
  `max(stored, highest live id + 1, 1)` and writes a repaired floor **before** returning; a missing
  document is a fresh character at 1.
* `allocate()` hands out the current value and durably records the following one **before** the id
  leaves the module, so a grant that never lands burns its id instead of freeing it. A failed write
  leaves both the in-memory value and the file unchanged.
* `verify(live_instance_ids)` rejects non-positive ids, duplicate ids and a counter that does not
  lead every live row.
* the id is `i32` and bounded: after `2147483647` is handed out the next call returns `Exhausted`
  without writing.

Everything in that list is local policy. No original allocation rule, no numeric ceiling and no
ordering rule was recovered.

## Integration contract for the lead

1. Open the counter once per character where gameplay state loads, passing every id the build can
   observe: the live rows plus the prepared content's own `initial_state` ids. The current content
   starts at 7, the live campaign at 35, so the counter opens at 36.
2. In `Inventory::grant_with_tint`, replace the `max(id) + 1` derivation with `counter.allocate()`.
   Keep the existing staged-clone atomicity, the merge, the `fits` stack check and the 38/2/38/3
   replies unchanged.
3. Never lower the counter in `consume`, `equip` or on load. A failed grant leaves a permanent gap,
   which is the shape the captured id space already has.
4. Persist the counter with the same write-before-use ordering, either as the sibling document this
   module already owns or folded into the version 2 gameplay snapshot by its owner.
5. Retire the two characterisation tests above when the call site changes.

## Verification

| Step | Command | Result |
|---|---|---|
| RED | `scripts/cargo.ps1 test --test piece_081` against a recompute-only revision | 8 failed, 4 passed, 1 ignored (`red-recompute-only.log`) |
| GREEN | `scripts/cargo.ps1 test --test piece_081` | 12 passed, 0 failed, 2 ignored (`green-durable-counter.log`) |
| Private evidence | same target with `--ignored` and `OSFR_PIECE_081_CATALOG`/`OSFR_PIECE_081_SAVE` | 2 passed: 1386 ids in 2..=4595 with 3209 gaps; live campaign hands out 36 and records 37 (`green-private-evidence.log`) |
| Checker | `python -m unittest discover -s tests/pieces -p "test_piece_081.py" -v` | 8 passed (`python-checker-tests.log`) |
| Ordinary suite | `scripts/cargo.ps1 test` | 136 test binaries, 1338 passed, 0 failed, 57 ignored (`full-suite.log`) |
| Format | `rustfmt --edition 2024 --check` on the two owned Rust files | clean after formatting those files only |

The checker was also run over the preserved original catalogue and over the current campaign pair
plus both pinned community trees (reports `original-catalogue-report.json`,
`campaign-and-community-report.json`).

## Boundaries and gaps

* No original server allocation routine exists in the supplied material, so "never reuse an id" is
  a local invariant justified by the sparse captured id space. Any later recovery of the original
  rule must be checked against it rather than assumed.
* If the counter document is lost, ids retired above every observed id are unrecoverable and can be
  handed out again. The floor is `max(stored, observed + 1)`; this module does not fabricate a rule
  to close a case it cannot see.
* The original client has not accepted anything this module produces. Native acceptance needs a
  grant after a retired top id, a restart and a fresh grant, with the actual client showing the new
  item instance.
* The two production characterisation tests deliberately fail if production's allocator changes
  before the lead retires them.
