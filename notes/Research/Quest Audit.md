---
type: research
task: W92
updated: 2026-10-16
status: staged, bot-proven for three quests; lead integration and native acceptance pending
---

# Quest Audit

W92 covers Build Plan M2 items 2.4/2.5. Only `scripts/quest_audit.py`, this note and private lab outputs were written. No Rust, shared content, player save, release binary, Plan or Ledger was edited. Scratch gateways created their own disposable saves. No native client was launched or controlled.

## Result and evidence limits

The census matches the current Ledger: **653 regional quests, 14 authored objective lists, 639 hollow records, 215 absent giver keys**. The largest inferred single-type class supported by the routed engine is **talk: 86 quests, 74 hollow**. Eight conservative referrals are staged. Four have both giver and target spawning; two have unmet predecessors. The three bot gates pass, including two normal offers. This is not a claim that 639 quests have been restored.

Classification covers every record, including **146 unknown** and **149 mixed** cases. Text rules are triage hypotheses (R), not a recovered retail objective table. A name mention is a candidate entity, not proof it is a required target. Unbound enemy/item identities, quantities, delivery semantics and minigame ids remain unresolved. Guide prose can include introductory dialogue or later tasks; its inferred types are recorded separately. 604 records matched a guide table row; all rows retain their content file/index and available evidence.

`lab/reports/quest-audit-20261016/review/` is the authoritative audit output:

- `quests.json`: all 653 records, inferred/authoritative objective types, split-schema/runtime support, original goal counts and target existence, giver presence/spawn status, NPC/enemy/item/zone mentions, exact locale ids/lines, guide file/lines, prerequisites, generation and runtime blockers.
- `classification.csv` and `classification.md`: machine-readable census and the table below.
- `staged-quests.json`: eight complete replacement quest records, for review only. `content/` is a disposable copy with those records applied.
- `summary.json`, `inputs.json` (SHA-256 of locale, source data, guides, reused scripts and relevant Rust files), `content-check.json`.

Only `objectives` differs in the eight staged records. Existing rewards, locale fields, prerequisites, source metadata and all 14 authored quests are preserved. 3000021 was already native-completable through the NPC named-target fallback despite its hollow data; it is an explicit-objective regression control, not newly restored gameplay.

## Classification

| Class (text inference unless authored) | Count | Hollow | First five examples |
|---|---:|---:|---|
| collect | 55 | 55 | 3000038, 3000039, 3000053, 3000055, 3000056 |
| craft | 18 | 18 | 3000007, 3000015, 3000017, 3020002, 3020016 |
| deliver | 62 | 62 | 3000005, 3000023, 3000028, 3000052, 3010015 |
| gather | 6 | 6 | 3000006, 3030033, 3040005, 3050002, 3050005 |
| kill | 34 | 32 | 3000002, 3000013, 3000032, 3000049, 3010005 |
| level_job | 16 | 16 | 3110007, 3110013, 3110018, 3110024, 3130010 |
| minigame | 57 | 57 | 3020011, 3030004, 3030005, 3040007, 3040009 |
| mixed | 149 | 149 | 3000001, 3000003, 3000004, 3000008, 3000010 |
| talk | 86 | 74 | 3000011, 3000020, 3000021, 3000022, 3000031 |
| unknown | 146 | 146 | 3000009, 3000014, 3000016, 3000018, 3000019 |
| use_interact | 11 | 11 | 3000012, 3000054, 3030002, 3040039, 3070020 |
| visit | 13 | 13 | 3000033, 3010012, 3040014, 3060011, 3060017 |

The table uses authored goals when present, otherwise conservative description keywords and guide fallback. `objective_types` retains multiple needs for mixed cases. `unknown` explicitly means insufficient evidence; it does not mean no objective is required.

Locale coverage uses the existing checker?s JenkinsLookup2(`Global.Text.<id>`) reader against `lab/runs/client-baseline-20261014T203930Z/client/locale/en_us_data.dat`. Missing fields: 16 title, 131 description, 169 intro and 184 end text. A missing binding does not prove no corresponding prose exists somewhere else in the locale. No neighbouring text slots were newly guessed.

Entity coverage distinguishes data from runtime: 1,199 regional NPC records do not all spawn. `world.rs:181-184` skips placements other than `legacy_data`, even when a guessed position exists. Enemy definitions in `enemies.json` are distinguished from routed field/battle enemies. Item mentions are checked against the checker?s pinned runtime inventory, not the much larger Legacy item reference. `zones.json` holds regional labels; the actual overworld is zone 0, not 17 proven zone instances. Text mentions cannot establish missing retail ids or counts.

## Engine route and priority

Read-only source verification found the regional route in `Gameplay::handle_npc_packet` (`gameplay.rs:1095`, TalkToNpc) and encounter defeat handling (`gameplay.rs:4482`, KillActorKind). `gateway.rs` routes family 49/2 and 49/14, and the executable loads `Gameplay` with `--content`. Graph coverage reported changed metadata for the gateway/gameplay files; the relevant source was read directly. This agrees with the Ledger and newest relevant `Native Round5 050310` reviews: native 3000021, 3000059, 3000060, 3050011 and dev-offered 3030007 are existing controls.

`content.rs:162-194` accepts five goal variants: defeat, talk_to_npc, collect_item, reach_poi and visit_zone. `quest_engine.rs:1565-1607` adapts them. Other internal evaluator variants are not automatically authorable through split JSON. `quest_engine.rs:1293` evaluates authoritative events; only talk and kills currently have regional gameplay producers. The opening engine?s crafting/gathering/visit events do not prove the regional catalogue receives them. Loaded regional requirements remain `QuestRequirements::default()` (`quest_engine.rs:1651`).

Priority is a playtest-value/effort judgment (R), not measured player preference:

| Rank | Class | Value and remaining effort |
|---|---|---|
| 1 | Talk (86; 74 hollow) | Existing offers, counters and turn-in route. Eight text-supported bindings; four spawn-ready, two normal-offer bot proofs. Smallest data-only expansion. Remaining referrals need disambiguation, placement or predecessor work. |
| 2 | Kill (34; 32 hollow) | Existing combat counter route has high playtest value. New quests still need authoritative target/count evidence and actual field/battle placement. A model id is not automatically a kill definition id. |
| 3 | Collect 55, deliver 62, gather 6, craft 18, visit 13 | Useful quest variety, but needs lead-owned event wiring and exact item/recipe/resource/location bindings; delivery may require multiple steps and item consumption. Not a data-only class today. |
| 4 | Minigame 57 | Requires goal schema/producer work and reachable activity completion. Several corresponding games are still unrouted. |
| 5 | Use/interact 11, level/job 16, mixed 149, unknown 146 | Resolve the evidence and model the complete sequence before generation. Internal enum support alone is insufficient. |

## Staged referrals and provenance

The text itself is recovered from the shipped locale. Its association with these locally assigned quest ids is inherited content, often reconstructed by earlier backfill; it is not a decoded retail quest database. NPC keys, stable quest ids, objective id 1 and the rule ?one Talk completes this referral? are reconstructed policy (R), recorded in every generated objective. No retail reward was invented. The generator does not consult or copy Legacy source code. It reuses current NPC records whose placement provenance can include permitted Legacy built data.

Locale rows below refer to the pinned `en_us_data.dat`, using one-based file lines. NPC indices are zero-based JSON array indices under `data/content/npcs/`. Full file hashes are in `inputs.json`.

| Quest | Title | Description Global.Text id / locale line | Target record | Runtime/prerequisite gap |
|---|---|---|---|---|
| 3000021 | Introduce Yourself | 94248 / 85249 | Ricky Danger (9500000254); `cobblestone_village.json` records[16] | none identified |
| 3000022 | Introduce Yourself | 116637 / 90990 | Francis Panic (9500000786); `stillwater_crossing.json` records[6] | none identified |
| 3020014 | Blacksmith: Do Work! | 94419 / 109300 | Mili Ironcall (9000000405); `greenwood_forest.json` records[38] | predecessor 3020013 required |
| 3030008 | Brawler: Bad Business | 103355 / 91539 | Sabastian Cabet (9000000446); `blackspore_swamp.json` records[67] | predecessor 3030007 required |
| 3110012 | Ninja: Return to the Mentor | 78874 / 52274 | Ty (9000000223); `lakeshore.json` records[36] | target does not spawn (needs legacy_data placement); giver absent or does not spawn |
| 3120010 | Pet Trainer: Mosey On Up | 90040 / 8191 | Vidor Stormshroud (9000000231); `shrouded_glade.json` records[79] | giver absent or does not spawn |
| 3120022 | Pet Trainer: Down by the Bay | 90093 / 109146 | Moanna (9000000153); `seaside.json` records[63] | target does not spawn (needs legacy_data placement); giver absent or does not spawn |
| 3120024 | Pet Trainer: Seek the Master | 90103 / 54911 | Kirill Moonrunner (9000000130); `sanctuary.json` records[78] | target does not spawn (needs legacy_data placement); giver absent or does not spawn |

The generator rejects existing objectives, multiple unresolved targets, absent givers, same-giver referrals, possessive item searches and additional action verbs such as retrieving, racing or training. It accepts descriptive role suffixes and uses explicit region text to disambiguate duplicate NPC names. It deliberately stages known-but-unspawned NPC bindings with runtime blockers; it does not invent placements. Historical completeness still needs human review of title/description links and the full guide row, especially inferred Pet Trainer referrals.

Native-map checks, saved to `native-map.txt`, read accepted **confirmed** labels for `00c7cd40` DispatchQuestPacket, `00c7a990` ApplyObjectiveUpdate and `00a95420` ShowQuestEndScreen. No speculative native label was used to infer content. Per-quest targets and counts are not encoded by those generic consumers.

## Reproduction and gates

From `project`:

```powershell
python -I scripts/quest_audit.py --self-check
python -I scripts/quest_audit.py --out ../lab/reports/quest-audit-20261016/review
python -I scripts/quest_audit.py --check-copy ../lab/reports/quest-audit-20261016/review/content --out ../lab/reports/quest-audit-20261016/review
python -I scripts/content_check.py --out ../lab/reports/quest-audit-20261016/baseline-check.json --strict
```

The generation command requires a fresh output content directory and refuses overwriting it. For a repeat use a fresh child output directory. `content_check.py` has no content-path CLI flag: `--check-copy` loads that unchanged checker, assigns only its `CONTENT` root to the private copy, then invokes its original `main()` with `--strict`. It does not patch the checker or temporarily replace live files.

Actual results:

```text
PASS self-check: role suffix, duplicate/location, possessive, second task, output confinement
canary: PASS -- disabling NPC location disambiguation makes self_check fail
staged checker: == 0 errors, 17276 warnings across 134 files (4.8 s)
baseline checker: == 0 errors, 17276 warnings across 134 files (4.6 s)
verification: PASS -- 653 ids; 8 objective-only changes; all 14 existing objective lists preserved; live input hashes unchanged
```

Run `python -I ../lab/reports/quest-audit-20261016/run_checks.py` to repeat the self-check, baseline matcher failure, canary, preservation checks and saved bot receipt checks. The canary mutates code only in memory. `verification.json` and `canary.json` retain the assertions. Checker warnings are unchanged: 16,551 missing runtime items, 478 raw text hashes, 215 absent giver references and 32 other existing warnings. Zero errors is the requested checker gate, not proof all warnings are harmless.

Rust build:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\quest-audit-20261016\target'
& scripts/cargo.ps1 build --manifest-path prototype/osfr-tools/Cargo.toml --bin osfr-gateway --bin osfr-bot --offline
```

Exit 0; `Finished dev profile ... in 2m 04s`, 9 existing dead-code warnings. `build-receipt.json` pins the tested executable hashes. Shared Rust source was not frozen by this worker, so the binary hashes and runtime results are the reproducible evidence boundary.

The private `run_bots.py` assembled `bots-v5/content` from the existing `opening-anvil-placement-v1/content` campaign, overlaid the copied repository data and eight staged records, and merged the additive W85 basics by profile id rather than replacing the existing basics. The existing pinned inventory/category/old quest/combat support files were reused. No quest prerequisite, reward, NPC placement or save was manually edited for testing.

```powershell
python -I ../lab/reports/quest-audit-20261016/run_bots.py
# Each gateway:
# target/debug/osfr-gateway.exe 127.0.0.1:0 --reference <lab>/references/rust-gateway-reference --content <report>/bots-v5/content --state <report>/bots-v5/quest-<id>-scratch.json --dev-tools --studio-port 0 --seconds 18
# Each bot:
# target/debug/osfr-bot.exe --gateway <listening-address> --reference <lab>/references/rust-gateway-reference --expect-timeout 1800 --script <report>/bots-v5/quest-<id>.bot
```

`bots-v5/bot-results.json` records all exact argv arrays, addresses, PIDs and saved quest states. `quest-*.bot`, `quest-*-bot.log` and `quest-*-gateway.log` contain the packet checks. Each script proves offer 49/1, acceptance 49/3 and activation 49/8, target Talk 49/9 and 49/10, turn-in 49/13 and completion 49/4 plus 49/12. Progress expectations precede the dialogue expectation because the bot discards earlier unmatched messages while waiting.

| Quest | Offer path | Bot output | Saved result |
|---|---|---|---|
| 3000021 | Normal Samantha dialog | PASS (132 messages received) | completed, counts [1] |
| 3000022 | Normal Valerie dialog | PASS (183 messages received) | completed, counts [1] |
| 3030008 | Jones nearby, `realm offer 3030008 v0` | PASS (203 messages received) | completed, counts [1] |

All three gateways exited 0 after their own 18-second limit; bot exits are 0. The 3030008 developer offer bypasses unmet 3030007, so it proves the staged objective/turn-in path, not normal eligibility or completion of the preceding chain. Normal offers were not claimed for 3020014 or the four unspawned referrals.

Early scratch attempts are retained as failure logs: a repository-only package omitted runtime campaign files and the non-opening handler lacked `realm goto`; a whole-file overlay replaced additive ability data; then the first progress assertion waited past the packets it needed. Those setup/script errors were corrected in bots-v5. No runtime authorization or protocol guard was changed to make a test pass.

## Security and remaining gates

Threat model: local content and locale are inputs; output paths must not alias live content or unrelated files; test helpers must not reach shared/native sessions. The tool is an offline operator script, not a remote endpoint. It uses JSON, fixed local helper imports, a fixed private output root, exclusive report creation and a fresh copy directory; source content symlinks are refused. `--check-copy` resolves both checked and report paths under the same owned root. No network, shell, credential, account or permission implementation was added.

Applied `secure-code-guardian` and a `security-reviewer` self-pass: automated AST/import checks, output-confinement assertion and manual review of copy/write/import paths. **No new security findings.** `lab/reports/quest-audit-20261016/security-review.json` retains the review receipt. Scratch dev gateways bound only loopback ephemeral ports with Studio disabled. No dependency, global install, agent, commit, publish, external message or native client was used.

Open gaps: 146 unknown and 149 mixed classifications need evidence review; entity/quantity references remain unbound for most hollow quests; four staged quests lack spawning actors; two spawn-ready quests have prerequisites; no rewards or level/job gates were reconstructed here; no new native run or independent lead review is claimed. Native UI objective labels, historical multi-step completeness and reward presentation remain unverified. Shared Plan/Ledger integration belongs to the lead.

Ponytail: reused locale hashing/loading, the existing talk matcher for diagnostics, the unchanged checker, current NPC keys, the proven quest routes, private campaign and existing bot. Built one stdlib audit/generator; deliberately did not build Rust routes, another locale reader, placements, reward guesses, a new schema or dependencies. Reverse-engineering supplied provenance discipline and confirmed-label checks; no new asset format, engine reconnaissance or web research was needed.
