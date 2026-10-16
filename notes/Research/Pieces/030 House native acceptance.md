# 030 House native acceptance

Status: acceptance replay, gate report and evidence validator implemented and locally verified,
2026-10-09 04:10 UTC. Evidence label: **RECONSTRUCTED acceptance harness over ORIGINAL opcode,
denial, property and wiki facts**. This is not a native housing acceptance claim: no client run was
made, and the harness states the unproven gates itself.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_030.rs` | acceptance wire table, transcript schema, six gates, replay |
| `prototype/osfr-tools/tests/piece_030.rs` | 19 owned vectors plus the lead's evidence export, `#[path]` include |
| `scripts/pieces/piece_030.py` | read-only recovery checker, 9 probes |
| `tests/pieces/test_piece_030.py` | 10 stdlib checker tests and two negative controls |
| `lab/reports/piece-030-20261009T032826Z/` | RED/GREEN logs, checker report, replay export, controls, hashes |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save, release or other worker's
path was touched.

## Why this item exists

Items 006, 022, 023, 025 and 026 each solved one layer of housing, and every one of their notes ends
at the same gate: *did a real client enter, place, move, remove, visit and restart successfully?*
Item 025's "next gates" section hands that gate to item 030 explicitly. The acceptance verbs are
the item's own text; nothing existed to score them.

This item therefore ships two things that work today and are honest about the third:

1. a **replay** that drives the reconstructed state machine through enter, place, move, remove,
   visit and restart, writing a real snapshot and a real 19-row transcript;
2. a **gate evaluator** for a transcript captured from the running client, which refuses to pass a
   gate on missing, contradictory or unexplained evidence and can only report complete native
   evidence when the transcript's origin is a native capture;
3. no native claim. The replay's own report prints `native evidence complete: false`, and the only
   route to `true` is a capture the lead supplies.

## Original facts used

- Housing is the inner-opcode family of base opcode **127**
  (`lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs`, 42 members). The shipped client
  constructs the same numbers with `FUN_00aa4990(<opcode>)` immediately before the matching
  `::vftable`, and the checker compares all 42 against the enum name for name
  (`lab/decompiled/client/FreeRealms/pseudocode.c`, sha256 `f84a5867…`). The acceptance subset in
  `ACCEPTANCE_WIRE` is 1, 2, 3, 5, 6, 19, 28, 40, 41, 45, 46, 51 and 53.
- `HousingPacketHousingError` (46) reads one code word. The client's code 2 and code 6 call the same
  handler `FUN_00ab5590`, which prints the event key `HousingPermissionDenied` and the message
  **"You do not have permission to enter that house."**; locale id **421055** carries the same
  text. This item re-derived the whole switch independently: 1 → `00ab54b0`, 2/6 → `00ab5590`,
  3 → `00ab5670`, 4 → `00ab5750`, 5 → `00ab5830` (agreement with item 023).
- The client's placement denial path prints **"Fixture placement denied."** and reads
  **`HousingMaxFixtures`** from the property definition; no numeric limit is attached to either.
- The acceptance lot is `Houses.json` row 1
  (`lab/references/FreeRealms-Legacy/src/Resources/Houses.json`): property 1, zone 1, name id 6514,
  build area x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8.
- The wiki `Lots` page revision **49783** ("Lots are housing areas of property that the player can
  own and build on") corroborates the design at a non-technical level.

## What is reconstructed

Everything below is local policy, labelled `RECONSTRUCTED` in the module:

- the transcript schema (`SessionRow` plus the decoded `RowBody` facts a capture tool can supply)
  and the rule that a row's body fixes its opcode, so a mislabelled row is rejected;
- the six gate rules: enter needs the owner's request answered by zone data for the right zone;
  place needs a client request before the response and refuses a reused guid; move needs a
  transform that actually changes (or a lone `UpdateFixturePosition` confirmation) and refuses a
  move on an unplaced or already-removed fixture; remove needs the pickup request and its
  `FixtureRemove` answer; visit needs a second actor answered by zone data and permits only the
  owner or the admitted guest to edit (an unauthorised edit answered with code 2 is the accepted
  refusal, an unanswered one keeps the gate unproven, a successful one fails it); restart needs a
  restart marker and post-restart instance data whose fixture set and transforms equal what the
  transcript recorded;
- the replay cast: owner/guest/stranger guids, two fixtures, a local fixture cap of 64, and the
  request-then-response row grouping (the client's own handling of the placement reply is not
  recovered, so the grouping is a local convention, not an original order);
- the rule that `native_evidence_complete()` is only reachable for `EvidenceOrigin::NativeCapture`,
  and that the lead still confirms it.

## Verification

All commands ran from `%USERPROFILE%\Documents\Projects\CartoonMMO\project` with the private
toolchain (`scripts/cargo.ps1`, `..\lab\tools\wiki-env\Scripts\python.exe`). Logs and the exported
artifacts are in `lab/reports/piece-030-20261009T032826Z/`.

| Check | Command | Result |
| --- | --- | --- |
| RED | `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_030` | `couldn't find file tests\..\src\restoration\piece_030.rs` (`red-missing-module.log`) |
| GREEN | same command | **19 passed, 0 failed, 1 ignored** (`green-test.log`) |
| Lead export | same target, `exports_the_replay_evidence_for_the_lead`, `--ignored --exact`, `OSFR_PIECE_030_REPORT_DIR` set | 1 passed; wrote the snapshot, `piece-030-gates.txt/.json` and `piece-030-transcript.txt/.json` (`export-replay.log`) |
| Family sweep | `--test housing --test piece_022 --test piece_025 --test piece_026 --test piece_030` | 26 + 12 + 8 + 12 + 19 (+1 ignored) passed, 0 failed (`family-sweep.log`) |
| Format | private `rustfmt --edition 2024 --check` on both owned Rust files | exit 0 (`rustfmt-check.log`) |
| Checker | `python scripts/pieces/piece_030.py --report …` | 9/9 probes ok, exit 0 (`checker-run.log`, `piece-030-recovery.json`) |
| Checker tests | `python -m unittest discover -s tests/pieces -p "test_piece_030.py"` | 10 passed (`python-tests.log`) |
| Rust drift control | checker against a copy with opcode 41 → 42 | exit 1, `HousingPacketFixtureRemove is opcode 42, expected 41` (`negative-control-rust/`) |
| Corpus drift control | checker against a temp lab whose row 1 max x is 999.0 | exit 1, property and module probes drift (`negative-control-corpus/`) |

Three defects were found by these checks rather than by inspection: the acceptance expectation
treated the enum as 43 members when the supplied enum carries 42; the error-switch probe first
selected an unrelated switch in the export until it was anchored on the permission-denied target;
and the constant parser failed on the `[f32; 6]` type of `ACCEPTANCE_BUILD_AREA`. The first Rust
GREEN attempt exposed a fourth: the move gate scored a removed fixture's placement echo as a
post-removal movement. All four were fixed, and the Python suite keeps a synthetic regression for
each.

The replay's own report (exported artifact) is:

```
origin: local replay
enter    PASS     enter request at row 1 answered by zone data at row 2 for zone 1
place    PASS     place request at row 4 answered by fixture … at row 5
move     PASS     1 fixture movement(s) changed the transform
remove   PASS     pickup request at row 12 answered by fixture remove at row 13
visit    PASS     guest … entered at row 14 and received zone data at row 15
restart  PASS     restart at row 18 restored 1 fixture(s) in instance data at row 19
native evidence complete: false
```

## Deliberate gaps

- **No server-to-client body layout is recovered.** Opcodes 2, 28, 40, 41, 51 and 53 remain
  undecoded; item 025's `UNRECOVERED_OPCODES` lists them. A transcript row therefore carries the
  facts a capture tool has decoded, not bytes this module parses.
- **No packet writer.** Nothing here can answer a client; that stays with item 025's codec.
- **No capture exists.** The native client has never sent or received anything this item scored, so
  every gate is currently demonstrated only against the reconstructed stack.
- **No lifecycle beyond the six verbs:** no collision, snap, drag preview, landmarks, upkeep,
  visitor caps, invitations, naming, interactive furniture or save migration. Those stay with items
  006/027/028/029 as their notes describe.
- **Local numbers stay local:** the replay's fixture cap 64, its guids and its furniture identities
  are labelled; no recovered field carries any of them. `HousingMaxFixtures` still has no recovered
  value.

## Native and integration gates

1. Lead registration: `src/restoration/piece_030.rs` is not in `src/lib.rs`; the test includes it
   by `#[path]` and the lead drops that include on registration.
2. Wire gate (item 025): recover the server replies 2/28/40/41/51/53 so a placement answer can
   reach the client.
3. Native run (this item's acceptance): with the integrated server, enter the lot, place, move,
   pick up, have a guest enter, restart the server and re-enter. Capture the events as rows and call
   `evaluate_transcript(EvidenceOrigin::NativeCapture, …)`; the module then reports per-verb
   `PASS`/`FAIL`/`UNPROVEN` and only a complete pass sets `native evidence complete: true`.
4. Lead confirmation: even a complete native transcript is evidence for the lead's review, not an
   acceptance this module grants.

Skills applied: **RTK** (commands ran normally; no `rtk proxy` fallback needed), **ponytail** (one
transcript type, one wire table, one pass per gate; the state machine and codec were reused instead
of re-implemented), **caveman** (terse reporting; the note itself stays complete), **game-recon /
community-first research** (the client export and pinned community packets were read before any
decoding), **reverse-engineering** (the housing error switch, the client constructors and the
denial strings were read out of the Ghidra export), **game-automation** (not applied: only the lead
controls the shared native window; the module defines the evidence contract that run must satisfy).
`asset-pipeline` did not apply: no asset is produced.

Cross-links: [[Housing Restoration]], [[025 Furniture placement client wire]],
[[026 Housing inventory transactions]], [[023 House permissions]], [[../Design/MMO Worker
Contract]], [[../MMO Work Queue]].
