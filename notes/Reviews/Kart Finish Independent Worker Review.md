# Kart Finish Independent Worker Review

Independent read-only **worker** verification of the kart race finish binding
claims in [[Research/Kart Race Finish Protocol]]. No source, content, save, UI,
release or process change; writes are this note and
`lab\reports\kart-finish-independent-worker-review-20261009T155257Z/`.

## Provenance (same-worker qualification)

`Research/Kart Race Finish Protocol.md` and
`Research/Kart Lobby Definition Framing.md` were both authored by piece031, and
piece031 then authored the checker under
`lab/reports/kart-lobby-finish-independent-review-20261009T154627Z`. That
checker is a same-author enumeration, not independent worker acceptance. This
review independently re-derives the **finish** claims; the lobby 102/2 framing
and its five tables are explicitly **lobby-pending** here (bounded observation
only, see below).

## Verdict (finish)

Finish binding confirmed: the 33/14 payload is 73 bytes with the claimed field
order, the native dispatcher's machine stores land on the claimed racer fields,
the results data source and trigger match the shipped Lua consumer, and the
capture contains **no finish** — 9,358 progress frames, only laps 0/1 against
lap limit 3. Two bounded qualifications: the payload-offset → object-field
mapping is corroborated by the machine stores plus the capture decode rather
than re-derived instruction-by-instruction, and the claimed sort predicate
`00b9abe0` could not be located in the pinned pseudocode export.

## 1. Capture re-derivation (my own parser)

`capture-recompute.json` (9358 `33/14`, 7 `33/1`, 0 `33/12`):

- Every progress payload is exactly **73 bytes**; the payload is file bytes
  `[11:]` and the index's `inner_length = payload_len + 4` (the 73-byte payload
  files are 84 bytes).
- laps `{0: 8869, 1: 489}`, lap limit `{3: 9358}`, local flag `{0: 9358}`,
  slots 0..7, 8 distinct guids, **0 frames at or above the lap limit**.
- Cross-tab: `(laps=0,no times) 8864`, `(laps=0,times) 5`, `(laps=1,times) 489`,
  `(laps=1,no times) 0` — this is the exact corrected form of the author's
  "494 frames match the lap-1 frames" wording (494 carry times; 5 still report
  laps 0).
- State frames: times 0.441/2.472/2.799/7.692/8.141/8.736/13.951 with values
  3,4,5,6,7,8,9; last-8 progress rows reproduce the author's `proof.json`
  values (e.g. slot 5 best/total 58.52, slots 6/7 laps 0 with times 0.0).

## 2. Machine offsets (raw PE bytes, ImageBase 0x400000)

`pe-machine-offsets.json` — every cited store verified byte-for-byte:

| address | bytes | store |
| --- | --- | --- |
| 00ba7813 | `89 8B EC 15 00 00` | `[ebx+0x15ec] = ecx` (laps) |
| 00ba7820 | `89 93 10 16 00 00` | `[ebx+0x1610] = edx` (slot) |
| 00ba782f | `F3 0F 11 83 18 16 00 00` | `[ebx+0x1618]` (0x1d float) |
| 00ba7840 | `F3 0F 11 83 1C 16 00 00` | `[ebx+0x161c]` (best lap) |
| 00ba7851 | `F3 0F 11 83 20 16 00 00` | `[ebx+0x1620]` (total time) |
| 00ba786b | `F3 0F 11 83 14 16 00 00` | `[ebx+0x1614]`, guarded by `cmp [esi+0x194],9` / `jne` at 0x00ba7860 |
| 00ba77c4 | `89 86 A0 01 00 00` | `[esi+0x1a0] = eax` (lap limit) |
| 00ba77ca | `88 8E A4 01 00 00` | `[esi+0x1a4] = cl` (local flag) |

## 3. Native source, data source, trigger, Lua

- Dispatcher `00ba74d0`: `case 0x1/0xc/0xe` present and
  `*(undefined ****)(param_1 + 0x194) = local_11b4;` (state store) verified.
- Parser `00b9fc80`: bounded cursor reads and the local-flag store
  `*(bool *)(param_1 + 0x48) = **(char **)(param_2 + 8) != '\0';` verified.
- `00ce3330`: `0x161c → row+0x18` (BestLapTime), `0x1620 → row+0x1c`
  (TotalTime) and `uVar13 = (uint)(*(int *)(*(int *)(param_1 + 0x254) + 0x1a0) < *(int *)(iVar5 + 0x15ec) + 1)`
  → `row+0x20` (Finished) verified verbatim.
- `00ba6530`: `param_1[0x68] <= *(int *)(iVar3 + 0x15ec)`,
  `*(int *)(iVar3 + 0x1624) != 10`, three `FUN_009797a0("RaceResults:Show",0,0);`
  calls and the phase write `= 10` verified.
- Lua `ScriptsBase.lua`: `RaceResultsCol` at line 48292 with
  `Position/Guid/Name/BestLapTime/TotalTime/Finished`, `if finished == 1 then`
  at 37351, `SetPlayerRaceResults` at 37354, `DsTable.Find(...)` at 37360.
- **Not verified**: `00b9abe0` (claimed sort predicate) is absent from the
  pinned `pseudocode.c`; the +0x1614/laps/time grouping claim stays unverified.

## 4. Vectors vs captured traffic

`vectors.json` decode with my own layout: `progress_finish` 73 B, laps 3 /
limit 3 → Finished true; `progress_mid_race` 73 B, laps 1 / limit 3 → false.
The last captured progress payload is laps 0 / limit 3 with zero times →
the capture never reaches the vector's finish condition. No finish packet was
observed anywhere.

## 5. Lobby (bounded, pending)

Not re-derived here. Only the bounded observation that the two `op102/sub2`
captures exist and are byte-identical
(`racing-13231-flow0-frame15239-op102-sub2.bin`,
`racing-23867-flow0-frame44523-op102-sub2.bin`, file sha256
`4f6d75c49554e39588a8117a94a650b0154f5d40bb0bf6a45a1e0073a2cb6932`, 2393 B).
The 102/2 framing and the five tables remain author claims pending an
independent pass.

## Limits

No finish was observable; `Finished`/`Position` are client-derived; payload
fields 0x0c/0x10/0x1d and the four trailing u64s stay unnamed; the sort
predicate and the full stack-slot arithmetic were not re-derived.

Machine-readable bundle: [`review.json`](lab\reports\kart-finish-independent-worker-review-20261009T155257Z/review.json)
sha256 `3dfa2359ba817dfb822c3a8ef449c2795686a92b81f36ca4737ffaa4624f37b5`.
