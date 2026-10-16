# 070 Quest journal wire

Status: ready for lead review, 2026-10-09 05:20 UTC. This piece removes the **last captured bytes**
from the opening quest-journal writer: the two unnamed i32s at the end of the root tail. It ships a
typed, validated tail writer plus an independent writer-verification (`scripts/pieces/piece_070.py`)
and a native opcode-49 replay audit. It grants nothing, edits no content and does not claim a
playable journal.

Owned paths: `prototype/osfr-tools/src/restoration/piece_070.rs`,
`prototype/osfr-tools/tests/piece_070.rs`, `scripts/pieces/piece_070.py`,
`tests/pieces/test_piece_070.py`, this note and `lab/reports/piece-070-20261009T051415Z/`.
No shared `lib.rs`, gameplay, gateway, Cargo manifest, launcher, content or state file was touched.

## What was still captured

`opening_journal.rs` already serialized every proven quest/objective field, but kept
`root_unknown: [u8; 8]` verbatim from the reference span and appended those bytes to every rebuilt
journal. They are the writer's `ClientQuestData.Unknown4`/`Unknown5` — the last two i32s after
`completed count`, `selected quest id` and the select flag. This piece makes them writer output.

## Recovered original facts

**Field order.** The supplied compiled writer (decompiled `lab/decompiled/server/Gateway/
Gateway.Login/ClientPcData.cs`, quest block before `ClientAchievementData`) emits
`quest count, records..., Unknown(i32), Unknown2(i32), Unknown3(bool), Unknown4(i32),
Unknown5(i32)`. Native `FUN_00940c60` (`pseudocode.c` 1104250-1104355) stores the same five values at
quest-manager `+0x10fc, +0x10f4, +0x10f8, +0x1100, +0x1104`; the shorter record variant
`FUN_00940e60` (1104356-1104421) also writes `+0x1100`.

**The two unnamed fields are write-only.** Grepping the whole preserved pseudocode for those
offsets returns only those writers (`0x1100`: lines 1104333/1104338/1104410/1104415; `0x1104`:
1104342/1104347). The one other `0x1104` hit (line 1589775) dereferences `param_1 + 0x1104` as a
pointer on an unrelated object. No preserved function reads the quest manager's pair back, so both
are presentation-neutral state with **unrecovered semantics**.

**Reference span.** `lab/references/rust-gameplay/quest-journal.bin` is the exact 7,970-byte
captured span (sha256 `1b36c0f5…1037`), tail at offset 7,953 with values
`241, 2513, true, 10, 30` — identical to the recorded `ClientQuestData`
`Unknown/Unknown2/Unknown3/Unknown4/Unknown5` of
`lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`. Note the recorded
`quests.json` `offset` 157832 indexes the **tunnel packet**; the supplied serializer's inner
payload puts the same span at 157,819 (verified against `lab/reports/quest-original-serializer-oracle.bin`).

**The pair is constant.** 39 preserved self inputs (the recorded sample, the
OpenSourceFreeRealms reference copy and every run analysis input) declare `10`/`30` while the named
tail fields vary over 12 distinct states. Rows are in `journal-wire.json` → `tail_evidence.rows`.

## Reconstruction boundary

`JournalRootTail::opening_campaign(completed, selected, auto_select)` emits the recovered constants
`10`/`30` for reconstructed campaigns; `with_recovered_fields(...)` exists only for re-serializing a
span that declares other values (the oracle path). Validation is domain-only: completed count
non-negative, selected id `-1` or positive, bool bytes exact. The semantics of the two unnamed
fields stay a documented gap; nothing derives them from gameplay.

## Writer-verified serialization

`scripts/pieces/piece_070.py` runs the **supplied compiled** serializer
(`QuestSerializerOracle.exe` → `Gateway.dll ClientPcData.ReadFromJSON`) on two synthetic inputs with
different declared tails:

| variant | declared tail (completed, selected, flag, u4, u5) | serialized tail |
| --- | --- | --- |
| recovered-pair | 0, 910001, false, 10, 30 | `00000000 b1e20d00 00 0a000000 1e000000` |
| altered-pair | 7, -1, false, 7, 45 | `07000000 ffffffff 00 07000000 2d000000` |

For each output the checker parses the journal at the recorded offset and requires the declared
17 bytes exactly, the prefix before the span byte-identical to the captured payload, and the suffix
after the span byte-identical too. That proves the tail layout for arbitrary values rather than
replaying the captured pair. Outputs and hashes: `oracle-output-*.bin` in the report bundle.

## Native replay audit (recorded run 171251)

`--run client-baseline-20261008T171251Z` replays every decoded opcode-49 packet against the content's
objective order and the run's authoritative before/after states:

| frame | packet | meaning |
| --- | --- | --- |
| 3876 | 49/9 quest 910004 key 91000406 current 1 progress 1.0 | live objective update |
| 3877 | 49/10 quest 910004 key 91000406 progress 1.0 | objective completion |
| 4071 | 49/4 quest 910004 | turn-in removes the quest |
| 4072 | 49/12 count 4 | CompletedQuestCountUpdate |

Checks passed with zero errors: base rewarded stages 3 + 1 removal = 4 = the emitted absolute count
= the after-state rewarded count; the live key is objective index 5 of stage 910004 and the wire
counter matches the authoritative `counts[5] = 1`; lengths are the recovered 10/27/23/10 bytes;
4158 gateway frames / 4156 CRCs / complete reassembly unchanged. This accepts the **wire**
behaviour of labels/progress/removal, not their on-screen rendering.

## Verification

```
prototype/osfr-tools
& '..\..\scripts\cargo.ps1' test --test piece_070
  RED  before implementation: couldn't find file `tests\..\src\restoration\piece_070.rs`
  GREEN: 3 passed; 0 failed; 1 ignored
$env:OSFR_JOURNAL_REFERENCE='%USERPROFILE%/Documents/Projects/CartoonMMO/lab/references/rust-gameplay'
& '..\..\scripts\cargo.ps1' -CargoArgs @('test','--test','piece_070','--','--include-ignored')
  GREEN: 4 passed; 0 failed  (private span: tail 241/2513/true/10/30; only the 17 tail bytes change)
& '..\..\scripts\cargo.ps1' build --lib        -> Finished `dev` profile
& '..\..\scripts\cargo.ps1' test --test opening_journal  -> 6 passed; 0 failed; 2 ignored

project
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_070.py"
  GREEN: 13 passed (Ran 13 tests, OK)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_070.py' `
    --run client-baseline-20261008T171251Z `
    --content '..\lab\references\opening-journal-text-v2\content' `
    --reports-dir '..\lab\reports\piece-070-20261009T051415Z'
  reference tail 241/2513/true/10/30; 2 oracle variants; 4 run packets; run_errors []
```

## Explicit gaps and remaining gates

* The two unnamed fields stay semantically unrecovered; only their byte positions, write-only nature
  and captured values are proven.
* Integration seam (lead-owned): register `pub mod piece_070;`, replace `OpeningJournal::root_unknown`
  with `JournalRootTail::from_span` on load and `opening_campaign(...).write(...)` when rebuilding,
  and delete the duplicated tail walker in `opening_journal.rs`. Until then the captured bytes are
  still copied in production.
* Native gate: the journal page was not visually re-checked here. The recorded run 171251 emitted no
  usable journal screenshot (its `journal-*` images show the world map), and a self payload's journal
  offset varies with identity length, so per-run self checks need the lead's content/offset pairing.
  Next native step: turn in a quest, reopen Hero's Journal, and confirm the row is gone, the
  completed count and labels render, then reconnect and repeat. The 49-stream audit above can be run
  on that run with `--run <name> --content <content dir>`.
* No reward, currency, item, save or content behaviour is changed by this piece.

Cross-links: [[../Native Opening Quest Journal]], [[../Native Quest Journal UI]],
[[../Native Journal Live Objective Keys]], [[../Opening Journal Reconstructed Labels]],
[[../../Design/MMO Worker Contract]], [[../../MMO Work Queue]].
