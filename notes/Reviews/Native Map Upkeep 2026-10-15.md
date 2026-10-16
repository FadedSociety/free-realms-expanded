# Native Map Upkeep 2026-10-15 — W87

**Verdict: PARTLY PROVEN.** All six stuck shards were cleared using the dispatcher's completion gates, with no workers launched. The live map received 23 scoped label changes: **8 upgraded, 14 corrected at unchanged confidence, 1 downgraded**. Another 57 validated corrections remain staged because whole-shard acceptance fails existing evidence checks or changes unrelated audit metadata. The broad consistency gate does **not** pass; do not mark W87 complete in the Build Plan or Ledger.

## Scope and inputs

Worker ownership was native-map state/outputs and this review note. No Rust, content, saves, shared plan/ledger, launcher, release binary, client UI, commits, installs, or external messages were changed by W87. Other workers' checkout changes were observed and preserved.

Read: AGENTS, MMO Worker Contract, Build Plan, Build Ledger, Native Map index, Build Guides index and the Character/Items/Combat guides, AddPc Layout, Native Combat Controls, Inventory Catalog Framing Compatibility, latest Native Round5 review, RTK, Ponytail (full), Caveman, reverse-engineering, codebase-memory, secure-code-guardian and security-reviewer. Native facts came from the executable and map, not Legacy source. No C# source was copied.

- Executable: `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`.
- SHA-256: `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` (recomputed).
- Database: `lab/native-map/run/map.sqlite`; `fn` and `label` tables, queried read-only during discovery.
- Private receipt root: `lab/native-map/run/upkeep-W87/`.
- `targets.txt`: 130 speculative/unknown labels selected from `net.packet.player_update`, `net.packet.ability`, `game.ability`, `game.actionbar`, `ui.actionbar`, `game.item`, and explicit build dependencies. `labels-before.json` preserves their original records and the already-labelled anchors.
- `evidence/<address>.dis.txt`, `.json`, `.txt`: 151 function entries, with nmq disassembly, original export metadata and pre-upkeep labels. These are inspection spans, not a claim of complete control-flow recovery. Dispatcher-specific jump-table cases and additional remove/position/item spans are separate receipts.
- Proposed names describe recovered mechanics; they are not original source symbols. Reviewer fixes are capped at **likely**, following `validate.check_fix`. No original server rule or numerical balance was recovered here. Any server choice beyond these client mechanics remains reconstructed policy (R).

The codebase graph was used for pipeline structure, project `cartoonmmo`, generation `2026-10-16T01:56:50Z`. Coverage reported no recorded issue for dispatch/recheck/xreview/validate. The graph omitted Dispatcher methods from the function query; source reads were the authority for those methods. Current gameplay/gateway routing and the ledger were consulted; W87 did not infer missing gameplay from old map notes.

## Six stuck shards

The live database initially matched the October 14 status snapshot: two `mapping`, four `reviewing`. Recovery constructed the pipeline's review-only dispatcher with `xreview.make(run)` and invoked `finish_map`/`finish_review` at each recorded latest attempt. Its `no_launch` path returned incomplete work to the normal queue. No SQL state patch, process adoption, process termination, or model launch was used.

| Shard | Before | Final | Gate result |
|---|---|---|---|
| S0079 | reviewing | review_pending | 0/40 verdicts; pipeline requeued review |
| S0182 | reviewing | review_pending | 0/40 verdicts; pipeline requeued review |
| S0212 | reviewing | review_pending | 0/40 verdicts; pipeline requeued review |
| S0215 | mapping | pending | incomplete mapper output; pipeline requeued map |
| S0618 | reviewing | accepted | completed 55/55 review; three existing valid reviewer fixes; 0 uncured disputes |
| S0672 | mapping | pending | incomplete mapper output; pipeline requeued map |

The PID recorded for S0672 (`22920`) was reused by a newer Codex process: its live start date was October 15, while the stuck dispatch record was from October 14. W87 did not adopt or stop it. The completion-only recovery avoided the dispatcher's PID-only adoption risk. CIM command-line inventory was denied; `Get-Process` provided the start-time observation. This is an upkeep-tool limitation, not evidence about the current unrelated process's work.

S0618's 55 newly accepted records are separate from the 23 W87 label changes below. It contains earlier reviewers' work, not new W87 function interpretation. Final state: 825 accepted, 5 failed, 63 pending, 4 queued, 4 review_pending; zero mapping/reviewing. Total labels: 99,188. `status.json` was refreshed through `Dispatcher.status`.

## Applied label changes

Acceptance used `validate` and `check_fixes_file`, then the existing `Dispatcher.accept` method. A cloned database rehearsed each change first. Nine shards were allowed only after their mapper outputs passed and the rehearsal changed exactly the selected labels. Live acceptance ran in one SQLite transaction with preflight-input checks and a before-apply backup. The final comparison of **all** label rows found 23 changes and zero unexpected differences, including audit metadata.

| Addresses | Change | Byte provenance |
|---|---|---|
| `008d6830` | unknown → likely; `ReadSignedI16HeaderPair` | `movsx` at `008d6853`, `008d687d`; destinations +4/+8, reader error/clamp, `ret 4` |
| `008fde60`, `008fded0` | unknown → likely; rental packet constructor/destructor | member receivers +0xc and +0x1b4 at `008fde97`, `008fdeaa`, `008fdf00`, `008fdf10`; cookie values are bookkeeping |
| `0091d000` | unknown → likely; AddPc constructor | `0091d02b` base subtype 1; five IStrings at +0xe0/+0xf0/+0x100/+0x110/+0x120, not six; title member +0x148 |
| `0091d4d0` | unknown → likely; customization packet constructor | `0091d50b` stores List vtable at +0xc, not +0x10 |
| `00925380` | speculative → likely; `ParseAddPcSpan` | calls `0091aa10`, AL result, plain `ret`, caller stack cleanup; bounds/allow-trailing gates |
| `00965f90` | unknown → likely; profile swap attachment-group creation | `00965fc2` loads the manager pointer from +0x58; `ret 0xc`; nine pushes before `00966051`, ninth value is `007c4710` result |
| `00bf7240` | unknown → likely; check/repair attachment slots | empty-list success bypass at `00bf72a6..00bf72c6`; clears +0x67 at `00bf7423`; virtual +0x40 repair branch at `00bf7486` |
| `0072dcc0` | speculative → unknown | constructor invokes `0072d420`, then writes the keyed node's +0x28; the earlier ItemStatDefinition class binding was disputed and is withdrawn |
| `0072e050` | unknown retained; corrected byte observations | four text-string sites, including destination +0x234; old claim of two strings is withdrawn |
| `00908a50`, `00908ac0`, `00908b30`, `00908ba0`, `00908c10`, `00908c80`, `00908cf0`, `00908d60`, `00908dd0`, `00908e40` | unknown retained; corrected byte observations | each wrapper ends in plain `ret`, not `ret 16`; earlier stdcall prose was withdrawn. Semantic packet names remain unresolved |
| `00bf6e10` | unknown retained; corrected byte observations | name accessors feed identifier lookups, not a simple string-equality test; old detailed matching prose was withdrawn |
| `0103e130`, `0103e210` | unknown retained; corrected byte observations | first clears +0x28 after free; second's time calculation is not simply source-time minus now. Detailed rejected semantics were withdrawn |

Exact before/after confidence, accepted name and classification for each address: `applied.json`. Full reviewed records: the nine owning shards' `W87-review.jsonl`. The pipeline also queued its normal independent fix checks. **Likely** is not a claim that Claude's lead review has occurred.

## Facts for W82, W84 and W85

### W82: job outfits and profile selection

- `00965f90` consumes three stack words; its attachment-group constructor `00964ba0` receives **nine**, not eight. The ninth is the resolved `007c4710` result. The old label's call signature omitted it.
- `00bf7240` can repair missing attachment slots and clears its +0x67 status byte; it is not log-only. Empty groups have a guarded success bypass. Its own final true result does not by itself prove every required slot existed.
- The already-confirmed `00bfb620` profile-swap path independently scans required slots through `007972d0`/`007972f0`, resolves each key through `0072f530`, and on a miss logs the missing required slot, calls `00730380` and returns (`00bfb767..00bfb78b`). Required-slot completeness still matters for completing a profile swap.
- `00c906f0` is a constructor receiving its datasource name from the caller; it does not select the literal `BaseClient.Profiles` itself. Its corrected record is staged because S0406's whole-shard gate fails.
- No Kart Driver kit item IDs, historical starter equipment, or server unlock policy were recovered by W87. Outfit IDs still require W82's data evidence. Header/profile/attachment mechanics do not establish historical starter kits.

### W84: item catalogue

- **35/34 envelope correction:** both request writer `00a3da40` and reply reader `008d7980` have `u16 family, u16 subtype, i32 item-id word, i32 payload-byte-length, payload`. The current Items guide's single-definition paragraph omits the item-ID word. `008d79c3` stores it at packet +0x14 before `008d79d1` calls `008bfc00`; `00a3da50..00a3da86` writes the value from +0x14, followed by `0090add0` at `00a3da9b`. It is not a string pointer.
- Dispatcher case 34 starts at `00933501`. It constructs the request class, parses through `008f8180`, constructs `ClientItemDefinition` through `0090b720`, reads through `00919e10` and registers through `00a3c290`. The payload contains **one definition**, unlike the count-prefixed bulk 35/37 payload.
- `00919e10` passes the definition receiver in ECX to `00bae660`; its apparent argument-free pseudocode call is misleading. Four additional dwords land at +0xf0/+0xf4/+0xf8/+0xfc. Corrected reader record is staged because S0241's whole-shard gate fails.
- `00724300` constructs its stat container at **+0xd4**, not +0x80; corrected record staged because S0150 fails. Three-byte helpers `00724880`/`007249c0` are `mov eax,ecx; ret`, returning the receiver, not a stack argument; their corrections are staged too.
- `00a3df00` returns the cached row index/value, including -1 after requesting a miss. It does not return an item-definition pointer. `00c96360` returns node+4 or -1. Corrections are staged in S0295/S0407.

### W85: basic attacks and abilities

- Rechecked the already-confirmed dispatchers without changing them: `008a7eb0` selects modes 1–4; `008a7e40` routes default/job to `009cf430`, mode 2 to `00a12840` when the player exists, mode 3 to `009aaeb0`, mode 4 to `009b6590`.
- Raw comparisons in `008a5850` accept types **3, 6–9, 14–16, 22–23, 1009** when a current game exists. This is a client input-routing predicate, not job-specific attack balance.
- `00a36a70` always resets existing arming state before returning for a matching slot/bar pair: repeated selection toggles cancellation. Job-bar selectors are copied from +0x10/+0x14/+0x18 and flag +0x2c; item/minigame nodes use +0x18/+0x1c/+0x20 and flag +0x50. Corrected record is staged because re-accepting S0294 replaces the lead's unrelated `00a382b0` audit metadata.
- `00a35860` reads the **second stack argument** as its four-word position pointer and consumes 20 stack bytes; decompiler parameter numbering is not reliable. Staged with S0294.
- `00a309c0` returns an AL boolean. Its empty-list path starts with `xor al,al`; it does not return a dangling/uninitialized pointer, despite both an old label and an old reviewer interpretation. Staged in S0293.
- No Ninja/Warrior/Wizard ability IDs, damage coefficients or starter weapon IDs were established here. Server rules remain (R).

### Player updates: AddPc, remove and position

Raw jump table `00934618`, indexed by subtype minus one (`0092f530..0092f540`), resolves case 1 to `0092fec3`, case 3 to `00930714`, case 34 to `00933501`, case 37 to `009335e3`. The AddPc case calls constructor `0091d000` and span parser `00925380`. Its direction-to-yaw consumption remains consistent with the accepted AddPc Layout note.

Remove reader `008d69f0` sign-extends the variant word into object +0xc (`008d6a1c`) and reads the GUID into +0x10/+0x14 (`008d6a48`, `008d6a4e`). Constructor `008b38a0`'s +0xc argument is not the GUID. Position uses **outer opcode 125**, not the 35/7 style case: constructor `008b3910`, incoming parser `0090de90`, outgoing change comparison `008b39a0`. The comparison includes both +0x30/+0x31 bytes. Their corrected records are staged because S0222's mapper gate fails. Do not infer wire direction from a constructor alone.

These are static client contracts. The current Build Ledger already records native AddPc visibility; W87 did not rebuild it or perform a native run.

## Checks and blocker

Commands below ran from `project`. The capture and acceptance commands are one-shot: their guards now prevent overwriting before-evidence or replaying acceptance. `verify.py` is the repeatable read-only gate and calls the installed pipeline validators; no alternate validator was introduced.

```powershell
python -I '..\lab\native-map\run\upkeep-W87\collect.py'
# targets=130 anchors_and_targets=151 disassembly_failures=0
python -I '..\lab\native-map\run\upkeep-W87\review.py'
# verdicts=130 fixes=80 shards=39
python -I '..\lab\native-map\run\upkeep-W87\apply_checked.py'
# canary_invalid_call_rejected=1
# APPLIED {"validated_records":1186,"changed":23,"unexpected":0,
#          "counts":{"downgraded":1,"corrected_same_confidence":14,"upgraded":8}}
python -I '..\lab\native-map\run\upkeep-W87\verify.py'
```

Final focused output, exit **0**:

```text
fixes_ok=80 fixes_invalid=0 fixes_missing=0
live_changed_records=23 errors=0
mapper_shards=10 valid_records=1241 errors=0 bad_shards=0
sqlite_quick_check=ok
preservation_changed=23 unexpected=0
S0079=review_pending
S0182=review_pending
S0212=review_pending
S0215=pending
S0618=accepted
S0672=pending
```

Broad check, `python -I '..\lab\native-map\run\upkeep-W87\verify.py' --all-mappers`, exits **1**:

```text
mapper_shards=39 valid_records=5515 errors=261 bad_shards=21
```

The broader failures are in unchanged mapper outputs, not invalid W87 fix records. They include stale pseudocode excerpts and vtable-slot anchors. Examples: S0150's `007240e0` and `00725140` cite `if ((param_2 & 1) == 0)` absent from the current export; S0241 has old destructor-code anchors and a `BaseClient::vftable[1]` anchor; S0406 has stale datasource vtable slots. Exact errors for all 21 shards are in `mapper-consistency.json`. They were not silently removed or assigned replacement confidence.

Even the initially clean 14-shard rehearsal would alter nine unrelated records' audit metadata: `008e9d20`, `00911370`, `009113b0`, `00912310`, `009112f0`, `00911330`, `00a2de10`, `00a382b0`, `00bd9880`. Those shards were excluded. `preflight.json` records the failed preservation rehearsal; `preflight-safe.json` records the passing nine-shard one. In particular, the lead's send-helper audit at `00a382b0` was preserved.

**Remaining work is blocked at the pipeline acceptance path:** 57 W87 corrections (27 proposed upgrades, 29 corrections at unchanged confidence, 1 downgrade) have passed `check_fix`, but cannot be safely applied through whole-shard `accept` while these existing gates/history problems remain. `recheck.py done` currently records verdicts/caps but does not apply their `fix` objects or raise confidence; treating it as a fix applier would be wrong. Repair of the stale mapper evidence and history-preserving accepted-label review tooling is a separate lead-owned follow-up. No direct label-table patch or validation bypass was used.

The 50 other reviewed targets retain their uncertainty. No claim is made that every dependency is now confirmed. Generated vault guides and the pre-existing `native-map.tsv` remain older snapshots; current queries must use `map.sqlite` and this receipt. No global regeneration of shared notes was performed.

## Files and security review

Project file written: this note. Native-map writes: `map.sqlite`/SQLite journal sidecars, `dispatch.log`, `status.json`, S0618's review-round marker and acceptance/fixcheck outputs, the nine `W87-review.jsonl` files, and `fixcheck-queue.jsonl` additions through the pipeline. Private W87 files include four helper scripts, per-shard review JSONL, input/target/before-state receipts, 151 function evidence sets, dispatcher/additional disassembly, preflight clone and before-apply backup databases, application/gate/security receipts. No original binary was changed.

Security review used secure-code-guardian's local threat boundary and security-reviewer's manual pass. The helpers have no remote input, listeners, credentials, shell execution or process control. SQL values are parameterized; acceptance validates fixed shard/address forms, checks preflight inputs, backs up the database and guards concurrent records. The invalid-callee canary was rejected by the pipeline. A stdlib AST scan plus manual review found no actionable security findings. Bandit was unavailable and was not installed; this was not a production network security audit. Replay/receipt-overwrite guards were added to the one-shot helpers.

**Ponytail:** reused nmq, xreview's no-launch dispatcher, validate/check_fix and Dispatcher.accept. Used stdlib receipts and a cloned rehearsal. Did not build a mapper, parser, new disassembler, dependency, Rust module, worker fleet or global notes regeneration. Caveman applied to chat; persisted evidence remains complete.

## W89 follow-up, 2026-10-16

**Verdict: PARTLY PROVEN; W89 is not complete.** All 57 staged W87 corrections were applied with their prior records and audits preserved. W87's 39-shard mapper check now has **0 errors**, down from 261. Three backlog shards completed full review and acceptance: **S0079, S0182, S0212**. S0215 and S0672 still lack complete mapper output; the expanded 44-shard check reports exactly these two errors. They were not marked accepted or filled with fabricated unknown records.

Ownership stayed within native-map scripts/state and this appended section. No Rust, content, saves, launcher, client, shared Plan/Ledger, commits, agents, external messages or installations were touched. The lead still owns integration and shared plan updates. RTK, Ponytail (full), Caveman, codebase-memory, reverse-engineering, secure-code-guardian and security-reviewer were applied. There was no asset-format or game-client work requiring an asset reader, community search or native run.

### Migration decision and preserved history

The validator was **not changed or weakened**. Old evidence was divided into two cases:

1. **Equivalent anchors:** migrate a vtable slot only when the current export has exactly one matching class/slot, retaining any explicitly supplied table address or object offset. The current export adds subobject offsets such as `+0x30[2]`. Reject ambiguous matches, changed table addresses and changed slot indices. For scalar deleting destructors, migrate the old `== 0` early-return anchor to the current `!= 0` free branch only when the complete current body explicitly calls `_free(param_1)` in that branch and returns the receiver. The existing descriptions already say free-when-set. This rewrote 114 mapper records in 17 shards and reduced 261 errors to 19.
2. **Changed or ambiguous meaning:** review the current function and native bytes, then write a complete correction through `check_fix`, capped at likely. Seventeen residual mapper records were reviewed. Seven concern renamed pseudocode temporaries; six of those also need the recovered post-free loop reflected in their description. The remaining ten have shortened or obsolete vtable identities. Direct table-entry bytes were checked for every residual vtable correction. All 19 residual errors were resolved. Fifteen records already had accepted labels and were corrected there too; `0101a1f0` and `0101a200` had no accepted label and remain mapper-only corrections, pending their owning shard's normal review.

Original mapper bytes are preserved in `lab/native-map/run/upkeep-W89/mapper-before/` and `review-mapper-before/`. `migration.json` contains original/current file hashes and every before/after record; `review-migration.json` contains each reviewed correction and reason. No existing mapper/reviewer/audit file was deleted. The private helpers refuse completed replay; the reviewed-migration helper also preserves its original backup on interrupted retry.

Whole-shard `Dispatcher.accept` rebuilds records from mapper output and can replace unrelated post-acceptance audit metadata. It remains the path for newly mapped shards, but is unsuitable for revising a small accepted subset. The existing `recheck.py done` now has an explicit `--apply-fixes` mode. It uses the original `check_fix` gate, requires accepted rows to match the claim-time snapshot exactly, and applies the complete batch in one `BEGIN IMMEDIATE` transaction. It retains every old audit plus the complete prior record (apart from the audits already retained separately) inside the new audit entry. Owning shard, status and pass fields are unchanged. Duplicate/missing/extra addresses, invalid fixes, replay, changed records and invalid batch paths are rejected. The default verdict-only recheck behavior is unchanged. Normal independent fix checks still queue after application.

`xreview.py claim --sid` was added to reserve only the assigned shard, preserving the existing prompt, chunking and finish-review gates. It avoids taking a higher-priority unrelated worker's shard. No dispatcher worker/process launch or PID adoption occurred.

### Applied work and native provenance

- **57/57 staged W87 corrections applied; 0 rejected:** 27 upgrades, 29 same-confidence corrections, 1 downgrade. Batch `recheck/B0021`, rehearsed on a cloned database first: `rehearsal_changed=57 unexpected=0 fixes_valid=57`. The W87 byte evidence and `review.jsonl` remain the authority for their recovered facts; W89 did not invent new gameplay rules.
- **15 accepted-label migration corrections applied; 0 rejected:** batch `B0023`. The earlier `B0022` staging attempt encountered two functions without accepted labels and aborted before any label write; its terminal receipt explicitly records `aborted`, `applied: 0`. The two mapper-only corrections were not promoted. This intermediate failure and retry are preserved.
- **361 new labels accepted** through the existing full-review gates: S0079 121/121, S0182 123/123, S0212 117/117. W89 supplied 201 remaining verdicts, including 32 valid complete fixes; the preceding 160 verdicts were retained. New reviewer files are S0079/S0182 rounds 3–4 and S0212 rounds 1–3. Every new verdict's current `nmq show` and disassembly receipts are in `upkeep-W89/evidence/`; `provenance.json` indexes all 201 addresses and reasons.
- Final live state: **99,549 labels**; shards: 828 accepted, 5 failed, 63 pending, 4 queued, 1 review_pending; no mapping/reviewing process was started. `status.json` was refreshed through `Dispatcher.status` with its normal `Spend` helper.

Native source: `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, SHA-256 recomputed as `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`. `fn.c`, `fn.vslots`, the raw executable and the original pipeline validators were used. No Legacy C# was copied. Proposed identifiers are descriptive labels, not recovered original source symbols. No original server rule or numerical balance is established here.

Important recovered corrections, each with full address-specific receipts:

| Addresses | Verified correction | Receipt/provenance |
|---|---|---|
| `008cc3d0`, `008cc480`, `008cc530`, `008cc5f0`, `00a3d170`, `00c43390` | These drain the entire linked sequence after free, rather than removing only one node. | `evidence/<addr>.review.txt`; e.g. `008cc455..008cc45c` restores the saved next pointer and loops; `00c43418..00c4341f` does the same. |
| Ten residual vtable records, listed in `review-migration.json` | Replace obsolete secondary-table notation with the actual current table/slot identity. | `evidence/<addr>.slot.txt` contains the four executable bytes at the table entry, decoded and asserted equal to that function address. |
| `0057cb63`, `0057cceb`, `0057e14e` | Color and leaf/sentinel bytes differ: `+0x24/+0x25` and `+0x2c/+0x2d`. | Current node constructors `00572027`, `0057205a`, `0055515f`; owning function show/dis receipts and S0079 verdicts. |
| `007d5850` | Equality walks predecessors to the first equal key, not the last duplicate. | Current body and predecessor `007d4340`; S0182 show/dis receipts and verdict. |
| `007d6920`, `007d7380` | Cleanup calls the container's callback; array swap delegation does not establish a free operation. | Current receiver expressions and native `007d738b..007d739e`; S0182 verdicts. |
| `00879230` | Four unrolled groups link sixteen inline nodes, not four. | Native `00879250..0087926d`, step `0x50`, four links per iteration; final count 16. |
| `0087a5a0`, `0087a690` | One inline-derived array undergoes derived/base destruction; these are not two array subobjects. | Same receiver/fields across native vtable stores; S0212 show/dis receipts. |
| `0087c390`, `0087c920` | Sum stored node weights at `+0x14` plus 16. The 4096 bound applies to the growth candidate; an explicit larger request can exceed it. | Native field loads/adds and `0087c94d..0087c98f`; S0212 verdicts. |
| Other corrections and agreements | Exact offsets, receivers, range copies, return conventions and remaining uncertainty are recorded per function. | `S0079.verdicts.jsonl`, `S0182.verdicts.jsonl`, `S0212.verdicts.jsonl`, `provenance.json`; no broader behavior is inferred. |

### Gates, exact remaining errors and files

Commands ran from `project`; private receipts live in `../lab/native-map/run/upkeep-W89/`. Repeatable command/output pairs are retained in `gate-commands.json`, `verify-before.txt`, `regression.txt`, `w87-after.txt`, `w89-after.txt`, and `verification.json`.

```powershell
python -I '..\lab\native-map\run\upkeep-W87\verify.py' --all-mappers
# BEFORE, exit 1:
# mapper_shards=39 valid_records=5515 errors=261 bad_shards=21
# sqlite_quick_check=ok

python -I scripts/native_map/test_recheck.py --baseline
# exit 1: AssertionError: validated fix object was not applied
python -I scripts/native_map/test_recheck.py
# exit 0:
# recheck_fixes=ok history=preserved confidence=capped rollback=ok replay=rejected
# duplicates=rejected concurrent_edit=rejected path_escape=rejected scoped_claim=ok

python -I '..\lab\native-map\run\upkeep-W89\migrate.py'
# migrated_shards=17 migrated_records=114 errors_before=261 errors_after=19
python -I '..\lab\native-map\run\upkeep-W89\apply_staged.py'
# rehearsal_changed=57 unexpected=0 fixes_valid=57
# APPLIED: applied=57 rejected=0 unexpected=0; upgraded=27 same-confidence=29 downgraded=1
python -I '..\lab\native-map\run\upkeep-W89\review_migration.py'
# B0023: applied=15 rejected=0
# reviewed_migration_records=17 remaining_mapper_errors=0
python -I '..\lab\native-map\run\upkeep-W89\finish_review.py' S0079
# ACCEPTED 121 records; all 121 reviewed
python -I '..\lab\native-map\run\upkeep-W89\finish_review.py' S0182
# ACCEPTED 123 records; all 123 reviewed
python -I '..\lab\native-map\run\upkeep-W89\finish_review.py' S0212
# ACCEPTED 117 records; all 117 reviewed
```

W87's frozen verifier now reports `mapper_shards=39 valid_records=5646 errors=0 bad_shards=0`, `fixes_ok=80 fixes_invalid=0 fixes_missing=0`, `live_changed_records=23 errors=0`, and `sqlite_quick_check=ok`. Its exit remains 1 because its historical preservation check still expects only W87's original 23 changes: `preservation_changed=456 unexpected=433`. Those 433 subsequent W89 changes are all explained by the current preservation receipt below; W87's verifier and baseline were preserved rather than rewritten to hide the temporal difference.

The W89 verifier compares **every label column**, retains the prior audit sequence and before-image for all 72 existing-label revisions, validates all changed live records, and expands the mapper scope to W87's 39 shards plus all five backlog shards:

```powershell
python -I '..\lab\native-map\run\upkeep-W89\verify.py' --all-mappers --receipt
# exit 1, solely the two missing-mapper errors below:
```

```text
staged_applied=57 staged_rejected=0 reviewed_migration_applied=15
preservation_changed=433 existing_changed=72 added=361 unexpected=0
audit_history_errors=0 live_record_errors=0
mapper_shards=44 valid_records=6116 errors=2 bad_shards=2
sqlite_quick_check=ok
S0079=accepted validated=121/121
S0182=accepted validated=123/123
S0212=accepted validated=117/117
S0215=pending validated=109/163
S0672=pending validated=0/182
```

Every remaining mapper error:

| Shard | Error and cause | Unfinished gate |
|---|---|---|
| S0215 | `missing records for 54 targets`; the existing mapper file ends before those functions. | Complete and validate the remaining 54 mapper records, then independent full review of all 163. |
| S0672 | `cannot read output: [Errno 2] No such file or directory: .../shards/S0672/out.jsonl`; all 182 mapper records are absent. | Produce and validate the mapper output, then independent full review of all 182. |

`verification.json` lists **all 236 missing addresses**, without the validator CLI's 40-address truncation. These are input/completeness blockers, not unresolved anchor migrations. This scoped worker did not launch mappers or write and then independently approve its own missing mappings. A zero-error check covering these two shards, and their acceptance, remain unproven. This is why W89 remains partial.

Project files written: `scripts/native_map/recheck.py`, `scripts/native_map/xreview.py`, new `scripts/native_map/test_recheck.py`, and this appended section. Private writes: the receipt helpers and before/rehearsal databases, mapper backups, migrated mapper outputs, evidence/provenance/gate/security receipts, recheck batches B0021–B0023, three shards' new review rounds/prompts/model markers, the map database, normal fixcheck queue additions, dispatch log and status. `before.sqlite` is the W89 baseline. Existing review/audit files and raw executable were preserved. Initial tempfile-based regression runs hit Windows sandbox ACL traversal errors; tests now use ordinary private native-map receipt directories without changing ACLs.

### Security review and limits

Secure-code-guardian and security-reviewer were applied to the local file/SQLite boundary. The manual scoped production-diff review found **0 actionable findings**. The stdlib AST pass found no dynamic SQL, `shell=True`, production dynamic code execution, network creation or process control. Batch path/address/shard guards, exact address sets, stale-snapshot rejection, rollback after a later invalid fix, replay rejection and unrelated-shard reservation were exercised. Bandit, Semgrep and Gitleaks were unavailable; none was installed. Test-only execution of the Git HEAD source is the regression oracle, not execution of mapper input.

SQLite commits the full correction/audit batch atomically. Secondary queue/receipt files are written afterward; an I/O failure then leaves the durable in-row audit authoritative and requires reconciliation of those secondary files. The claim snapshot prevents accidental replay. No such failure occurred in the applied batches. Confidence caps and queued independent fix checks remain distinct from Claude's lead acceptance. No native client or gameplay claim is made.

**Ponytail:** reused `validate`, `check_fix`, recheck batches, the existing independent fix queue, nmq and xreview/Dispatcher full-review gates. Added one opt-in correction path, one scoped reservation filter and a stdlib regression check. Deliberately did not build a second validator, mapper fleet, disassembler, dependency, asset reader, Rust module or global note regeneration.
