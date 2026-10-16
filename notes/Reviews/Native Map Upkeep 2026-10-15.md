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


## W128 mapper completion and invalid-entry resolution, 2026-10-16

**Verdict: mapper completion gates passed; both shards staged for independent full review.** W128 authored all 236 missing mapper records: S0215's last 54 and all 182 for S0672. The existing 109 S0215 records are unchanged. Both canonical outputs pass the original validator with zero errors. Neither shard was self-accepted. `006ca9c3` is an invalid instruction entry; its complete rejection fix passes the existing fix checker and is staged only.

### Scope, provenance and method

Writes stayed within the two assigned shard folders, `lab/native-map/run/w128-*`, the dispatcher-owned queue/state/log changes, and this appended section. No pipeline script, Rust code, content, save, binary, shared Plan/Ledger, existing review verdict, commit or native client was changed. No workers or processes other than the scoped analysis commands were launched. The lead retains shared-plan updates and independent review.

The immutable source is `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, recomputed SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`. The supplied IDA full-run `labels.jsonl` hash is `0d84d3da6df5ee1e31e578ccab3ae8f151485f9eb49b41211ec914d2822c3130` (`w128-evidence/inputs.json`). No additional game input or community source was acquired.

`w128-evidence/S0215-facts.json` and `S0672-facts.json` preserve the current `fn` rows. Each newly authored target has a complete `nmq show`, raw-byte and `nmq dis` transcript at `w128-evidence/<addr>.txt`. `w128-provenance.json` indexes all 236 records and their selected anchors. W128 read the bodies and chose the labels/evidence; no script templated records or automatically selected validator-compatible anchors. Stdlib collection reused the existing `nmq` reader and its pinned Capstone environment. The existing 109 records remain prior mapper work awaiting the same independent full-shard review, not a fresh W128 semantic re-audit.

All proposed identifiers are descriptive annotations, not original symbols. These are static client facts. No original server rule, reconstructed policy, runtime gameplay result or packet direction is established by the new records.

| Shard | Existing / new | Complete output | Confidence: confirmed / likely / speculative / unknown | State |
|---|---:|---:|---|---|
| S0215 | 109 / 54 | 163 | 129 / 9 / 25 / 0 | `review_pending` |
| S0672 | 0 / 182 | 182 | 42 / 19 / 121 / 0 | `review_pending` |

Confidence remains conservative for unbound math and STL helpers: mechanical names are speculative when only one selected anchor is available. No fabricated unknown record was used to complete a shard.

Selected recovered facts, with address-specific full transcripts:

- **S0215, `0088be40`, `0088be50`, `0088caf0`, `0088cb00`, `0088c170`:** visibility virtual adapters are tail jumps; stack arguments and target return pass through. Each class is bound by its own RTTI slot. The existing `render.occlusion` subsystem was reused.
- **S0215, `0088c790` / `0088c7a0`:** the former reads `[ECX+8]`; the latter takes a stack object, returns its +8 DWORD and ends in `ret 4`. `0088cc60` consumes eight stack bytes despite using one pointer. `0088ce70` returns a float through ST0; native `0088cecb: fld [esp+0xc]` contradicts the inferred void return.
- **S0215, `0088cce0`, `0088cee0`:** three comparisons form an eight-way spatial index; insertion can split a bucket above 16 entries, with byte counts saturated at 255. Ghidra float-typed pointer variables are addresses in the native instructions. This does not identify a gameplay domain.
- **S0672, `012958c0`, `01295c10`, `01295fe0`:** account context-menu, Friends and Ignores handlers construct local UI actions. `01295ab0` reads ObjectName/AccountID and returns success but contains no image-loader call. Class names do not prove implemented image loading.
- **S0672, `01296cd0..01297180`:** direct audio handlers call service virtuals; rerouted actions queue local messages. `01296db0` supplies Volume=1.0, whereas `01296f30` reads named Volume. Callbacks consume eight stack bytes. The queues do not prove wire sends.
- **S0672, `01297520..0129b6d0`:** checked MSVC tree helpers have 24/32-byte nodes, color bytes +0x14/+0x1c and sentinel tags +0x15/+0x1d. Twelve-byte call wrappers return the original iterator (prefix operation); `0129af10..0129b000` return the tree and consume two unused arguments. The full record and byte receipt distinguish each helper, direction and offset; no payload class was inferred from adjacency.
- **S0672, `01297360`:** RuleSetDB serialization/compression returns the original serialized byte length, not the constant zero inferred by Ghidra. Supporting helper `013c2330` and final EAX load are retained in `w128-boundary/entry-call-and-extra-evidence.txt`. That file also proves `0129b030`'s output-pointer return and `ret 12` beyond the export's short byte count.

### Staging receipts and gates

`python -I -B ../lab/native-map/run/w128-stage.py` invoked **`xreview.make` and the existing `Dispatcher.finish_map(full_review=True)`**, without a dispatcher loop, worker launch, reviewer verdict or `accept` call. It set only S0215 and S0672 to `review_pending`, created round 1's normal 40-record samples and standard prompts, and preserved every live label column. The transaction's before/after digest covers **99,549 unchanged labels**; the only changed shard rows are the two assigned ones. The per-shard receipts are `shards/<sid>/w128-review-receipt.json`; the combined receipt is `w128-stage-receipt.json`.

Both shards have `out.jsonl`, `findings.md`, `mapper`, `review_round`, `review-sample.txt`, `xreview-prompt.md` and their receipt. No `review.jsonl` was fabricated. The independent reviewer can use the existing scoped `xreview.py claim ... --sid` path. Full review of all 163/182 records and acceptance are still open gates.

The repeatable verification command, run from `project/`, is:

```powershell
python -I -B ../lab/native-map/run/w128-verify.py
```

It invokes the original CLIs with these results (all exit 0):

```text
python -I scripts/native_map/validate.py ../lab/native-map/run/map.sqlite S0215 ../lab/native-map/run/shards/S0215/out.jsonl
valid_records=163 errors=0
python -I scripts/native_map/validate.py ../lab/native-map/run/map.sqlite S0672 ../lab/native-map/run/shards/S0672/out.jsonl
valid_records=182 errors=0
S0215 stage=review_pending round=1 chunk=40 independent_review_required=1
S0672 stage=review_pending round=1 chunk=40 independent_review_required=1
S0215_retained_records_unchanged=109/109
new_mapper_records=236 evidence_transcripts=236
```

Three `nmq.py label` samples per shard were run after staging, each exit 0:

```text
S0215 0088be40: (unlabelled)
S0215 0088c7a0: (unlabelled)
S0215 0088ce70: (unlabelled)
S0672 012958c0: (unlabelled)
S0672 01296cd0: (unlabelled)
S0672 0129af10: (unlabelled)
```

These are **not post-acceptance results**: the task's independent-review staging option was used. Post-acceptance sample labels remain the lead/review gate. Exact command/output pairs and exit codes are retained in `w128-gate-commands.txt`; the full verification output is `w128-verification.txt`.

### `006ca9c3`: invalid entry; correction not applied

The real routine is **`006ca9b0..006caa61` (exclusive end, 177 bytes)**. Decoding from its prologue yields `006ca9c1: 8a68fc` (`mov ch, byte ptr [eax-4]`), followed by `006ca9c4: 8d58fd`. Thus `006ca9c3` is the displacement byte `fc`, not an instruction boundary. Returns are at `006caa01` and `006caa60`, followed by `cc` padding. IDA full-run **line 38473** independently recognizes the exact `006ca9b0` entry and produces complete pseudocode; **line 38474** associates `006ca9c3` with that same routine, with `boundary_match=false`. The latter's missing pseudocode/xrefs are exporter behavior and were not used as absence evidence.

An independent raw scan of every initialized executable section found **two direct calls to `006ca9b0`**, at `006caaee` (`e8bdfeffff`) and `006caba3` (`e808feffff`). Both decode from `006caa70`'s real entry; the callers pass the table pointer in EAX, result state in ECX and one stack argument, cleaned by the caller. No raw E8/E9, near Jcc, short Jcc/JMP or LOOP/JECXZ target reaches `006ca9c3`. This scan does not disprove arbitrary computed runtime addresses.

Every initialized section was also scanned for the absolute DWORD `006ca9c3`. Its only three occurrences, at **`01868268`, `01868623`, `0186862b`**, are `c3 a9 6c 00` inside UTF-8 locale words (`?l`, `aqu?l`, `Aqu?l`), not vtable or address operands. Raw bytes, surrounding strings, scans, IDA rows and the two current accepted before-images are in `w128-boundary/`. Commands include `nmq.py dis 006ca9b0 177`, `bytes 006ca9b0 192`, `dis 006ca9c3 32` and the caller decode from `006caa70`; `nmq-evidence.txt` and `entry-call-and-extra-evidence.txt` retain them.

`w128-fixes/B0001/` has exactly the recheck input trio: `sample.txt`, full accepted-record before-image `records.jsonl`, and `review.jsonl` with one complete `reject` fix. `B0001` is an **unreserved staging name**, not a live recheck batch. The fix sets `name=null`, `kind=unknown`, `confidence=unknown`, and records `invalid_entry`, `superseded_by=006ca9b0` and the boundary evidence. It retires the misleading semantic label; a label fix cannot delete the erroneous `fn` row or repair graph/ranges. The valid containing routine already has its own accepted label. No `recheck.py done`, live claim or fix application ran.

```text
python -I scripts/native_map/validate.py --fixes ../lab/native-map/run/map.sqlite S0822 ../lab/native-map/run/w128-fixes/B0001/review.jsonl
fixes_ok=1 fixes_invalid=0 fixes_missing=0
boundary=mid_instruction_invalid_entry real_start=006ca9b0 end_exclusive=006caa61 verified_calls=2 invalid_entry_direct_targets=0 false_text_pointers=3
fix_checker_canary=PASS batch_shape=PASS invalid_label_unchanged=1 fixes_applied=0
```

### Files, security and remaining limits

Private outputs are the two shards' mapper/review staging files; `w128-collect.py`, `w128-boundary.py`, `w128-stage.py`, `w128-verify.py`; `w128-evidence/`, `w128-boundary/`, `w128-fixes/`; and the W128 provenance, manifest, gate and staging receipts. The pipeline wrote the two shard-state rows and appended its normal messages to `dispatch.log`. The sole project edit is this section; previous note bytes are preserved at `w128-evidence/upkeep-note-before.md`. No original binary, mapper history directory or existing label/audit was rewritten.

Ponytail (full), Caveman, reverse-engineering and the mapper's REA evidence methodology were applied. Secure-code-guardian and security-reviewer covered the local file/SQLite boundary. The scoped automated stdlib AST pass and manual review found **0 actionable findings**: fixed scoped paths, parameterized SQL, read-only analysis connections, existing dispatcher staging path, replay guard, no `shell=True`, untrusted code evaluation, network listener, credential access or process control. Bandit, Semgrep and Gitleaks were unavailable and were not installed. This is a local helper review, not a production-network security audit.

Open gates: independent full review and acceptance of both shards; the six post-acceptance label samples; independent review and optional application of the staged invalid-entry retirement; lead-owned `fn` boundary/graph repair. The earlier 109 S0215 records passed mechanical validation and were preserved, not independently semantically re-reviewed by their completing mapper. No broader native-map, runtime or gameplay completeness claim is made.

**Ponytail:** reused `nmq`, current exports, the existing PE/Capstone reader, `validate`/`check_fix`, `xreview.make`, `Dispatcher.finish_map` and the recheck trio. Deliberately did not build a mapper fleet, replacement validator/disassembler, dependency, pipeline extension or gameplay code.


## W129 independent review input blocker, 2026-10-16

**Verdict: BLOCKED before semantic review.** The required IDA second opinion is absent for all 345 targets: the supplied `lab/native-map/run/ida-check/full-20261016/labels.jsonl` contains zero S0215 addresses and zero S0672 addresses. It inspected previously accepted confirmed/likely labels, whereas these two shards have no accepted labels. The full run's 99.5% boundary agreement cannot be applied to these targets. No semantic verdict, reviewer fix, reservation or acceptance was issued by W129. Both shards remain `review_pending`.

### Evidence and attempted recovery

W129 read the current Plan/Ledger, AGENTS, worker contract, native-map introduction, W128 section, reviewer prompt, and the dispatch/xreview/validate/recheck/nmq review paths. The normal path is `xreview.py claim --sid`, complete 40-address review chunks, `validate.py --fixes`, then `xreview.py done`, which uses `Dispatcher.finish_review` and ultimately `Dispatcher.accept`. Corrections are complete records, capped at likely; speculative/unknown remain available. W129 did not substitute mapper confidence for evidence or write guessed verdicts to advance that path.

`w129-collect.py ghidra` collected `nmq.show_one`, `cmd_callers`, `cmd_bytes` and `cmd_dis` for every target, with the existing reader and its Capstone subprocess. These are collected source transcripts, **not independently reviewed labels**. Address-specific provenance is `lab/native-map/run/w129-evidence/<address>.txt`; the source is the current `fn` row and the unchanged preserved client executable. Its verified SHA-256 is `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`. The supplied IDA JSONL SHA-256 is `0d84d3da6df5ee1e31e578ccab3ae8f151485f9eb49b41211ec914d2822c3130`. `baseline.json` retains those hashes, both coverage counts and every before-image label column; both mapper outputs and the prior review note are preserved alongside it.

To recover the missing second opinion without changing the shared IDA database, W129 copied the client and its 350,088,467-byte `.i64` into `lab/native-map/run/w129-ida/`. Source/copy hashes matched. The helper reuses `ida_check.load_ida_api` and `inspect_label`, disables automatic analysis, and requests `close_database(save=False)`. Running the existing IDA plugin environment failed at IDALib initialization, before database opening:

```text
%USERPROFILE%/.claude/plugins/cache/mrexodia/ida-pro-mcp/0.1.0/.venv/Scripts/python.exe -I -B ../lab/native-map/run/w129-collect.py ida
exit 1; stdout and stderr empty; IDA records written: 0
```

A bounded diagnostic loaded `idalib.dll`, enabled console messages before importing `idapro`, and exposed the failure:

```text
before_idapro_import python=%USERPROFILE%\.claude\plugins\cache\mrexodia\ida-pro-mcp\0.1.0\.venv\Scripts\python.exe
idalib_dll_loaded
Fatal registry error: Access is denied. -> OK
Fatal registry error: Access is denied. -> OK
```

The permission-denied registry operation is the observed blocker; its specific registry key and root cause are unverified. No global configuration, registry setting, permission or installation was changed. No IDA MCP query tool was exposed in this session. The shared `.i64` still matches the private copy after the attempt. Repairing IDALib access in an authorized environment or providing an address-complete IDA export is required to resume the requested review.

### Gates and receipts

Run from `project`:

```powershell
python -I -B ../lab/native-map/run/w129-collect.py ghidra
# ghidra_transcripts= 345; exit 0
python -I -B ../lab/native-map/run/w129-blocker.py
```

The latter invokes the original mapper validators and ten `nmq.py label` commands; full command/exit/stdout/stderr pairs are in `w129-gate-commands.txt` and `w129-blocker.json`:

```text
S0215 valid_records=163 errors=0
S0215 00887f80 (unlabelled)
S0215 00887fe0 (unlabelled)
S0215 00888160 (unlabelled)
S0215 008881b0 (unlabelled)
S0215 00888210 (unlabelled)
S0215 supplied_ida_present=0 missing=163 state=review_pending
S0672 valid_records=182 errors=0
S0672 012958c0 (unlabelled)
S0672 01295ab0 (unlabelled)
S0672 01295b90 (unlabelled)
S0672 01295c10 (unlabelled)
S0672 01295fe0 (unlabelled)
S0672 supplied_ida_present=0 missing=182 state=review_pending
labels_changed=0 semantic_verdicts=0 acceptance_calls=0 security_actionable_findings=0
```

All invoked validation/label commands exit 0. This is **mapper validation, not review validation**. The label samples demonstrate the still-unlabelled state; they are not accepted-address samples. The receipt lists all 345 missing IDA addresses, not a truncated sample.

| Shard | Collected transcripts | Semantic verdicts (agree/refine/downgrade/rename/reject) | Unreviewed | Review validation / acceptance | IDA agreement |
|---|---:|---|---:|---|---|
| S0215 | 163 | 0 / 0 / 0 / 0 / 0 | 163 | Not run: required second opinion unavailable | Unavailable; 0 IDA rows |
| S0672 | 182 | 0 / 0 / 0 / 0 / 0 | 182 | Not run: required second opinion unavailable | Unavailable; 0 IDA rows |

There are no reject/downgrade reasons or acceptance receipts because those operations did not occur. Full per-address semantic review, pipeline fix validation, acceptance, ten accepted-address samples and accepted-record IDA agreement remain open. W128's separate invalid-entry retirement is outside W129's assigned shards and was not applied. No recovered gameplay/server facts are asserted by this blocked input audit.

### Files and local security pass

Writes: `w129-collect.py`, `w129-blocker.py`, `w129-blocker.json`, `w129-gate-commands.txt`, `w129-evidence/` (baseline, mapper/note before-images and 345 transcripts), `w129-ida/` (private client/database copies), and this appended section. No shard output, pipeline script, live map row, shared IDA input, Rust, content, save, release binary or shared Plan/Ledger was modified. No agent, native client, commit or publication was started.

Secure-code-guardian and security-reviewer were applied to the scoped local helper boundary. Threats considered were path escape, accidental shared-database mutation, executing mapper text and unbounded child commands. Helpers use fixed owned paths, read-only parameterized SQLite queries, JSON parsing, argument-list subprocesses and validator/query deadlines. The transcript collector's disassembler is the existing nmq implementation; no mapper-provided code is executed. Compile/AST checks found no `eval`/`exec` or `shell=True`; manual review found zero actionable security findings. No production-network security claim is made.

**Ponytail:** reused nmq, the installed Capstone reader, IDA checker's inspection routines and original validator CLIs. Deliberately did not build a second disassembler/validator, fabricate IDA agreement or semantic verdicts, bypass pipeline acceptance, install dependencies or change global IDA settings.


## W129 independent full review and pipeline acceptance, 2026-10-16

**Verdict: PROVEN for the assigned static-map gates.** The previously missing IDA input was supplied by the lead, and all 345 records were independently reviewed against the existing address-specific nmq show/callers/bytes/dis transcripts and the IDA export. Both shards are now `accepted` through the unchanged `xreview.py` full-review path. The earlier W129 blocker above is historical, not the current result. This session was not a mapper; both mapper outputs remain byte-identical to the pre-review copies. No original symbols, original server rules or native gameplay behavior are established by acceptance.

### Scope, evidence and tools

Source client: `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, recomputed SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` (`w129-inputs.json`). No additional game input was acquired. Lead-supplied IDA JSONL: `lab/native-map/run/ida-check/full-20261016/w129-shards-labels.jsonl`, SHA-256 `94e4c3188a2e9e90aee6902d5b1495fa6f71c2979433df5adbd27cf826038baf`.

Every verdict has per-address provenance: `w129-evidence/<address>.txt` contains the reused nmq transcript, and the verdict's `provenance.ida_line` identifies its exact line in the 345-row IDA export. Complete corrected records retain mechanically checked anchors. `w129-supplement.txt` records extended function tails and supporting helper queries. IDA was not launched and its shared database was not changed. The reverse-engineering skill and the pipeline evidence-workflows reference were applied; game-recon/web research/asset-pipeline/native automation were not needed for this supplied-binary label audit.

Ponytail full: reused nmq, its existing Capstone path, `validate.check_fix`, and the original `xreview`/dispatcher completion logic. Stdlib helpers only collect, compile human decisions, submit normal chunks and verify receipts. No new dependency, mapper, alternate acceptance path, pipeline edit, game/client launch, agent, commit or runtime feature was built. Raw pseudocode stays in the lab. Shared code, saves, binaries, Plan/Ledger and unrelated labels were not edited.

### Verdicts and acceptance

| Shard | Agree | Refine | Rename | Downgrade | Reject | Complete fixes | Accepted records | Final confidence C/L/S/U |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| S0215 | 139 | 1 | 1 | 0 | 22 | 24 | 163 | 111/27/25/0 |
| S0672 | 181 | 1 | 0 | 0 | 0 | 1 | 182 | 41/20/121/0 |

The 22 rejects replace wrong concrete details with complete validated records; they do not retain the rejected text. Every reviewer fix is capped at likely, with four already-speculative corrections kept speculative. Confidence was lowered on 18 S0215 records and one S0672 record under that rule. No separate `downgrade` verdict was needed. S0672's 121 speculative records describe narrow checked-iterator/tree mechanics that the bytes and IDA support; they remain speculative and do not acquire a game-domain identity merely through acceptance. The pipeline's logged `dispute 0%` is the **unfixed-dispute rate** after valid reviewer fixes, not zero raw rejects.

Each shard completed five ordinary 40-record rounds (S0215: 40/40/40/40/3; S0672: 40/40/40/40/22). For every round, the canonical `review[-N].jsonl` was written, `validate.py --fixes` passed, then `xreview.py done` called the dispatcher's `finish_review`; the final round called its own `accept`. `xreview.py claim --sid` reserved only the named shard. No label row was hand-edited. Per-round model markers are `codex-w129-independent-reviewer`, a reviewer identity marker, not a claim about a separately launched model. The artifact helpers did not launch workers.

Acceptance receipts: `lab/native-map/run/shards/{S0215,S0672}/w129-acceptance-receipt.json` and combined `w129-acceptance-receipt.json`. Exact CLI argv, stdout, stderr and exit codes: `w129-accept-commands.json`. Canonical verdicts remain under each shard; complete review copies are `w129-S0215-review.jsonl` and `w129-S0672-review.jsonl`.

### Runnable gates and observed output

From `project/`, `python -I -B ../lab/native-map/run/w129-verify.py` invokes the unchanged validator and ten nmq label queries, and checks canonical full coverage, applied fixes/caps, source rows, mapper hashes and unrelated labels. Exit 0:

```text
S0215
valid_records=163 errors=0
fixes_ok=24 fixes_invalid=0 fixes_missing=0
accepted_records=163 full_review_records=163 mapper_unchanged=true reused_transcripts_current=true
IDA start agreement=163/163 (100.0%); size agreement=161/163 (98.8%); pseudocode available=144/163
S0672
valid_records=182 errors=0
fixes_ok=1 fixes_invalid=0 fixes_missing=0
accepted_records=182 full_review_records=182 mapper_unchanged=true reused_transcripts_current=true
IDA start agreement=182/182 (100.0%); size agreement=181/182 (99.5%); pseudocode available=177/182
outside_labels_unchanged=true target_fn_unchanged=true
security_review: 0 findings; automated AST scan and manual scoped review complete
```

Every chunk also passed review validation: S0215 fixes_ok=9/8/6/1/0 and S0672=1/0/0/0/0; all ten outputs ended `fixes_invalid=0 fixes_missing=0`. Acceptance stdout:

```text
S0215 fully reviewed: 163/163 records, 0 disputed (0%); accepting with per-record caps
S0215 ACCEPTED 163 records, dispute 0%
S0215 -> accepted (round 6)
S0672 fully reviewed: 182/182 records, 0 disputed (0%); accepting with per-record caps
S0672 ACCEPTED 182 records, dispute 0%
S0672 -> accepted (round 6)
```

Five accepted-address readbacks per shard, each via `python -I -B scripts/native_map/nmq.py label <address>`:

- `00887f80`: `[accepted:likely] ParticleController::ScalarDeletingDtor`.
- `00889470`: `[accepted:speculative] CopyXYZAndScalarToFloat4`.
- `008894b0`: `[accepted:speculative] RandomUnitDirectionFromState`.
- `0088aee0`: `[accepted:likely] AllocateAlignedRecords16`.
- `0088ce70`: `[accepted:speculative] MaxFirstThreeExtents`.
- `012958c0`: `[accepted:likely] Engine::AccountRequestContextMenu::OpenContextMenuDialog`.
- `01296cd0`: `[accepted:confirmed] Audio::AudioPlaySound::Handle`.
- `01297bf0`: `[accepted:speculative] CheckedTreeValue7BF0`.
- `0129ad50`: `[accepted:speculative] FindSignedTreeKeyTag15_AD50`.
- `0129b030`: `[accepted:likely] EraseTreeNodeTag15`.

The complete readbacks, including behavior text, are in `w129-gates.txt`; `w129-verification.json` also retains each command, final confidence counts and before/after checks.

### Every rejected, renamed or refined record and evidence

Each line below is a recovered static-byte/RTTI observation, not reconstructed server policy. Full before/after records and exact evidence paths are in the corresponding per-address verdict.

- **S0215 `00887f80` - reject (likely):** Binary vslots bind this address to PositionFromShapeController slot 8, not the claimed OffsetCurveController slot 8 (that is 008881b0).
- **S0215 `008885c0` - reject (likely):** 008885c0 TEST ECX sets flags only; the null path leaves EAX untouched, not a null-test result.
- **S0215 `008885d0` - refine (likely):** 00888690 calls this body; its OrbitControllerInstance slot-1 binding plus arrays at +0x20/+0x30/+0x40 identifies the owning destructor body.
- **S0215 `00888f60` - reject (likely):** Constructor stores -1 to twelve WORDs at +8..+0x1e, representing six packed index pairs; it does not initialize six short indices.
- **S0215 `00889470` - reject (speculative):** 00889481..00889496 copies only source XYZ to +0/+4/+8 and scalar to +0xc; EAX remains destination, ret 8.
- **S0215 `008894b0` - reject (speculative):** 00889547..00889556 forces W=0 before normalization; EAX is the output pointer. This is a random 3D direction, not a four-random-component quaternion or void return.
- **S0215 `00889740` - reject (likely):** Callee 008894b0 normalizes XYZ with W=0; describing its output as a quaternion propagates a wrong data format.
- **S0215 `00889c80` - reject (likely):** 00889c80 builds direction XYZ from spherical angles and copies it to record+0x10; only W is zero. The orientation components are not all zero.
- **S0215 `00889ee0` - reject (likely):** Stores eight -1 WORDs, not five; writes 0x3f800000 (1.0), not zero, at each endpoint +0xc.
- **S0215 `0088a0f0` - reject (likely):** Both bounds vectors receive 0x3f800000 at component +0xc; the claimed zero fourth components are wrong.
- **S0215 `0088a380` - reject (likely):** 0088a38b CALL EDX is followed by RET without writing EAX; nonnull path passes through callee EAX, null path leaves incoming EAX.
- **S0215 `0088a390` - reject (speculative):** 0088a3bd calls 008785c0 with original receiver in ECX and no pushed argument; stack cookie is SEH bookkeeping, not static initialization data.
- **S0215 `0088a5c0` - reject (likely):** 0088a5f0 calls 008785c0 with ECX=this and no stack argument; cookie is bookkeeping. The body has scalar deleting destructor shape.
- **S0215 `0088a730` - reject (likely):** 0088a764 loads output from argument1+0x18; +0x28 is the source record/time field, not velocity output. Scale is min(context+0x20-recordTime, context+0x20-context+0x1c).
- **S0215 `0088aee0` - reject (likely):** Disassembly pushes alignment 0x10 and count<<4 BEFORE either manager virtual+4 call; the claimed no-stack-argument calls are wrong.
- **S0215 `0088af50` - reject (likely):** Disassembly pushes alignment 0x10 and count<<4 BEFORE either manager virtual+4 call; the claimed no-stack-argument calls are wrong.
- **S0215 `0088afc0` - reject (likely):** Disassembly pushes alignment 0x10 and count<<4 BEFORE either manager virtual+4 call; the claimed no-stack-argument calls are wrong.
- **S0215 `0088ba80` - reject (likely):** 0088ba80 rewrites pointer argument to [arg+0x15c] or global 01be46d8, loads receiver from this+4 and JMPs 00dbe330. It is not a flag or void-return call.
- **S0215 `0088bae0` - reject (likely):** 0088bae0 loads backend into EAX and never overwrites it after field store; returns backend pointer, not void.
- **S0215 `0088baf0` - reject (likely):** 00dbf160/00dbf110 operate on global flag word index 0, not per-backend flags; bytecode uses cdecl (0,-1)/(0,mask).
- **S0215 `0088bc80` - rename (likely):** Derived-then-base vtable writes and field clear match the teardown prefix in 0088c950, not constructor 0088c8a0; no constructor caller exists.
- **S0215 `0088bcf0` - reject (likely):** 0088bcf0 loads stack value into EAX, stores it at +0x2bc and RET 4 leaves EAX unchanged; return is that value, not void.
- **S0215 `0088bd10` - reject (likely):** 0088bd16 loads output pointer into EAX and all six stores leave it unchanged; returns output, not void.
- **S0215 `0088c9c0` - reject (speculative):** 0088c9db loads receiver from [ECX+8]; 0088ca4a returns output EAX and 0088ca57 is RET 4, contradicting caller cleanup.
- **S0672 `012958c0` - refine (likely):** IDA exposes X/Y tags; bytes at 0176cf0c are 59 00 00 00 58 00 00 00 (Y/X). The supposedly unnamed integers are screen coordinates.

### IDA agreement and limits

All accepted records have matching IDA function starts: **345/345, 100.0%**. The checker's `boundary_match` field tests the start address only (`ida_check.py`). Size/end agreement is **342/345, 99.1%**: S0215 161/163 (98.8%), S0672 181/182 (99.5%). These are mechanical agreement rates, not a semantic-correctness percentage. Three Ghidra source ranges are short: `00889740` 414 vs IDA 423 bytes (+9), `00889c80` 587 vs 594 (+7), `0129b030` 689 vs 695 (+6). Their actual tails were decoded from the start in `w129-supplement.txt`; the last returns are ret 16, ret 16 and ret 12. The source `fn` rows remain unchanged as required by scope. IDA pseudocode exists for 321/345 targets (S0215 144/163, S0672 177/182); the 24 remaining rows still expose IDA boundaries/call graphs/strings, and their native instructions were reviewed. No 99.5% statistic from unrelated previously accepted labels was substituted.

Important second-opinion disagreements were resolved with bytes: IDA's old-count==0 renderings at `0088bf70`, `0088c060`, `0088ca60` lose the arithmetic-result flags from XADD (the immediate JNE tests new-count zero). IDA's C comparisons at `0088c220`, `0088c240`, `0088cce0`, `0088cd40` omit the unordered carry behavior. `0088ce70` really returns an extent in ST0 (`fld` at `0088cecb`), despite IDA's integer-return inference. IDA's SIMD frame/local warnings were not treated as proof of nonexistent arguments. The export also reports 17 caller-graph and five callee-graph disjoint flags in S0215; those do not override direct bytes or RTTI. S0672's one serializer-name flag at `01297360` reflects unlabeled serializer callees, not failure of its observed virtual+0x28 serialization call.

The default pipeline sampled three corrected records into its later independent fixcheck queue: `0088a0f0`, `0088afc0`, `0088baf0`. Those post-acceptance spot-checks remain pending for the lead; this reviewer did not self-check its corrections as another independent reviewer. Broader semantic uncertainty remains in already-speculative helpers, exact source types and historical field names. No native/dynamic behavior was tested or claimed.

### Security review and files written

Applied `secure-code-guardian` and `security-reviewer` to the local file/command helpers. Threat model: wrong-shard writes, shell injection and overwriting another review. Automated AST scan plus manual inspection found **0 findings, 0 security fixes required**: fixed shard allowlist, parameterized read-only SQL, argv-list subprocesses, no shell execution/deletes, exclusive canonical verdict creation, coverage/cap assertions and unchanged unrelated-label digest. No login, credential, remote player input, network listener or Studio surface changed. This is task-scoped review, not a server security audit.

Writes: `w129-review.py`, `w129-supplement.py`, `w129-supplement.txt`, `w129-inputs.json`, `w129-decisions.py`, `w129-{S0215,S0672}-review.jsonl`, `w129-accept.py`, `w129-accept-commands.json`, `w129-acceptance-receipt.json`, `w129-verify.py`, `w129-verification.json`, `w129-gates.txt`; canonical review/sample/model/prompt/round/receipt artifacts inside the two owned shard directories; this appended section. Pipeline-owned side effects are two shard rows and 345 label rows, normal event log/WAL and three fixcheck-queue entries. Original earlier W129 blocker/collection artifacts are preserved.


## W133 independent fix checks of B0024/B0025/B0026, 2026-10-16

**Complete through the normal pipeline:** 130 address verdicts in B0027-B0030: **106 confirm (`agree`), 0 refine, 24 reject/cap**. This includes every one of the 128 unique queued fixes selected from B0024/B0025 and two explicit supplemental checks. Both semantic corrections (`006b9730`, `016c2365`) and the retirement of `006ca9c3` hold. All 93 sampled chunk associations hold; the rejected chunk records retain misleading earlier semantic text, not disproved boundary corrections. Confirm does not promote the pipeline confidence.

### Scope and provenance

The unchanged source client is `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`, SHA-256 `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`. The supplied IDA export is `lab/native-map/run/ida-check/full-20261016/labels.jsonl`, SHA-256 `0d84d3da6df5ee1e31e578ccab3ae8f151485f9eb49b41211ec914d2822c3130`. Every verdict records its exact IDA line and its own `w133-evidence/<address>.txt` nmq show/callers/dis transcript. Complete fn/label/IDA rows and available owner pseudocode are in the paired JSON dossier. These are recovered static-client facts; map names remain annotations, not original symbols. No original server rule or native gameplay claim is made.

Applied Ponytail full, Caveman, reverse-engineering and the reviewer evidence-workflows reference. Secure-code-guardian and security-reviewer covered the scoped local file/subprocess helpers. No community source, fresh game inputs, IDA process, native client, descendant worker, Rust change, dependency, commit or publication was used. Game-recon, web research, asset conversion and native automation were unnecessary for this supplied-artifact check.

`queue_fixcheck` selects every hot gameplay/network/minigame/world/script record and a stable 20% of other subsystems. Recomputed that rule and matched the selected addresses to the append-only queue. Historical duplicate queue lines were deduplicated by address. B0026 and the descriptor-array correction do not meet that deterministic selection; they were explicitly added as supplemental fixcheck targets, not silently reported as queued. No queue line was deleted.

| Source batch | Applied fixes | Unique queued | Claimed | Done |
|---|---:|---:|---:|---:|
| B0024 | 248 | 93 | 93 | 93 |
| B0025 | 104 | 35 | 35 | 35 |
| B0026 | 1 | 0 | 0 | 0 |
| Supplemental 006b9730 / 006ca9c3 (checks only) | 0 | 0 | 2 | 2 |

### Independent evidence and outcomes

An independent scan of every initialized executable section found **zero raw E8 rel32 candidates** targeting any of the 93 sampled chunks. A scan of all initialized sections found **zero absolute pointer candidates** to them. All **174** recorded incoming branches were decoded and verified against their exact target. This supports the chunk associations and falsifies none of their no-direct-call/no-vtable claims. IDA mismatch rows intentionally omit pseudocode/xrefs; their empty fields were not used as negative evidence. `chunk_of` remains an association, not exclusive source-language ownership, and shared/noncontiguous tails remain separate annotations. Arbitrarily computed runtime targets are not disproved by this static scan.

An independent operand/stored-pointer byte check proved all **50 literal citations**, including permitted prefix citations. It follows actual instruction operands and, where needed, one stored pointer; it does not accept nearby numerical data or a symbol spelling as literal use. Receipts: `w133-evidence/independent-entry-scan.json` and `independent-literals.json`.

- `006b9730` (IDA line 1085): `006b977b` loads `[esi+8]`, `006b977e` adds 8 to ESI, and `006b9789` returns `[esi+4]`. Both null guards and string comparisons agree with IDA. `006df3f0`/`006e8bd0` push static tables `017e0cd8`/`017e2444`; no next-link traversal was restored.
- `016c2365` (IDA line 66636): `016c2371` stores `0176ab48`, whose bytes contain exactly `GFC_Drivers_Draw_FontDeviceWin32`. Getter `004c886d` returns `01ccbe58`; descriptor/link/head stores agree with both decompilers. No question mark was restored.
- `006ca9c3` (IDA line 38474): decoding from `006ca9b0` yields `006ca9c1:8a68fc` and next instruction `006ca9c4`. The labelled byte is the `fc` displacement. The existing retirement stays `name=null`, `kind=unknown`, `confidence=unknown`; function-row repair remains separate.

Every non-confirm below is a `reject` under the standard fixcheck prompt: a false concrete behavior, argument, return or flag claim. No replacement `fix` objects were written. Chunk boundary metadata is retained.

| Address | One-line evidence for rejection |
|---|---|
| `0051eb29` | Chunk boundary holds, but 0051eb29 pushes 0,0 then calls nonreturning 00d83c46 at 0051eb2d. Normal capacity branch enters 0051eb32; it does not resume insertion after rethrow as the preserved description says. |
| `0058221b` | Chunk boundary holds, but 0058221f calls nonreturning 00d83c46 after two zero pushes. Normal 00582148 branch enters 00582224; the claimed throw-then-shift flow is false, not unresolved. |
| `005fcc1c` | Chunk boundary holds; 005fcc20 calls nonreturning 00d83c46. Normal branch 005fcb49 reaches 005fcc25. This entry rethrows; it does not perform the claimed recovery fill assignment. |
| `005ff354` | Chunk boundary holds; 005ff358 calls nonreturning 00d83c46. Normal branch 005ff281 enters 005ff35d. The described cleanup/continuation is not reachable by falling through this rethrow entry. |
| `0062b94e` | Chunk boundary holds; 0062b952 calls nonreturning 00d83c46. Normal branch 0062b872 enters 0062b957. Iterator initialization and range insertion are not exception recovery after this entry. |
| `0062e627` | Chunk boundary holds; 0062e62b calls nonreturning 00d83c46. Normal branch 0062e554 enters 0062e630. The claimed catch insertion/cleanup path conflates rethrow with the separate normal block. |
| `007adf80` | Chunk boundary and setup hold, but 007adfd0 calls AIL_sample_playback_rate, then only pops EDI/ESI and ret 4 at 007adfd8. EAX playback rate is returned, not discarded; failure preserves zero. |
| `008dec2f` | Chunk boundary holds, but 008dec3d:f6824201000008 tests byte [character+0x142] with mask 0x08, bit index 3. Preserved prose says bit 8, a wrong concrete flag index. |
| `00a720f0` | Citation correction holds, but prose still names BBaseClient.SocialShare. 00a72118 loads pointer 01b50144=01815d04; bytes there spell BaseClient.SocialShare. The preceding B at 01815d03 is not passed. |
| `00ac7f40` | String correction holds, but exact-cursor claim is false: 00ac801b subtracts cursor from end; 00ac8021:7e1d accepts signed end-cursor<=0, including cursor past end. Existing-record branch also reuses it and continues model lookup, rather than ending processing. |
| `00f0f0a0` | Literal correction holds, but this is an ESI-register helper, not MSVC thiscall: 00f0f0a0 reads [esi] without ECX transfer, caller 00f0fe20 supplies ESI, and EAX returns the taken old slot while cursor advances. Hooking it as thiscall would use the wrong receiver. |
| `01409070` | Chunk boundary holds, but 013ce130 is a no-argument getter; key/factory pushed at 01409070/75 remain for 013ce230. The explicit 013ce130(0x222e2,01409000) claim is false. |
| `01442d70` | Registration sequence holds, but 01442d81 calls 013ce230 and 01442d86 immediately returns, preserving EAX (the written map slot). The explicit no-return-value claim is false. |
| `01446960` | Chunk boundary holds, but 013ce130 consumes no key/factory arguments. Pushes at 01446960/65 survive until 013ce230 ret 8; the explicit getter-with-pair claim is false. |
| `01446df0` | Chunk boundary holds, but 013ce130 consumes no key/factory arguments. Pushes at 01446df0/f5 survive until 013ce230 ret 8; the explicit getter-with-pair claim is false. |
| `0144bde0` | Registration sequence holds, but 0144bdf1 calls 013ce230 then 0144bdf6 returns with unchanged EAX (the written map slot); explicit returns-nothing claim is false. The previously pushed pair is consumed by 013ce230. |
| `0145a670` | Chunk boundary holds, but the pair pushed at 0145a670/75 is still on the stack for 013ce230 at 0145a681; that helper reads both arguments and ret 8. Claim that it has no stack arguments is false. |
| `0145b4e0` | Chunk boundary holds, but the pair pushed at 0145b4e0/e5 is still on the stack for 013ce230 at 0145b4f1; that helper reads both arguments and ret 8. Claim that it has no stack arguments is false. |
| `014880f0` | Chunk boundary holds, but 013ce130 is a no-argument getter; pushed key 0x17c/factory 01488090 belong only to 013ce230. Explicit getter(classID,factory) claim is false. |
| `014ac9f0` | Chunk boundary holds, but 013ce130 is a no-argument getter; pushed key 0x23e/factory 014ac990 belong only to 013ce230. Explicit getter-with-pair claim is false. |
| `014d0a80` | Chunk boundary holds, but 013ce130 takes no arguments; 013ce230 alone consumes key 0x1a5/factory 014d0a20 and ret 8. Explicit both-take-id-and-factory claim is false. |
| `014f9960` | Registration sequence holds, but 014f9971 calls 013ce230 then 014f9976 immediately returns with unchanged EAX (the written map slot). Explicit returns-void claim is false. |
| `014fa0e0` | Registration sequence holds, but 014fa0f1 calls 013ce230 then 014fa0f6 immediately returns with unchanged EAX (the written map slot). Explicit returns-void claim is false. |
| `0151ab90` | Chunk boundary holds, but 013ce130 takes no arguments; key 0x230/factory 0151ab30 remain pushed for 013ce230. Explicit getter(0x230,factory) claim is false. |

### Pipeline and gates

From `project/`, ran `python -I -B scripts/native_map/recheck.py make ../lab/native-map/run ../lab/native-map/run/w133-addresses.txt --kind fixcheck --size 40` (4 batches), then the existing `claim` CLI four times (B0027-B0030). After writing verdicts, completed each with `python -I -B scripts/native_map/recheck.py done ../lab/native-map/run <batch> codex-w133-independent-fixcheck`. All four exited 0; no `--apply-fixes` was used by this independent checker. Exact done argv/stdout/stderr/exit receipts: `w133-completion-commands.json`.

| Fixcheck batch | Claimed | Done | Confirm | Refine | Reject |
|---|---:|---:|---:|---:|---:|
| B0027 | 40 | 40 | 32 | 0 | 8 |
| B0028 | 40 | 40 | 37 | 0 | 3 |
| B0029 | 40 | 40 | 29 | 0 | 11 |
| B0030 | 10 | 10 | 8 | 0 | 2 |

`python -I -B ../lab/native-map/run/w133-verify.py post` exited 0. It reuses the original `validate.check_fix` with each actual owning shard and an in-memory copy of the current record, checks verdict coverage/shape, normal completion receipts and applied audit/cap behavior. This is record validation, not an alternative label writer. The ordinary `validate.py --fixes` CLI expects replacement fix objects for rejects, while the special fixcheck prompt explicitly forbids them; it was not incorrectly used to validate verdict-only fixchecks.

```text
pipeline_check_fix: valid_records=130 errors=0
B0027 claimed=40 done=40 {"agree": 32, "reject": 8}
B0028 claimed=40 done=40 {"agree": 37, "reject": 3}
B0029 claimed=40 done=40 {"agree": 29, "reject": 11}
B0030 claimed=10 done=10 {"agree": 8, "reject": 2}
fixcheck_format: reviews=130 missing=0 duplicate=0 extra=0 fix_fields=0
independent_chunks: targets=93 raw_E8_candidates=0 absolute_pointer_candidates=0 incoming_branches=174
independent_literals: citations=50 operand_or_pointer_proven=50
outside_labels_unchanged=99764/99764
input_hashes_unchanged=2/2
security_AST_and_manual_scope_review: actionable_findings=0 fixes_required=0
verdicts: confirmed=106 refined=0 rejected=24
```

All five required readbacks ran through the unchanged `nmq.py label` CLI with exit 0; full text is in `w133-gates.txt` and `w133-receipt.json`:

```text
006b9730 [accepted:likely] LookupStringEntryByKey
016c2365 [accepted:likely] Register_GFC_Drivers_Draw_FontDeviceWin32
006ca9c3 [accepted:unknown] -
007cf79f [accepted:likely] DelayLoad_ResolveImport_Chunk
00ac7f40 [accepted:unknown] -
```

### Files, security and limits

Writes: pipeline-created `recheck/B0027` through `B0030` (sample/kind/prompt/records/claimed/review/done); `w133-addresses.txt`, `w133-collect.py`, `w133-scan.py`, `w133-decisions.py`, `w133-verify.py`, `w133-evidence/` (130 paired dossiers/transcripts, snapshots, manifest, independent scans and note before-images), `w133-supplement.txt`, `w133-verdicts.jsonl`, `w133-completion-commands.json`, `w133-receipt.json`, `w133-gates.txt`, and this appended section. Normal pipeline-owned side effects are the 130 label audit/cap updates, owning shard audit appends and dispatch log/WAL. No pipeline source or unrelated label was hand-edited.

Security scope: fixed local paths and numeric target addresses, read-only parameterized analysis SQL, argv subprocesses with bounded verification timeouts, no evaluation of mapper text, no shell execution, listener, credential access, delete or process control. Stdlib AST checks and manual review found **0 actionable findings, 0 fixes required**. This is not a production-server security audit.

The fixcheck completion path caps the 24 rejected labels to unknown and clears their names. It preserves their prior prose and adds the rejection evidence; it does not rewrite the descriptions, restore prior mapper names, remove invalid fn rows or repair graph/ranges. Those retained semantic descriptions and boundary/graph repair are lead follow-up items. The 223 fixes outside the queue and explicit supplements were not semantically re-reviewed here. No dynamic or fresh IDA acceptance is claimed.

**Ponytail:** reused nmq, the supplied IDA export, existing Capstone environment, `queue_fixcheck`, `recheck make/claim/done` and `validate.check_fix`. Deliberately did not build a new pipeline, disassembler, validator, dependency, gameplay module or native test harness.
