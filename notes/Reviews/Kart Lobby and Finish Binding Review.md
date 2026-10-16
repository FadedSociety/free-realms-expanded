# Kart lobby and finish binding review

Same-worker verification with a separately implemented checker, October 9, 2026 (UTC). Scope: verify
[[Research/Kart Lobby Definition Framing]] and [[Research/Kart Race Finish Protocol]] against the
pinned Ghidra export, the pinned PE, the shipped UI Lua and the already decoded capture bodies, plus
the two builder reproducers. No source, content, save, UI, release or other note was modified; the
reviewer re-derived every checked claim with its own checker instead of importing the builders'
scripts. Root's native Mining/Harvest work was untouched.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | verdict, evidence, corrections, residual risk |
| `lab/reports/kart-lobby-finish-independent-review-20261009T154627Z/review.json` | original checker output, sha256 `40f869033406df3bb4bcc63d1d25b62c3150488b5ace4973e8b6b7248c095111` |
| `lab/reports/kart-lobby-finish-independent-review-20261009T154627Z/review-kart-lobby-finish.py` | the reviewer's own checker (no builder code imported) |
| `lab/reports/kart-lobby-finish-independent-review-20261009T154627Z/review-run.log` | run output |

## Verdict

**Both notes pass this separately implemented checker, with one minor wording correction** (below). Ten independently implemented
checks pass: the 102/2 framing, the five tables and their byte budget, the community-subset claim,
all 34 inner-33 subtype bindings, the 33/14 payload layout (machine-verified against the racer
stores), the captured distributions (9,358 progress frames, 7 state frames, 0 finished), the
results data-source binding, the results trigger, the no-141-inference claim, and the synthetic
vectors' self-consistency.

## What was re-derived, and how

**102/2 framing.** Re-read from `FUN_0091e060` (two `i16` header words, sub must be 2),
`FUN_008bfc00` (`i32` length with a `cursor + length <= end` bound and a `(payload, length)` out
pair) and `FUN_00919b60` (five table parsers at `+0x130/+0x50/+0x88/+0xc0/+0xf8`). The reviewer's own
decoder consumed **2378 of 2378** bytes of both captured bodies (which are byte-identical,
sha256 `51c8f011…`), yielding table counts 6/30/43/37/17 and row 0 `(1,1,1,37,3030)`. Row layouts
were re-derived per parser: games 4×i32, options 3×i32, values 3×i32, display order 2×i32 + i32
length + bytes, settings 5×i32 in the order `0,1,2,4,3`.

**Community subset.** The pinned `LobbyGameDefinition.cs` writes the entries dictionary plus four
zero counts; with a 6-row table that is `4 + 120 + 16 = 140` bytes, which cannot be the 2378-byte
capture. The builder's "community writer is a subset, not the native framing" statement is correct.

**Subtype map.** Re-extracted from the constructors (`FUN_00b992d0(<n>)` + vftable stores) without
reusing the builder script: all 34 subtypes are present and unique, including
1 `UpdateVehicleGameState`, 12 `DestroyProxiedVehicleRace`, 14 `UpdateVehicleRaceProgressData`.

**33/14 layout.** Re-derived the read order from `FUN_00b9fc80`: 4+4+4+4+4+4(+1 byte)+4+4+4+4 then
four `u64` pairs = **73 bytes**, and confirmed with machine bytes from `FUN_00ba74d0` that the
dispatcher stores them at the claimed racer offsets (`00ba7813 [ebx+0x15ec]`, `00ba7820
[ebx+0x1610]`, `00ba782f …1618`, `00ba7840 …161c`, `00ba7851 …1620`, `00ba7860/786b …1614`) with the
stack slots equal to object base `esp+0x164` plus the packet offset, which maps to payload offsets
`0x14` (laps), `0x08` (slot), `0x1d`, `0x21` (best lap), `0x25` (total time), `0x10`, `0x19` (lap
limit) and `0x18` (local flag).

**Capture.** All 9,358 `33/14` files decode with a 73-byte payload: laps ∈ {0,1}, lap limit = 3 in
every frame, local flag 0, racer slots 0..7, 8 distinct racer guids, best/total times zero except
494 frames, and **no frame at or above the lap limit** — the capture has no finish. The 7 `33/1`
files are 15 bytes each with payload states `3,4,5,6,7,8,9`; `33/12` never appears.

**Results binding.** `FUN_00ce3330` row writes (`+0x18 ← racer+0x161c`, `+0x1c ← racer+0x1620`,
`+0x20 ← (processor+0x1a0 < racer+0x15ec+1)`) and the `FUN_00ba6530` trigger (`lap limit <= laps`
and `phase != 10` → hide HUD, `RaceResults:Show`, phase = 10) match the note, as do the Lua column
list, the `finished == 1` gate and the `RaceResults:Populate` call in `FUN_00ba2700`. The reviewer
also confirmed the results path (`00ba6530`, `00ce3330`, `00ba2700`) contains no `141` reference, so
treating the matchmaking pair as unrelated is correct.

**Builder reproducers.** Both were re-run unchanged: the lobby reproducer still reports
`proof_sha256 da47c416…` / `vectors_sha256 edfd1fca…`, and the finish reproducer still reports
`proof_sha256 09bf7bc2…` / `vectors_sha256 16a70fce…`, identical to the hashes recorded in the two
notes. `write_exact` idempotency held on re-run.

## Correction (minor, wording only)

The finish note says the 494 frames with non-zero times "match the lap-1 frames". The exact
cross-tabulation is `(laps=0, no times) 8,864`, `(laps=0, times present) 5`, `(laps=1, times present)
489`. So 494 frames carry times, and 489 of them also report `laps = 1`; five frames carry the times
with `laps` still 0 (the lap-completion frame). This changes no binding or conclusion and is left
un-edited in the research note per review scope; the lead may fold the wording into the next
revision.

## Residual risk and blockers

1. **No finish is captured.** The server's exact finish frame order (state value, final progress
   block, results) is still a client-requirement reconstruction; nothing here proves how retail
   ordered the last frames.
2. **Client-derived fields.** `Position` (sorted racer index) and `Finished` (lap comparison) are
   computed locally; a server writer must supply laps, lap limit and times correctly, because there
   is no placement or finished field on the wire.
3. **Unnamed fields.** `33/14` payload `0x0c`, `0x10`, `0x1d` and the four trailing `u64`s remain
   unmapped; the vectors fill them with zeros, which is unverified against a real server send.
4. **Test catalogue.** The only captured 102/2 is a test catalogue ("Test Lobby Game"), so retail
   per-game rows and option tables are still unrecovered.
5. **Vectors are private.** The synthetic payloads live in the lab only and are not content; using
   them in production requires a lead decision and the missing 102/2 catalogue data.

## Verification

```
& '<lab>/tools/wiki-env/Scripts/python.exe' review-kart-lobby-finish.py
{"checks": 10, "lobby_bodies": 2, "progress_frames": 9358, "state_frames": 7,
 "review_sha256": "40f869033406df3bb4bcc63d1d25b62c3150488b5ace4973e8b6b7248c095111"}
```

The checker re-derives each claim from the pinned sources; re-running it reproduces identical
output. Both builder reproducers were re-run afterwards with unchanged recorded hashes.

Cross-links: [[Research/Kart Lobby Definition Framing]], [[Research/Kart Race Finish Protocol]],
[[Research/Go Kart Native Integration Readiness]].


## Lead provenance qualification

The same worker, piece_031, authored both research notes and this checker. Import-free reimplementation provides a distinct check but does not constitute independent-worker acceptance. A different worker is reviewing consequential native bindings. The five lap-zero/non-zero-time records are now explicitly described in the finish note; original private proofs and vectors remain preserved byte-identically. Root made this wording/provenance correction after interrupting the author.
