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
