# Housing placement selection identity

Read-only identity research for the `127/2` first dword, 2026-10-09 (UTC). Reviewer owns this note
and `lab/reports/housing-placement-selection-identity-20261009T2230Z/` only. The frozen
`housing-native-codec` candidate and all production paths are untouched. Skills: RTK (normal
commands), Ponytail (one stdlib checker), Caveman (terse report), reverse-engineering (the
pseudocode anchors).

**Verdict: the `127/2` first dword is the client's placement *selection id*, not the placed item.
It is bound across three message types and separated from the authoritative ids the server sends:
`127/1` request id -> `127/2` first dword, echoed by `127/42` next to the model / fixture
definition / item definition ids; the paired `127/40` carries those authoritative ids, and the
selection id is neither the item definition id (nor any scalar sub-field of it) nor the
server-minted fixture guid. Which table/build owns the selection id is not identified; the pinned
community table's gear names for it are an id-space overlap, not a build-mismatch proof.**

## Binding evidence (96/96 checks, `verify-identity.log`)

* 12/12: every `127/1` first `u32` equals the second `u32` of the next `127/42` (echo), and that
  `127/42` also carries the model id, fixture definition id and item definition id.
* 8/8 placements: the `127/2` first dword equals one of the `127/1` selection ids, and the paired
  `127/40` carries exactly the `127/42` fixture definition id and item definition id.
* 8/8: the item definition id resolves in the pinned community `ClientItemDefinitions.json` to a
  housing model (`hsg_*`); the selection id resolves to a non-housing entry and is not any scalar
  sub-field of that item entry.
* 8/8: the `127/40` fixture guid is server-minted (`0x07f54ca17bf5c0xx`) and differs from the
  selection id.

| 127/2 frame | selection id (community name) | fixture guid | item definition | fixture definition | model id |
| --- | --- | --- | --- | --- | --- |
| 31494 | 4260 (Elite Black Shortbow) | `0x07f54ca17bf5c001` | 7466 Blue Square Block (`hsg_block_01.adr`) | 34 | 1926 |
| 31914 | 4260 (Elite Black Shortbow) | `0x07f54ca17bf5c002` | 7466 Blue Square Block | 34 | 1926 |
| 32016 | 4378 (Baseball Bat) | `0x07f54ca17bf5c003` | 14300 Camp Robbie Banner (`hsg_banner_robby_01.adr`) | 902 | 2159 |
| 32194 | 4179 (Battle Axe) | `0x07f54ca17bf5c005` | 77259 Blue Flat Rectangle Block (`hsg_block_flatrectangle_01.adr`) | 490 | 1988 |
| 32275 | 4214 (Warlord Axe) | `0x07f54ca17bf5c006` | 7462 Blue Rectangle Block (`hsg_block_02.adr`) | 323 | 1945 |
| 32335 | 4594 (Mega Hammer) | `0x07f54ca17bf5c007` | 16039 Chatty Totem (`hsg_totemsegment_chatty.adr`) | 68 | 3002 |
| 32574 | 3996 (Doggy Paper Hat) | `0x07f54ca17bf5c008` | 18110 Coffee Cups (`hsg_coffeecup_grp_01.adr`) | 1074 | 3058 |
| 32619 | 4595 (Mega Hammer) | `0x07f54ca17bf5c009` | 16040 Chugawug Totem (`hsg_totemsegment_chugawug.adr`) | 69 | 3003 |

## Native anchors

* Selection callbacks `FUN_00ab0d60` (pseudocode 1393520-1393568) and `FUN_00ab2250`
  (1394650-1394694) look the selection up with `FUN_0105bfa0(param_2,param_3)`, copy the placement
  vectors from `FUN_00cb93f0()`/`FUN_00cb9420()` into the processor `+0x4150..`/`+0x4160..`, and
  mark the mode (`+0x4140` = 1/2).
* Placement handler `FUN_00ac42d0` (1407911-1407944, mode 3) copies the looked-up object's
  `+0x40..0x4c` into `+0x4160..0x416c`; the wire builder then emits `+0x4160` as the first payload
  dword (1409713-1424). The `127/1` first `u32` comes from the same id, so request and commit agree.
* `FUN_0105bfa0` (2461550-2461582) is a hash-map lookup keyed by the two selection words
  (`piVar2[8] == param_2 && piVar2[9] == param_3`), not an item-definition accessor.

## Build-aware catalogue check

The pinned `Sanctuary/src/Resources/ClientItemDefinitions.json` resolves **all eight authoritative
item ids** to housing models (`hsg_block_01.adr`, `hsg_banner_robby_01.adr`,
`hsg_block_flatrectangle_01.adr`, `hsg_block_02.adr`, `hsg_totemsegment_chatty.adr`,
`hsg_coffeecup_grp_01.adr`, `hsg_totemsegment_chugawug.adr`), so the table is build-compatible with
the captured housing items. The selection ids resolve in the same table to unrelated gear and to no
sub-field of the item entry; they also appear in `Npcs.json`, `CoinStoreItems.json` and
`StoreBundles.json`, i.e. the numeric range overlaps several id spaces. There is therefore **no
evidence** that the selection id is a different build's item definition; its owning catalogue is
unidentified, so it stays a qualified unknown. The echo in `127/42` is not used as an authority
signal by itself — the authority link is the `127/40` fixture definition / item definition pair.

## Limits

Read-only capture and pinned-resource analysis; no UI, live input, gateway, content or save writes;
no native acceptance. The checker re-derives everything from `p2.pcap` (`65AC0801…`) through the
pinned piece-022 transport decoder. The `127/40` at frame 32098 has no paired `127/2` request, so it
is excluded from the placement table.

## Artifacts

`lab/reports/housing-placement-selection-identity-20261009T2230Z/` — `verify-identity.py`,
`verify-identity.log` (96/96 ok), `identity-report.json` (previews, placements, item rows, table
survey), `final-hashes.txt`.

## Correction receipt (independent review `4C553572…`, 2026-10-09)

The independent review `notes/Reviews/Housing Placement Selection Identity Review.md`
(`4C553572B2CE2EC51CC726B2373144924711F245B92484C36A640EC057A2C305`) reproduced the 96 data
checks and raised three documentation qualifications, applied here. The pre-correction note is
preserved byte-identically as
`lab/reports/housing-placement-selection-identity-corrections-20261009T1745Z/note-before-correction.md`
(`8D842519…`); the original report directory is untouched.

1. **Ordering.** Only **7 of 8** placements have a matching `127/1` before them. Placement frame
   **31494** (selection 4260) matches `127/1` frame **31629**, which is *after* it; that row
   supports selection membership/co-occurrence only, not a preceding-request pairing. The `127/2`
   selection ids are members of the `127/1` id set 8/8 (seven preceding, one following).
2. **`127/40` pair wording.** The fixture-definition/item-definition pair in `127/40` is a
   **client-observed co-occurrence** with the earlier `127/42`; the capture does not show the
   server authorizing ownership from it. Read "the ids the server sent alongside the fixture
   record" instead of "authority link"; ownership authority remains unproven pending server-side
   evidence.
3. **Namespace overlap is per id, not blanket.** 3996/4260 appear in `Npcs.json` +
   `CoinStoreItems.json`, 4378 in `Npcs.json` + `StoreBundles.json`, 4179/4214 in `Npcs.json`
   only, and **4594/4595 in none** of those three (only `ClientItemDefinitions.json`). The
   conclusion (owning catalogue unidentified; overlap is not a build-mismatch proof) stands; the
   blanket sentence is narrowed here.

Recheck: the checker re-run in the receipt directory reports **96/96 ok** (`recheck-96.log`) and
regenerates `identity-report.json` byte-identically (`2D4236E7…`), so the data fields are
unaffected. Structured receipt:
`lab/reports/housing-placement-selection-identity-corrections-20261009T1745Z/correction-receipt.json`.
