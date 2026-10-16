---
type: review
task: W119
date: 2026-10-16
---

# IDA String Review

**Review gates passed; staged only.** All **941** medium-severity `required_strings_missing` entries from [[IDA Cross-check]] have an address-specific classification: **837 method differences, 102 wrong string citations, 2 wrong semantic descriptions/names, 0 unclear**. The **104** staged corrections pass `validate.check_fix`. No live label was changed and no `recheck.py done` command ran. Independent lead review and application remain separate gates.

The two semantic cases concern the description/name at the existing entry, rather than a proven relocation of an entire function. `006b9730` scans a sequential descriptor array, rather than following next-entry links. `016c2365` registers an identifier without the question mark asserted by its old name. No replacement address for the nonexistent linked-list traversal or question-prefixed registration was invented.

## Scope and provenance

Paths below are relative to the CartoonMMO workspace. Input and output artifacts are under `lab/native-map/run/ida-check/full-20261016/`. Full decompiler output and disassembly remain in the lab. This note contains observations from the preserved client, not recovered server rules or gameplay acceptance.

| Input | SHA-256 |
|---|---|
| `labels.jsonl` | `0d84d3da6df5ee1e31e578ccab3ae8f151485f9eb49b41211ec914d2822c3130` |
| `review-queue.jsonl` | `51e55c24a5d9f3fbb2e25eb442c236ee7de4c44f6bbc033da9cbae5481c80e9e` |
| `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe` | `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` |

The map was queried with SQLite `mode=ro`. The final comparison found all **941** accepted records equal to their initial before-images, including their audit metadata. Other workers' paths, shared code, the map database, saves, binaries, and shared milestone notes were not edited. Neither IDA nor the native game client was launched.

`w119-evidence.jsonl` preserves each queue row, its exact `labels.jsonl` line number and complete IDA export row, the Ghidra `fn` row, and the accepted `label` before-image. `w119-classification.jsonl` classifies every address and every missing citation, with artifact/line receipts. Its TSV companion is the compact address index. `w119-artifacts.json` lists the complete output set and hashes.

## Method

1. Selected exactly the 941 medium queue rows. These contain **1,198** missing string citations. A label can have several citations and several resolution mechanisms.
2. Reused the [[IDA Boundary Review]] staging method and `nmq`'s read-only Ghidra facts. Read `ida_check.py` before interpreting its output: `_referenced_strings` follows `DataRefsFrom` for function items and requests **`STRTYPE_C` only**. It neither follows stored pointers/tables nor recursively inspects callees. An empty string list therefore does not establish absence of a string-dependent operation.
3. Decoded the pinned PE with the already-installed Capstone environment used by `nmq dis`. Used explicit PE section-to-file mapping, including `.rwdata`. Recorded **95,191** instructions across the 941 IDA numeric function ranges. Range decoding alone was not treated as coverage of switch successors outside those ranges.
4. For each positive direct/stored-pointer receipt, recorded the instruction address and bytes, string VA, encoding and decoded bytes; for a stored pointer, also recorded its slot and DWORD. Accepted only printable text with ordinary whitespace. Rejected numeric operations and accidental float/string overlap as string-use proof. Only `reference_proofs` in the evidence artifact are accepted positive receipts.
5. For indexed tables, checked the function's actual table access and stride, the relevant element slot, pointer bytes, and literal bytes. For the two truncated switch bodies, verified the branch/table path to the case block. For the SSL thunk, followed both tail jumps to the diagnostic-emitting body. These receipts are in `w119-table-evidence.jsonl` and `w119-controlflow-evidence.jsonl`.
6. Separated literal spelling errors from symbol/type errors. Ghidra FID comments, import names, RTTI/vtable names, and data-symbol names are not runtime literals. Removed FID-as-string evidence or retagged the verified reference as `code`/`vtable`. Corrupt literal spellings were replaced only after verifying the actual operand/pointer and literal bytes. Details are in `w119-symbol-evidence.jsonl` and `w119-citation-evidence.jsonl`.
7. Inspected the unrelated-string cases separately. `w119-neighbour-evidence.jsonl` preserves the source bytes and the true neighbouring/caller facts, where found. No missing string alone triggered a rename.
8. Built complete correction records in the existing recheck batch format. Called `validate.check_fix(db, actual_owning_shard, verdict)` for each correction. Its confidence cap means a corrected record is at most `likely`, even when its before-image was `confirmed`.

The 837 retained labels resolve this string disagreement; they are not a renewed audit of every prior semantic assertion. Proposed names, including the corrected registrar name, remain descriptive annotations rather than recovered original symbols. No community source code was used.

## Counts

| Classification | Functions | Disposition |
|---|---:|---|
| Method difference | 837 | Retain label; positive byte/reference path recorded |
| Label cites the wrong string | 102 | Change evidence only, plus the fix checker's confidence cap |
| Label describes the wrong function | 2 | Correct the proven description/name at the same entry; retain conservative confidence |
| Unclear | 0 | No unresolved queue classification |
| **Total** | **941** | Unique, exact queue coverage |

Resolution counts below are **citations**, not functions:

| Resolution | Citations |
|---|---:|
| Direct UTF-16 literal | 468 |
| Stored pointer to narrow literal | 437 |
| Stored pointer to UTF-16 literal | 3 |
| Direct narrow literal, including citation representation differences | 151 |
| Indexed table element | 17 |
| Switch successor omitted from the short IDA function range | 2 |
| Tail-jump chain | 1 |
| Corrupt literal spelling corrected | 42 |
| Symbol/FID citation corrected | 72 |
| Unrelated/non-string data citation corrected | 5 |
| **Total** | **1,198** |

The 72 symbol/FID citations comprise 8 FID annotations, 15 imports, 4 named tail targets, 22 vtable symbols, 20 RTTI symbols, and 3 data symbols. Multiple citations can belong to one correction.

## Ten worked examples

1. **UTF-16: `00405d3d`, retained.** IDA export line **37** reports the exact function start but sees only `G`. At `00405db7`, the instruction pushes `0175c640`, whose UTF-16 bytes contain the complete child-add warning cited by the label. Ghidra also shows the wide literal passed to `004e889c`. The checker requested the wrong string encoding.

2. **Stored filename pointer: `007989a0`, retained.** IDA line **2098** omits `EquipmentSlotDefinitions.txt`. `00798a14` loads the DWORD at `01b2f27c`; that slot contains `017eda78`, the narrow filename. The Ghidra call uses `PTR_s_EquipmentSlotDefinitions_txt_01b2f27c`. A scan of only the instruction's immediate data target reads pointer bytes rather than the filename.

3. **Citation representation: `0051d634`, retained.** IDA line **511** disagrees with citations `0177061c=Action` and `0177062c=ActionList`. Those are address-qualified Ghidra-export representations. Pushes at `0051d676` and `0051d651` reference the respective UTF-16 literals. The address prefix is a locator, not part of either literal. Conventional escaped newline/tab/path citations are treated similarly only when the decoded bytes match; arbitrary prefix removal is not accepted.

4. **Registration and keyword tables: `00c269c0` and `00e42ff0`, retained.** The former pushes Lua registration table `0182f028` at `00c269cc`; its first pointer is `0182f000`, containing `getSize`. Its `ImageControl` name is reached through `01b6c300`. The latter walks 21 keyword pointers at `01854430`; slot `01854434` points to `01854420`, containing `break`. Both operations use the cited strings through a table, even though IDA's direct C-string list omits them.

5. **Tail-jump diagnostic: `00f5fa90`, retained.** `00f5fa93: e988270300` jumps to `00f92220`; `00f92222: e919ffffff` jumps to `00f92140`. At `00f92149`, that body pushes `01879438`, the `SslConnectionFilterImpl::Send, not connected` diagnostic, before calling `00f99fd0`. The wrapper label correctly describes forwarding to the send path. The string is not local to its eight-byte entry.

6. **Switch successors: `00fc1740` and `01450c60`, retained.** IDA ends the former at `00fc1752`, just after `00fc174b`'s indirect jump. Case 4 uses slot `00fc1810`, pointing to `00fc17c4`; `00fc17e8` pushes `0187b77c` (`%f,%f,%f,%f`). For `01450c60`, IDA ends at `01450e99`, but an entry-range branch at `01450e1e` reaches continuation `01450eb0`. The switch at `01450ff9` uses remap `0145272b` for state `0x13`, then target slot `01452704`, pointing to `01451b5d`. `01451b61` pushes the missing state diagnostic at `018ecfa8`. Ghidra's larger control-flow body accounts for both strings.

7. **Symbols are not literals: evidence corrections.** `004ec464` calls through import slots `017510cc`/`017510c8` for `CreateFileA`/`CreateFileW`; the names belong in call evidence. `00a03270` pushes RTTI descriptors `01b4af6c`/`01b34a78`, not literal strings named `Class::RTTI_Type_Descriptor`. `007c5190` stores vtable VA `0175b24c` at `007c51c5`; that citation is retagged `vtable`. The `Library Function - Single Match` text at `0041f653` is a Ghidra comment. It is removed as string evidence; the existing body/callee evidence remains. All four IDA rows and instruction receipts are preserved.

8. **Corrupt prefix versus wrong name.** At `00450840`, the pushed `01764058` contains `GFontDeviceFT2`, matching IDA; it does not contain `.GFontDeviceFT2`. This receives an evidence-only correction. At `016c2371`, the registrar stores `0176ab48`, containing `GFC_Drivers_Draw_FontDeviceWin32`. Its old `QuestionPrefixed` name and its explicit `?` identifier assertion are false. That semantic correction removes the prefix claim from name, description and purpose at the same address. No question-prefixed registration was found.

9. **Numeric data mistaken for string evidence.** `00e31b80` compares a masked value against numeric `0x01800000`; it does not consume a Curve diagnostic. The actual diagnostic starts at `017ffff4` and is used by `008845e0`, `00885680`, and `00886420`. Its interior happens to overlap the numeric value interpreted as a VA. Only that evidence item is removed. Likewise `011f69c0` compares its DES rotation-table cursor with `018b8c6c`, inside an adjacent version banner. The banner is not embedded/emitted by the key-schedule operation; the false string item is removed and its existing schedule-code evidence retained.

10. **Wrong traversal description: `006b9730`.** Ghidra shows `param_1 = param_1 + 2`; IDA shows `v3 += 2`. Both load the next sequential name and return the matching entry's second word. This is a null-terminated array of two-DWORD descriptors, not traversal through a stored next-link pointer. The correction preserves the neutral function name, rewrites the mechanics/purpose, and replaces the unrelated `sfnt-table`/`postscript-font-name` citations with the actual array operations. True `sfnt-table` use is at `006ba3c0`. Separately, `/rsrc` belongs to path-copy helper `006be930`; the format selector `006be0f0` chooses `cff`/`truetype` instead, so its evidence alone is corrected.

## False-positive patterns and checker improvements

- **Encoding:** query IDA's actual string type and support UTF-16. A single first letter is not the complete wide literal.
- **Indirection:** distinguish address-of-literal from pointer storage, registration tables, and indexed table members. Record the slot and target instead of blindly decoding pointer bytes as text.
- **Control flow:** account for proven tail jumps and switch successors outside the short IDA range. A recursive traversal needs cycle/depth bounds and should report its path; absent graph edges are not proof of absence.
- **Representation:** separate an exported `address=text` locator from literal text. Decode conventional escapes once, with a parser that consumes escaped backslashes together; replacing `\r` inside a Windows path independently corrupts it.
- **Evidence typing:** FID comments, import symbols, RTTI/vtable names and data labels require their own evidence types. `validate.py` currently accepts a `string` appearing anywhere in Ghidra C, including comments/symbols; fix validation alone therefore does not prove literal use.
- **Data/code confusion:** do not promote every numeric operand equal to a readable VA into a string reference. Verify use and encoding. The `0x01800000` mask and the DES loop-end pointer are counterexamples.
- **Overlapping data:** Ghidra's exported string text can include bytes from an adjacent scalar or inferred symbol spelling. At `0091e670`, the float load from `01805960` must not prove `3@RenderDistance`; the actual string operand at `0091e68a` uses `01805964`, containing `RenderDistance`. The review's canary also rejects the erroneous `<ShadowMap` citation rather than accepting adjacent numeric bytes.

These are recommendations only. No checker or source code was edited.

## Staged correction format

`w119-fixes/B0001/` contains `sample.txt`, `records.jsonl`, and `review.jsonl`, each covering the same **104** addresses. `records.jsonl` holds complete accepted-record before-images. Every review row is `verdict=refine` with a complete `fix` object and map/IDA receipt locators. The parent `manifest.json` records counts and input pins.

`B0001` is a private staging name, not a reserved live recheck batch. Lead application requires a fresh live batch and the existing atomic before-image comparison. No claim/done markers or live recheck files were created. The existing application path preserves old audits and queues independent fix checks; no substitute apply path was built.

**Evidence-only scope is enforced:** all 102 citation-only records differ from their before-images only in `evidence` and, where necessary, the `likely` confidence cap. Six old prose fields also repeat garbled spellings and are explicitly held for lead review in `w119-prose-followups.jsonl`: `00450832`, `0091e670`, `00a720f0`, `00ce2320`, `008549c0`, and `013dd700`. The staged evidence correction does not renew those old prose assertions. This preserves the task's instruction to change only evidence for that class.

## Gates and commands

`w119-validation.txt` records the final gates:

```text
classification_total=941 method_difference=837 wrong_string=102 wrong_function=2 unclear=0
fixes_ok=104 fixes_invalid=0 fixes_missing=0
fix_checker_canary=PASS (fix.addr differs from verdict addr)
evidence_only_fixes=102 semantic_fixes=2 field_scope=PASS
citation_receipts=1198 direct_or_pointer=1059 indexed_table=17 controlflow=3 corrections=119 coverage=PASS
false_positive_canaries=PASS (float-adjacent prefix and numeric-address collision rejected)
scoped_live_labels_unchanged=941/941 live_map_fixes_applied=0
recheck_batch_shape=PASS sample=104 records=104 reviews=104 duplicate_missing_extra=0
recorded_nmq_commands=33 exit_nonzero=0
additional_canary_shadowmap_prefix=PASS
review_note=PASS worked_examples=10 receipt_locators=1198
```

Analysis used bounded `python -I -`/`python -I -B -` inline jobs with the standard library, plus the existing `lab/tools/ablation-env/Scripts/python.exe -I -` for Capstone. No script or dependency was installed. `w119-worked-evidence.txt` contains all 33 retained, aligned `nmq show`, `dis`, `callers`, and `bytes` commands with their output and exit codes.

The existing fix checker can be rerun from `project/` without applying corrections:

```powershell
@'
import json, sqlite3, sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path('scripts/native_map').resolve()))
from validate import check_fix
p = Path('../lab/native-map/run/ida-check/full-20261016')
db_path = '../lab/native-map/run/map.sqlite'
db = sqlite3.connect('file:' + db_path + '?mode=ro', uri=True)
rows = [json.loads(x) for x in
        (p/'w119-fixes/B0001/review.jsonl').read_text(encoding='utf-8').splitlines()]
bad = missing = 0
for row in rows:
    if 'fix' not in row:
        missing += 1
        continue
    shard = db.execute('SELECT shard FROM label WHERE addr=?', (row['addr'],)).fetchone()[0]
    errors, _ = check_fix(db_path, shard, row)
    bad += bool(errors)
    if errors:
        print(row['addr'], errors)
print(f'fixes_ok={len(rows)-bad-missing} fixes_invalid={bad} fixes_missing={missing}')
assert not bad and not missing and len(rows) == 104
'@ | python -I -B -
```

## Limits and handoff

- The supplied IDA export is the IDA evidence. No fresh IDA analysis or runtime test was available or claimed. Hex-Rays can be absent or truncated; missing pseudocode was not treated as negative semantic evidence.
- Positive receipts establish the disputed literal/reference path, not all historical claims in a retained label. Inferred C types and names still require independent judgment.
- Six prose repetitions remain explicitly flagged under the evidence-only ownership rule above. The Curve type implication in the old `00e31b80` name is also not established by the discarded diagnostic evidence; W119 does not invent a replacement type.
- Lead review, live application, and the normal independent fix checks remain outstanding by design. Shared Build Plan, Build Ledger, MODLOG, and Tasks updates remain with the lead under W119's ownership limits.

Ponytail: reused `nmq`, the pinned PE and installed Capstone environment, W115's batch layout, and `validate.check_fix`. No replacement disassembler, review framework, dependencies, code changes, or live-map writer was built.
