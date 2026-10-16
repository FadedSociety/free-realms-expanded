---
type: review
task: W115
date: 2026-10-16
---

# IDA Boundary Review

**Review gates passed; staged only.** All 340 high-severity `existing_function_boundary_mismatch` entries from [[IDA Cross-check#Full run (lead, 2026-10-16)]] have a classification and address-specific evidence. There are **248 pieces, 91 real function entries IDA merged, and 1 unclear invalid decode**. The 248 chunk corrections pass the existing fix checker. No correction was applied to the live map; Claude's independent review and application remain separate gates.

## Scope and provenance

The source queue and second-pass records are under `lab/native-map/run/ida-check/full-20261016/` (paths in this note are relative to the CartoonMMO workspace). W115 queried `lab/native-map/run/map.sqlite` read-only. No code, gameplay routes, content, saves, client, shared plan/ledger, or other worker output was changed. No IDA process or native client was launched.

| Input | SHA-256 |
|---|---|
| `labels.jsonl` | `0d84d3da6df5ee1e31e578ccab3ae8f151485f9eb49b41211ec914d2822c3130` |
| `review-queue.jsonl` | `51e55c24a5d9f3fbb2e25eb442c236ee7de4c44f6bbc033da9cbae5481c80e9e` |
| `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe` | `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` |

These are static client findings, not recovered server rules. Proposed map names remain descriptive annotations, not original symbols. W115 reviews entry boundaries; it does not independently re-audit every prior semantic description.

## Method

1. Selected exactly the queue's 340 high entries with the boundary-mismatch reason. Preserved each queue row, its `labels.jsonl` line number, full IDA row, Ghidra `fn` row, and accepted `label` before-image in `w115-evidence.jsonl`.
2. Checked the exporter before interpreting empty IDA fields. **All 340 mismatch rows have null pseudocode and empty caller/callee/string lists.** `scripts/native_map/ida_check.py:325-335` deliberately collects those fields only for exact starts. Empty lists therefore prove nothing about references. IDA's containing function start/end are still available. Of the queue's 255 distinct containing starts, 180 also have exact-start rows elsewhere in the supplied file; those rows are preserved in `w115-owners.jsonl`.
3. Read Ghidra facts through the same read-only SQLite tables used by `nmq show`/`callers`; inspected bytes through `nmq dis`/`bytes`. Used the existing Capstone environment called by `nmq`, with explicit PE section mapping. Ablation's own PE/address mapper was not used.
4. Scanned **every byte** of both initialized executable sections (`.text` and `BINK`) for `E8 rel32` targets equal to a queued address. A positive direct-call proof had to decode as a call from its preceding Ghidra function entry. Every real direct-call row records the exact instruction site and bytes. Every piece has **zero raw direct-call candidates**, so its absence result does not depend on a disassembler recovering the caller correctly.
5. Scanned every initialized PE section for absolute 32-bit pointers to the queued addresses, including unaligned and overlapping occurrences. This independently checks vtable absence instead of trusting missing `vslots`. Verified recorded vtable entries at **table base + 4 * slot index**, against the actual target DWORD. Two untyped tables were established through constructor stores. Two apparent address matches required separate instruction/text inspection below.
6. Recorded incoming `E9`, short/near conditional, and short unconditional branch sites with instruction bytes. Every piece has at least one verified branch into its exact entry and IDA's containing-function association. Decoding forward from an IDA start finds 217 aligned entries and one mid-instruction entry; the other 122 are noncontiguous IDA tails. A containing function's numeric start/end is not its complete chunk list.
7. Retained real direct-call/vtable entries. Traced the single address-taken fiber callback to its actual indirect call. Held the mid-instruction case for reanalysis. Marked the remaining jump-only blocks/tails as chunks; did not delete functions or force shared tails into an exclusive owner.

All per-address recovered facts have a `labels.jsonl` line locator, Ghidra address, and byte/reference receipt in `w115-classification.jsonl`, `w115-evidence.jsonl`, and `w115-xrefs.jsonl`. The TSV is a compact review index. `w115-worked-evidence.txt` contains 19 exact `nmq` commands and their output for the examples below.

## Counts

| Class | Count | Evidence / disposition |
|---|---:|---|
| Piece of a larger function | 248 | IDA containing start plus exact incoming branch; no direct-call candidate or vtable target; chunk fix staged |
| Real function IDA merged | 91 | 46 direct-call entries, 44 vtable entries, 1 callback with traced indirect call; retain map entry |
| Both wrong / unclear | 1 | `006ca9c3` is inside an instruction; hold for Ghidra reanalysis |
| **Total** | **340** | Exact queue coverage, no duplicates or omissions |

The real-function verdict establishes a separately invoked entry, including MSVC cleanup funclets. It does not imply that each entry is an ordinary source-language function.

## Ten worked examples

1. **File diagnostic cluster: pieces.** `008dddf1`, `008dde8a`, `008ddf2c`, and `008ddfb5` belong to IDA start `008ddc00` (end `008de128`; source lines 4438-4441). Incoming branches are respectively `008dddc0`, `008dde58`, `008ddefc`, and `008ddf85`. Their byte forms are short jumps, not calls. Each has zero direct-call candidates and zero absolute pointer references. Ghidra's apparent chain of function calls is a chain of blocks sharing the surrounding frame. All four receive chunk annotations.

2. **Player diagnostic continuation: piece.** `008de1ea` (source line 43285) is associated with IDA start `008de1e4`, end `008dff32`. `008de1b9` jumps directly to it. The bytes from the real prologue at `008de160` show `008de1e4` restoring ESI from `[ebp-0x14]`, followed by `008de1ea` updating the existing frame's exception state. There are no direct-call candidates or absolute pointer references to `008de1ea`. Here `chunk_of=008de1e4` records **IDA's association**, not a claim that the diagnostic routine's original prologue starts there.

3. **XML comment tail: piece.** `0073f770` (line 1626) belongs to IDA start `007414c0`. `007415de` contains `e98de1ffff`, a jump to `0073f770`. No direct-call candidate or pointer targets the label. The target is physically before IDA's start; this is evidence for a noncontiguous tail, not an invalid containment test based on numeric intervals.

4. **Parent-chain loop header: piece.** IDA associates `00409711` with `00409706` (line 26947). `00409706` contains `eb09`, jumping over the loop body to its condition at `00409711`. `00409713` branches back to `00409708`. No direct-call candidate or pointer targets `00409711`. The separate map function is a loop-entry block.

5. **Shared delay-load resolver: piece, retained as shared chunk.** IDA assigns `007cf79f` to `007cf795` (line 40672). Forty verified stub jumps reach `007cf79f`, starting at `007cf79a` and `007cf7bb`. There are no direct-call candidates or absolute pointers to the resolver entry. It remains a separately annotated shared tail; folding its behavior exclusively into the first stub would lose the other users. The staged fix keeps its prior description and all incoming branch sites.

6. **Action-bar tail and false pointer: piece.** `00a12840` belongs to IDA start `008a7e40` (line 6790); `008a7e57` jumps there. There are no direct-call candidates or vtable slots. The sole raw address-like occurrence, at `00b775a4`, straddles the displacement of the SSE instruction at `00b775a1` and the next instruction at `00b775a6`. It is not an address operand. Its rejection is recorded in the pointer receipt rather than silently discarded.

7. **Deleting destructors: real vtable entries.** IDA merges `00804900` into the this-adjusting thunk at `008048f0` (line 2659), but DWORD `017f19fc` equals `00804900`; the primary vtable calls the destructor independently of the thunk. The untyped cases `0068e593` and `0068e59b` (lines 37964-37965) are likewise retained: table entries `017bcbe0` and `017bccf8` point to them. Constructor stores at `0068e618` and `0068e60b` install those table addresses at object offsets `+0x24` and `+0`. Missing Ghidra `vslots` did not make these pieces. Nonzero-slot cases `00d57080` and `00e959f0` were verified at `01840ac4` (slot 14) and `01861600` (slot 1), not at their table bases.

8. **CRT cleanup helper: real directly invoked entry.** IDA includes `00d83198` in `00d830cb` (line 51615). `00d83174` contains `e81f000000`, a direct call to the exact helper address. The local parent-to-cleanup relationship does not erase its call entry. All 46 direct-call verdicts have equivalent site/byte evidence.

9. **Fiber body: real indirectly called entry.** IDA associates `00f46242` with `00f461af` (line 55225). At `00f461f2`, `684262f400` pushes its address; `00f461fa` calls `00f46e19`. That function stores its first argument in the new fiber record at `00f46e63`, passes the record to trampoline `00f46daa`, and the trampoline calls `[esi]` at **`00f46dda` (`ff16`)**, with ESI holding that record. The callback is genuinely called at its labelled entry even though it has no direct-call or vtable edge. It is retained, not folded.

10. **Cmap decode overlap: unclear; no fix.** `006ca9c3` (line 38474) is inside `006ca9c1: 8a68fc`, decoded from IDA start `006ca9b0`; the instruction ends at `006ca9c4`. Thus the labelled address is not an instruction-entry block. Its apparent pointers at `01868268`, `01868623`, and `0186862b` are the byte sequence `c3a96c00` in UTF-8 locale words, not vtable references. IDA may have the correct outer function; W115 does not call both tools wrong. Reanalysis from `006ca9b0`, verification of the function end, and retirement of the invalid Ghidra entry require lead review. No piece correction was fabricated for it.

## Staged correction format

`lab/native-map/run/ida-check/full-20261016/w115-fixes/B0001/` contains:

- `sample.txt`: the 248 unique piece addresses.
- `records.jsonl`: exact complete accepted-record before-images, including their existing audits.
- `review.jsonl`: one `refine` verdict with a complete `fix` object for each sampled address.

The three files match the inputs consumed by `scripts/native_map/recheck.py:70` (`apply_fixes`). The parent `manifest.json` describes the batch. `B0001` is a **staging name**, not a reserved live recheck batch. A lead application must use an unused live batch directory; the existing atomic before-image comparison still applies. No claim/done markers or live-map changes were created.

`validate.FIX_VERDICTS` has no `chunk` verdict and its kind enum has no chunk kind. Each fix therefore uses `verdict=refine`, `kind=function`, a visible `_Chunk` name, `chunk_of`, and `boundary_review` metadata. Existing block descriptions/evidence are retained, explicitly labelled as prior semantics not re-audited here. Confidence is `likely`, matching the fix checker's independent-review cap. Extra metadata survives the existing apply path. No tooling change was added to consume or count these annotations differently.

## Gates and commands

The main analysis ran as bounded `python -I -` / `python -I -B -` inline jobs using the standard library, plus the existing `lab/tools/ablation-env/Scripts/python.exe -I -` for Capstone decoding. No new script or dependency was installed. Existing `validate.check_fix` was called for each verdict with its actual owning shard. The full gate output is `w115-validation.txt`:

```text
classification_total=340 pieces=248 real=91 unclear=1
real_evidence direct_call=46 vtable=44 indirect_callback_call=1
piece_entry_proofs=248 missing_ida_owner=0 missing_branch_proof=0 direct_call_candidates=0 vtable_targets=0
fixes_ok=248 fixes_invalid=0 fixes_missing=0
fix_checker_canary=PASS (fix.addr differs from verdict addr)
recheck_batch_shape=PASS sample=248 records=248 reviews=248 duplicate_missing_extra=0
scoped_live_labels_unchanged=340/340 live_map_fixes_applied=0
```

The checker can be rerun from `project/` without applying anything:

```powershell
@'
import json, sqlite3, sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path('scripts/native_map').resolve()))
from validate import check_fix
db_path = '../lab/native-map/run/map.sqlite'
db = sqlite3.connect('file:' + db_path + '?mode=ro', uri=True)
path = Path('../lab/native-map/run/ida-check/full-20261016/w115-fixes/B0001/review.jsonl')
rows = [json.loads(line) for line in path.read_text(encoding='utf-8').splitlines()]
bad = missing = 0
for row in rows:
    shard = db.execute('SELECT shard FROM label WHERE addr=?', (row['addr'],)).fetchone()[0]
    if 'fix' not in row:
        missing += 1
        continue
    errors, _ = check_fix(db_path, shard, row)
    bad += bool(errors)
    if errors:
        print(row['addr'], errors)
print(f'fixes_ok={len(rows)-bad-missing} fixes_invalid={bad} fixes_missing={missing}')
assert not bad and not missing and len(rows) == 248
'@ | python -I -B -
```

Selected inspection commands, with complete outputs in `w115-worked-evidence.txt`, include `nmq.py show 0068e5b7 00f46e19 00f46daa`, `nmq.py dis 006ca9b0 72`, `nmq.py dis 00d830cb 224`, and `nmq.py callers 008dddf1`. All 19 recorded commands exited 0. The gate also rejects a deliberately mismatched `fix.addr`, checks all sample/snapshot/verdict address sets, and compares all 340 current accepted records with their preserved before-images.

## Limits and handoff

- Static absence refers to the pinned PE's direct-call encodings and stored absolute pointers. Arbitrarily computed runtime targets are not disproved. No dynamic or IDA UI session was available or needed for the staged review.
- IDA's mismatch rows omit pseudocode/xrefs by exporter design; absent fields were never used as negative evidence. Exact owner rows cover 180 of 255 starts. No fresh IDA chunk export was produced.
- `chunk_of` records IDA's association, including resumed and shared tails; it does not establish exclusive source-language ownership. The correction preserves block annotations and leaves function ranges, graph edges, and consumers unchanged.
- `006ca9c3` remains held for reanalysis. The other 91 retained entries are boundary decisions, not renewed verification of all their semantic names.
- Independent lead review and live application are outstanding by task design. No `recheck.py done` command ran. Shared Build Plan, Build Ledger, MODLOG, and Tasks updates remain with the lead under W115's ownership limits.

Ponytail: reused `nmq`, the existing PE/Capstone environment, accepted evidence, `check_fix`, and the recheck batch format. No replacement disassembler, review framework, dependencies, code changes, or live-map mutation was built.
