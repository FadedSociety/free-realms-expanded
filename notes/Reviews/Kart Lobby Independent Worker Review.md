# Kart Lobby Independent Worker Review

Independent read-only **worker** verification of the lobby 102/2 framing claims in
[[Research/Kart Lobby Definition Framing]] (piece031). No source, content, save,
UI, release or process change; writes are this note and
`lab\reports\kart-lobby-independent-worker-review-20261009T155631Z/`. Provenance: piece031 authored both the lobby and
finish notes and then the checker under
`lab/reports/kart-lobby-finish-independent-review-20261009T154627Z`, so this is
an independent worker pass, not a re-run of the author checker.

## Verdict

Framing, five-table byte layout, exact consumption, the community writer subset
and the 34 subtype constants re-derive cleanly from the pinned capture,
decompile and PE bytes. The finish review's one unverified item is upgraded: the
sort callback `00b9abe0` exists in the PE (absent from `pseudocode.c`) and its
byte-level grouping by `+0x1614` → laps `+0x15ec` → total `+0x1620` decodes as
claimed. Remaining unknowns stay explicit below.

## 1. Framing and body (my own parser)

- Both op102/sub2 files are 2393 bytes and byte-identical (file sha
  `4f6d75c49554e39588a8117a94a650b0154f5d40bb0bf6a45a1e0073a2cb6932`).
- Tunnel: outer u16 `5`, flag u8 `1`, inner length u32 `2386`; inner opcode
  `102`, sub `2`, body length `2378`; relation `inner_length = 8 + body_length`
  holds. Body sha256 `51c8f011c781fe376496c851889380ba36f6bbed911daa60eb964ecc71fd5a0c`.
- Five tables in the claimed order parse with counts **6 / 30 / 43 / 37 / 17**
  and consume **2378 of 2378** bytes; per-table budgets 124 / 484 / 692 / 666 /
  412 and display-order blob total 70 — all recomputed in
  `lobby-body-recompute.json`.
- Games rows match the note exactly, including row 1 `(1,1,1,37,3030)` and row 6
  `(6,6,6,60,417078)`; display rows 1-6 carry locale ids 3032/3033/3034/3036/
  3037/3040 with the digits '2','4','6','1','3','5'.

## 2. Native chain (decompile + PE)

- `FUN_0091e060` calls `FUN_008d67a0(&local_34)` and gates on
  `if ((local_24 == '\0') && (local_38 == 2))` — the sub==2 check, verified.
- `FUN_008f74c0` backs the header reader and calls `FUN_008bfc00`; that function
  reads the i32 length and bounds `cursor + 1` against the end.
- `FUN_00919b60` verbatim dispatches the five parsers in order at
  `+0x130` (games), `+0x50` (options), `+0x88` (values), `+0xc0` (display
  order), `+0xf8` (settings).

## 3. Community writer subset

`lab/references/Sanctuary/src/Sanctuary.Packet.Common/LobbyGameDefinition.cs`
(and its FreeRealms-Legacy copy) declares
`Dictionary<int, LobbyGameEntry> GameEntries` with a row serializer of
`Id, Type, Unknown, NameId` (four ints), and the packet serializer writes the
dictionary plus four zero counts. Six rows therefore serialize to
`4 + 6*(4+16) + 16 = 140` bytes — a strict subset of the 2378-byte native body
whose four remaining tables carry 30/43/37/17 rows.

## 4. Inner 33 subtype map (bounded)

`FUN_00b992d0(<subtype>)` call constants extracted from the pinned decompile are
contiguous **1..34** (1-10 decimal, 0xb..0x22 hex) and 34 distinct
`VehicleRacePacket*` class names are present. Count, continuity and the three
finish-critical entries (1, 12, 14) are verified; the full per-name binding for
every subtype is author-reported (vftable association not re-derived name by
name).

## 5. Sort callback 00b9abe0 (upgrade)

`00b9abe0` is absent from `pseudocode.c` but present in the PE. Raw bytes decode
as: `comisd xmm0,[esi+0x1614]` / `[edx+0x1614]` flag group, then
`mov/cmp [..+0x15ec]` laps, then `movss/comisd [edx/esi+0x1620]` total time —
matching the claimed sort order. Evidence window is recorded in
`final-source-checks.json` (manual byte decode; no full disassembler run).

## 6. Unverified (explicit)

Table 2/3 semantics and consumers; display-order byte-string consumer name;
settings name-id to string resolution; the retail catalogue (capture is the
"Test Lobby Game" catalogue); per-constructor name binding for all 34 subtypes;
and client tolerance for trailing bytes after the five tables (exact
consumption is proven, tolerance is not tested).

Machine-readable bundle: [`review.json`](lab\reports\kart-lobby-independent-worker-review-20261009T155631Z/review.json) sha256
`36c12ecc7ffe2ebe66ebb9ba97f7185a2a0e7f713b80d11d573a49119bc85a2c`.
