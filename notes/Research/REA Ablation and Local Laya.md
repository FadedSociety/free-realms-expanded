# REA, Ablation and local Laya

Requested October 9–10, 2026. Tools are installed locally. Installation and a
readiness check do not establish faster or correct game restoration.

## REA

- Upstream: https://github.com/morluto/rea, MIT; source pinned to
  `817efaca9aafc4a72884773ff8b93a32cb6e3350` in `../../../lab/references/rea`.
- Installed npm `rea-agents@6.3.0` under `%USERPROFILE%/.codex/tools/rea-runtime`.
- Official installer acquired the upstream skill. Setup supplied bundled skill
  revision34; the initial main revision35 is backed up in the private report, and
  the active `.codex/skills/reverse-engineer-anything` is aligned with the release.
  Setup also installed its matching instructions in `.agents/skills`.
- Added only the REA MCP registration to `.codex/config.toml`; setup backed up the
  previous configuration. Uses existing Ghidra12.1.4 and JDK21, no new engine.
- Scoped Codex/Ghidra doctor exits0 with `healthy=true`. The global environment
  health differs because other optional providers are absent; no need to install
  unrelated engines. Actual MCP visibility needs a Codex restart/reconnection.
- Native Windows support is experimental. No full-client REA decompilation or
  speed benchmark has been completed. The CLI remains available before reconnect.

## Ablation

- Upstream: https://github.com/Ablation-Tool/ablation; source pinned to
  `444574f60916c8c111f4d6084e904b5eca4d3f5e` in `../../../lab/references/ablation`.
- License conflict: upstream LICENSE contains GPLv3, while pyproject metadata says
  MIT. Preserve both; no upstream code incorporated into the Rust project.
- Installed2.49.0 in `../../../lab/tools/ablation-env`; CLI help exits0 and pip
  dependency check passes. A local Codex adapter is installed under
  `%USERPROFILE%/.codex/skills/ablation` and its validator passes.
- Important failed compatibility test: `ablation window FreeRealms.exe 0xb8a640`
  exits0 but prints x86-64 registers and unrelated instructions. Original PE is
  I386/imagebase0x400000; actual mapped bytes begin
  `6aff682d235e0164a1000000005081ec`.
- Inspection found WindowAnalyzer assumes ELF and defaults non-ELF to x86-64,
  without PE virtual-address mapping. BinaryContext's PE detection also falls
  back to x86-64 with installed LIEF1.0.0. A separate0.14.1 dependency probe did
  not repair it; restored1.0.0 and preserved both logs. No upstream source patched.
- Do not use these outputs for Free Realms conclusions until architecture and
  address mapping are repaired and independently compared with Ghidra. Its engine
  labeling documentation lists UE/idTech/Unity/Source2/CryEngine; ForgeLight is
  not demonstrated. Semantic matching remains an unverified optional candidate
  finder, not an accepted replacement for our existing decompiler.

## Local Laya

- The owner prefers https://huggingface.co/convaiinnovations/laya over Jev.
- Upstream runtime source: https://github.com/NandhaKishorM/laya, Apache2.0,
  commit `1adc59f7e371deb601fcfa18a14e25db238addcc`.
- Isolated runtime in `../../../lab/tools/laya-env`; pinned English model
  revision `7b928d828b7b0e022f929d9bd2e44165aa270148` under
  `../../../lab/tools/laya-model-cache`. Root weights842,609,210bytes.
- Machine inventory: Ryzen AI MAX+395/Radeon8060S,31.6GiB system RAM. Initial
  inference uses four CPU threads, no CUDA assumption, no cloud inference service.
- Model card documents512-token English context with192-token question budget.
  Evaluation refuses state snippets over300tokens rather than silently truncating.
  It uses a neutral two-option choice, with recovered XP/coin handler snippets
  and positive/confusable negative control cases.
- Initial load and download succeeded in22.78s. The first evaluation wrapper used
  the wrong tokenizer attribute; its failure is preserved and the wrapper repaired.
- Runtime warns of an invalid shipped temperature for11+option choices; no such
  choice is used here. Scores remain uncalibrated for our reverse-engineering task.
- Local diagnostic script/log/JSON and dependency locks live in
  `../../../lab/reports/reverse-tools-20261009`. No game snippets were uploaded to
  an inference API. Small diagnostic results must not be called a domain benchmark.
- Final diagnostic: default prompt4/8, explicit behavior criteria3/8, reversed
  labels3/8 on the same eight cases. First prompt calls every case positive.
  Median CPU inference0.50/0.62/0.69seconds respectively; cached startup6.4seconds.
  Three recovered snippets and five synthetic controls; these are repeated prompt
  probes, not24 independent test cases. Model manifest hashes all five downloaded
  files totaling846,195,574bytes. Runtime0.4.1 and dependency checks pass.
- Verdict: local inference works, but zero-shot results are not adequate for
  authoritative function filtering. Keep Laya available for evaluated candidate
  ranking; never discard functions or implement packet behavior on its score.
  A domain-labelled held-out evaluation/fine-tuning would be needed before broad
  use. No claim that the project has become faster is supported by this diagnostic.

Use Laya to shortlist textual snippets, then confirm original instruction/data
flow and real-client behavior. It classifies text; it does not recover machine
code into Rust or replace native acceptance.

## Native-map triage evaluation, October 10

The owner asked for Laya to be tested as a native-map triage aid with explicit criteria.
Ground truth: accepted, review-undisputed confirmed/likely labels from the native-map
pilot. Two balanced tasks, Ghidra pseudocode cut at whole lines to the 300-token state
budget, each run normally and with A/B swapped. Script and per-case rows:
`../../../lab/reports/laya-native-map-20261010/` (`eval_laya_map.py`,
`laya-native-map-eval.json`).

| task | n | accuracy | swapped accuracy | behaviour |
|---|---|---|---|---|
| library/runtime vs game code | 132 | 55.3% | 53.0% | answers "generic library" ~95% regardless of position |
| builds/sends/parses/dispatches a packet | 158 | 41.1% | 52.5% | mostly answers "yes, packet" regardless of code |

Verdict: no usable signal at chance-level accuracy; not used for native-map triage.
Jev (TypeSafe) remains an untested option; it would need an OpenRouter key and sends
pseudocode to a third-party API.

## REA in the native-map pipeline, October 10

Worker sessions only ever called REA's `binary_session` status (no binary open). On this
host REA's only PE route is a Ghidra import with default analysis, which the lead ran once
centrally (`lab/decompiled/ghidra-full-20261010b`) and exposes to workers through
`nmq show`/`nmq bytes`. Workers keep REA's methodology documents; the REA MCP server is
disabled for native-map workers to save ~230 MB RAM each.

## Jev Router (TypeSafe, via OpenRouter), October 12

The owner asked for Jev to be run on the same triage test as Laya. Same functions and expected answers (Laya's saved rows),
same task wording and A/B criteria, each run normally and swapped; pseudocode cut at whole lines to ~1200 characters
(~300 tokens, Laya's budget; code re-read from the current DB, which includes the no-return fix). Requests went through
the Codex Router (metered). Script and per-case rows: `../../../lab/reports/jev-native-map-20261012/`
(`eval_jev_map.py`, `jev-native-map-eval.json`).

| task | n | Jev accuracy | Jev swapped | Laya | Laya swapped |
|---|---|---|---|---|---|
| library/runtime vs game code | 132 | **94.7%** | **93.2%** | 55.3% | 53.0% |
| builds/sends/parses/dispatches a packet | 158 | 63.3% | 63.3% | 41.1% | 52.5% |

- Library vs game: strong and position-stable (chooses A 55% / 45% when swapped, i.e. follows the code).
- Packet: answers "no packet" ~80% of the time in both orders (yes-recall 34% / 32%). Part of this is the 300-token
  cut: packet evidence often sits later in the function or in its callees. Not reliable for packet triage as tested.
- Cost: $0.52 for all 580 questions (~$0.0009 each). Jev does not reveal which underlying model it routed to.
- Verdict: usable as a cheap pre-sorter for library vs game code (e.g. ordering or batching shards); not evidence for
  any label, and not a packet detector without a better-designed test (full code + callee context).
