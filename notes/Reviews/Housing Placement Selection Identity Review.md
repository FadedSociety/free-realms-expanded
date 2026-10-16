# Housing Placement Selection Identity Review

Independent read-only review of `notes/Research/Housing Placement Selection Identity.md`
(`8d842519d5f6244c65806afa4de7ac83f81949bdba7f4a084f0d4d1550f0c5e1`) and its
report `lab/reports/housing-placement-selection-identity-20261009T2230Z/` (96
checks). No source, gateway, UI, content or save was written; owned artifacts
are this note and `lab/reports/housing-selection-identity-review-20261009T174032Z/`.

Method: the pinned piece-022 housing decoder was reused to re-derive every
`127/x` row from `p2.pcap` (`65ac0801…`), then the reviewer's own correlation
rules, community lookups and pseudocode anchors were applied (no author checker
code was reused).

## Verified independently

| Claim | Result |
| --- | --- |
| counts | 12 x `127/1`, 8 x `127/2`, 9 x `127/42`, 9 x `127/40` |
| `127/1` selection echo | 12/12: first u32 equals the second u32 of the next `127/42` |
| `127/2` selection | 8/8 are members of the `127/1` request-id set |
| item ids | 8/8 resolve in the pinned community table to `hsg_*` housing models |
| fixture guids | all `127/40` records carry `0x07f54ca17bf5c0xx`, distinct from the selection id |
| native anchor | `FUN_00ac42d0` calls `FUN_0105bfa0`, sets mode `+0x4140 = 3`, copies `+0x40..0x4c` to `+0x4160..0x416c` and `+0x50..0x5c` to `+0x4150..` |
| keyed lookup | `FUN_0105bfa0` walks a hash bucket keyed by `piVar2[8]`/`param_2` and `piVar2[9]`/`param_3`, not an item-definition accessor |
| selection vs item | 12/12 checked: the selection id is not any scalar sub-field of the paired item entry |

## Qualifications the note needs

1. **Ordering.** Only 7 of the 8 placements have a matching `127/1` before them.
   The first placement, frame **31494** (selection 4260), has its matching
   request at frame **31629**, i.e. *after* the placement. That row supports
   membership/co-occurrence only; the note must not describe it as a paired
   preceding request.
2. **"Authority item pair" wording.** The capture shows the `127/40` record
   co-carries the fixture definition and item definition ids observed in the
   earlier `127/42`. That is a client-observed pairing, not evidence that the
   server authorizes ownership from that pair; the sentence "the authority link
   is the `127/40` fixture definition / item definition pair" needs a qualifier
   (e.g. "the ids the server sent alongside the fixture record"), pending
   server-side evidence.
3. **Namespace overlap is per id, not blanket.** The selection ids resolve to
   gear entries (Elite Black Shortbow, Baseball Bat, ...) and are not
   item-entry sub-fields, but the extra-file overlap varies: 3996/4260 appear
   in `Npcs.json` + `CoinStoreItems.json`, 4378 in `Npcs.json` +
   `StoreBundles.json`, 4179/4214 in `Npcs.json` only, and 4594/4595 in none of
   the three. The conclusion (owning catalogue unidentified; overlap is not a
   build-mismatch proof) stands, but the blanket sentence should be narrowed.

## Verdict

The core identity claim is independently reproduced: the `127/2` first dword
is the client's placement selection id, echoed by `127/42`, distinct from the
server-sent item/fixture ids, with a real keyed native lookup behind the
selection callbacks. No source, gateway, UI or native change was made. Raw
bytes stay in the lab; the executed checks and raw id tables are in
`verify.json` under the private report above.
