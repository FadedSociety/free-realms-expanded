# 082 Inventory search crash

Candidate artifact for work item 082. It preserves the fault evidence, pins the
exact data the crashed run received and ships a deterministic preflight for the
client's local search row model. It does not fix the crash and does not claim a
cause: the native fault address is still missing, so the native gate is open.

Allowed paths used: `prototype/osfr-tools/src/restoration/piece_082.rs`,
`prototype/osfr-tools/tests/piece_082.rs`, `scripts/pieces/piece_082.py`,
`tests/pieces/test_piece_082.py`, this note, and the private bundle under
`../lab/reports/piece-082-20261009T052121Z/`. No shared file was edited.

## What the client actually does (original)

`../lab/decompiled/client/FreeRealms/pseudocode.c`:

- `1170126` / `1170136` - `ClientPlayerItemManager` constructs
  `BaseClient.InventoryItemsData` and a second `InventoryDataSource` named
  `BaseClient.SearchInventoryItemsData`; `1170140` writes the search flag at
  object offset `0x2e8` on the second source only.
- `1340855` - `FUN_00a6deb0` is that source's constructor; it owns the
  `IString<unsigned short>` search text at offset `0x2f0`, an applied-text copy
  at `0x2ec` and a pending flag at `0x2d0`.
- `1341483` - `FUN_00a6e910` refreshes the source. Flag clear: plain rebuild.
  Flag set: load the stored search string (`FUN_0083d250`), rebuild the visible
  rows through the per-row predicate virtual at vtable slot `+0x3c`
  (`FUN_00a6e7b0`, `1341392`), copy the applied text, clear the pending flag.

So the Stuff search field is a purely local row rebuild. Matching that with the
captured traffic, the crashed run's last client-to-server frame precedes the exit
by 21.6 s; no search request appears on the wire. That is a fault-path fact, not
a cause.

## Fault evidence (re-verified from the preserved files)

Run `client-baseline-20261008T033402Z`, manifest SHA-256
`644cd786da7d60defe9cddf91d56d67e67f812f098b41a2cbea16eab7eeb44ee`, the same
hash the preserved comparison record lists:

- exit code `3221225477` / `0xC0000005`, `client_access_violation: true`;
- elapsed `90.265 s`, 1412 captured frames, last client-to-server frame at
  `t = 68.672 s` (`frames/001410.bin`, opcode 6) - 21.593 s before the exit;
- `FreeRealms.log` SHA-256 `e751e429da4e821af2b904a6aa8249b2a5a1f40f112bcf675331739a470c0b3e`
  (43283 bytes) ends after normal startup/headshot/render-thread lines, with no
  exception stack;
- saved gameplay state is byte-identical before and after the crash: 35 coins,
  definition 1195 instance 1077 count 10, quest 900001 completed/rewarded,
  trial best 18456 ms - no reward or loss is attributed to the crashed run.

The parent's original write-up is `../lab/reports/native-inventory-search-crash.md`;
this note re-verifies its machine-checkable claims and keeps the rest as its
stated hypotheses.

## What the crashed run's inputs contain (original, parsed here)

The client's own received catalogues, not a community substitute:

- 1386 owned instances and 1383 definition records (1329 unique ids; the 54
  repeats are byte-identical) from
  `../lab/references/rust-gameplay/inventory.json`;
- every definition record was parsed with the supplied server's writer order
  (`Gateway.Player/PlayerCode.cs:11-45`: Id, NameId, DescriptionId, Icon.Id,
  Icon.TintId, Tint, Unknown5, Cost, Class, ProfileOverride, Slot, two booleans,
  ModelName, TextureAlias, GenderUsage, Type, CategoryId, ...), cross-checked
  against the prepared fields (class, slot, icon tint, model, texture);
- the 44/2 table the client was sent has 71 categories and 74 group
  associations, and the group rows still reference 9 category ids the table does
  not define: `58, 60, 401, 402, 403, 404, 405, 406, 407`;
- 3 owned instances cannot resolve a definition at all:
  `1145/7960` (rental), `4594/78398`, `4595/78399`;
- 63 received definitions reference categories absent from the 71-row table:
  category `0` x53, `127` x6, `148` x1, `149` x3 - matching the earlier report's
  figure, now sourced from the wire records rather than the community catalogue;
- every referenced `NameId` resolves in the community locale export
  (`../lab/reports/locale-id-to-text.json`), so localization is not a gap here;
- the pinned community catalogues are not a substitute for the wire records:
  17 of 1329 definitions disagree on `NameId`/`CategoryId`, and 4 definition ids
  (`6346231, 35463452, 81256176, 163632473`) have no community row at all. The
  private Rust tooling's content is what the client held, so the preflight reads
  the wire order.

## Reconstruction: the search row preflight

`piece_082.rs` models the row list the search source walks: each owned instance
projects through the received catalogue, and the report lists rows whose
definition, category or localized name cannot resolve. `scripts/pieces/piece_082.py`
runs the same model over the preserved lab inputs and produces a compact
`preflight-input.json`; the Rust private test consumes that file and asserts the
identical counts and SHA-256 fingerprint, so both implementations agree on the
data state a native run would see.

Over the crashed run's inputs the preflight reports 1386 rows, 3 unresolved rows,
63 definition rows with absent categories and fingerprint
`72260f1c38e5dfe0013ea14130f8f99fe5d2be4f4fa8b6ec76b0ea3a8fe953bd`.

Labelled negative result: under a reconstructed case-insensitive substring rule
the crashed query `Wheeler` matches 10 owned rows (`1033, 1077, 1173, 1231,
1402, 1581, 2034, 3043, 3167, 3547`) and every one of them resolves its
definition, category and locale name. The simplest "the query hits an unresolved
row" explanation for that exact query is therefore not supported by this
dataset. The match rule itself is a reconstruction; the native predicate at
`+0x3c` is not recovered, so this prunes one variant and proves nothing else.

## Boundaries and gaps

- Original: the pseudocode offsets/addresses above, the fault-evidence numbers,
  the parsed 1386/1329/71/74 counts, the 3 unresolved instances, the 63
  absent-category definitions and the 9 dangling group associations.
- Reconstructed: the row model, the case-insensitive substring preview, and the
  fingerprint's canonical form. None of these is native behaviour proof.
- Unrecovered: the row predicate's actual match rule, the refresh path's full
  call graph, and any faulting instruction. No fault address, stack or dump was
  found for the crashed run, and none is invented here. Search stays a known
  limitation; none of this makes search safe.
- Nothing here grants items, currency or XP, edits content or state, or changes
  the native UI.

## Artifacts, commands and observed results

Private bundle `../lab/reports/piece-082-20261009T052121Z/`: `fault-evidence.json`,
`search-preflight.json`, `findings.json`, `provenance.json` (all nine pinned input
hashes match) and `preflight-input.json`.

```powershell
# checker over the preserved inputs (writes a new private bundle)
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_082.py

# focused Python tests
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_082.py" -v

# focused Rust tests
& scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_082')

# cross-language oracle against the produced private input
$env:OSFR_PIECE_082_PREFLIGHT = [IO.Path]::GetFullPath('..\lab\reports\piece-082-20261009T052121Z\preflight-input.json')
& scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_082','private_crash_run_preflight_matches_the_python_checker','--','--ignored','--exact','--nocapture')
```

Observed: Python `15 passed`; Rust ordinary `7 passed, 0 failed, 1 ignored`;
private oracle `1 passed, 0 failed`. RED before implementation: the Python suite
failed with `FileNotFoundError` for `scripts/pieces/piece_082.py`, and the Rust
test failed to compile with `couldn't find file tests\..\src\restoration\piece_082.rs`.

## Remaining gates for the lead

1. Controlled native reproduction with the pinned fingerprint: start a fresh run
   from this exact saved state (`lab/state/rust-gameplay-character.json`), enter
   the world, open Stuff, search `Wheeler`, and capture the exit code plus a
   fault address or dump. The preflight fingerprint identifies the data state;
   this candidate does not change it.
2. A fault address, WER dump or debugger backtrace is the missing evidence that
   would let a narrow fix be claimed. Without it, the ranked hypotheses in
   `native-inventory-search-crash.md` stay hypotheses.
3. Lead registration: `pub mod piece_082;` under `src/restoration` once the
   module is accepted (the owned test already `#[path]`-includes it).
