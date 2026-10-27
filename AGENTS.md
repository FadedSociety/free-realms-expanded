# Project instructions

## Context

the owner's current goal is to revive Free Realms into a fully playable game, with Rust as the preferred implementation language. The supplied inputs are `%USERPROFILE%\Downloads\OSFR.Server.zip` and `%USERPROFILE%\Downloads\OSFR.Client.zip`. FusionFall and Toontown remain secondary design references; they do not replace the Free Realms restoration target. The folder name CartoonMMO remains provisional.

Decompilation recovers managed C# and inferred native pseudocode. Rust requires deliberate reimplementation and behavioral verification. Do not call recovered files, a successful transport handshake, or a walking emulator a complete game.

## Working paths

- Repository: `%USERPROFILE%\Documents\Projects\CartoonMMO\project`
- Obsidian vault: `notes`
- Analysis lab: `%USERPROFILE%\Documents\Projects\CartoonMMO\lab`
- Installed reverse-skill package: `%USERPROFILE%\.codex\tools\reverse-skill`
- Codex reverse-skill entry point: `%USERPROFILE%\.codex\skills\reverse-skill-router\SKILL.md`

The reverse-skill adapter can use the local package path above; it does not need another clone. Read its package rules and selected methodology when using it for an actual analysis task. Installing the package did not configure its optional MCP services or all scenario-specific tools.

## Workflow

1. Read `MODLOG.md`, `notes/Tasks.md`, and the relevant research note before continuing.
2. The two supplied archives are preserved and hash-verified in `../lab/originals`. Existing intake manifests record every extracted file. Inventory and hash any additional inputs before analysis.
3. Preserve original files. Work from copies in `../lab/working` and write decompilation/extraction output to the corresponding lab folders.
4. Record evidence, uncertainties, exact commands, and output paths. Label hypotheses until verified.
5. Update `MODLOG.md` and the task list at useful milestones. Keep original prototype code in `prototype`.
6. Before implementing an asset format, search existing community tools and XeNTaX/ZenHAX/QuickBMS research. Run the applicable reader against preserved input copies, record its pinned source and compatibility evidence, and write a custom reader only for a demonstrated gap. Engine-specific tools must match the actual engine; Free Realms uses ForgeLight-family formats, not Unity merely because a community importer runs in Unity.

Use the installed skills when relevant. Keep explanations clear and concise. Communication styles remain selectable by the user; installation alone does not select a permanent mode.

## Explicit worker instructions, October 8

the owner explicitly selected RTK, Ponytail, Caveman and relevant game skills for all workers. Read `%USERPROFILE%\.codex\RTK.md` and the selected `SKILL.md` files under `%USERPROFILE%\.codex\skills`. RTK requires normal commands first, not blindly prefixing every command. Ponytail means reuse the smallest working implementation without removing requested functionality or validation. Caveman means concise communication; persisted research and verification notes remain clear and complete.

Use game-recon, game-research-websearch, reverse-engineering and asset-pipeline where the task needs them. Use game-automation for native acceptance; only the lead controls the shared game window. Other installed game skills apply when their actual workflow is needed. Do not publish, send external messages or manipulate unrelated games merely to exercise skills.

Delegated workers follow [[notes/Design/MMO Worker Contract.md]] and the scoped ownership in [[notes/MMO Work Queue.md]]. A final response ends the worker turn: report final only after concrete artifacts/tests, or an explained blocker. Put ongoing progress in commentary. The lead interrupts finished children and reviews their evidence before integration. the owner changed the limit on October9 to **at most five active subagents total**, superseding the historical100 request. Workers must not spawn descendants; the lead fills a freed slot only after reviewing its completed work. Keep checks bounded and report progress while the lead continues integration.

## Lead and worker change, October 10

the owner's Codex weekly limit was reached; Claude Code is now the lead. DeepSeek V4.1 Flash workers (max effort) run as separate `codex exec` processes through the local Codex Router, dispatched by `scripts/native_map/dispatch.py`. the owner raised the target to up to 100 concurrent workers; actual concurrency is bounded by free RAM (~650 MB per worker with all MCP tools kept on). Workers still never spawn workers. The native function map pipeline and its quality gates are described in [[notes/Native Map/Native Map]] and `scripts/native_map/`.

## Lead and worker change, October 16

Codex hit its usage limit (resets October 22). Workers now run as Claude subagents on **Sonnet** (scoped builds and research) or **Haiku** (mechanical work), never Opus. Each worker reads `lab/runs/claude/subagent-rules.md` and then its brief in `lab/runs/luna/prompts/`. The rules cover exclusive file ownership, no git stash/reset/commit, preserving line endings, a private build dir, and disk cleanup. Workers write patches for shared files instead of editing them. **The lead does all security work personally** (auth, login broker, transport, roles, admission, inspect privacy), and reviews every worker with the full suite in a clean worktree before committing. At most five workers at once.

## Publishing, October 26

This repository is going public: everything committed here is published, history included. Only the lead pushes, tags, publishes releases or changes visibility; workers never commit to a remote, push or publish. Every commit is authored as `FadedSociety <FadedSociety@users.noreply.github.com>`. Never commit:
- secrets, tokens, private keys or per-device state (launcher profiles, handoff keys, server ids, identity and account stores, saves);
- personal emails or names, or absolute home paths (write repo-relative paths);
- anything from `../lab`, as files or as prose copied from the wiki, article and forum research: carry ids, numbers and dates with a citation;
- client binaries, packs, extracted assets, game screenshots or decompiler output (cite the client function name and address instead);
- code or test fixtures copied from AGPL projects (Sanctuary, FreeRealms-Legacy);
- build output, logs, scratch scripts, or any file over 10 MB.

Before each push the lead runs the upload gate (`../lab/autopilot/publish-check.sh`) over every new commit. Players supply their own game client; release downloads carry only our server builds and content.

## Current state and plan (read first)

Start with `notes/Project Map.md`, which maps where every module, binary, test gate, script and lab folder lives.


The verified state of every system is `notes/Design/Build Ledger.md` (rebuilt from code and native reviews 2026-10-15). The living plan is `notes/Design/Build Plan.md`. Read both before choosing work. Older notes, including some below, can be stale: verify against the code (what is routed in `gameplay.rs`/`gateway.rs`/`bin/osfr-gateway.rs`) and the newest `notes/Reviews/Native *.md` before claiming a gap. Update the Plan and Ledger after any milestone, integration or native run. Developer tools: `notes/Design/Studio Toolkit.md` and `notes/Design/World Builder.md`.

## Current implementation

- `prototype/osfr-tools`: Rust pack tools, reliable transport and local gateway. Native acceptance now includes supported character creation, the complete reconstructed cave, farm objectives, original Cooking/Smelting/Forging movies with atomic recipe rewards, wolves/Gloam/Mac combat, ordered artifact interactions and the Queen road/bridge/discovery/turn-in. All17 local opening stages are rewarded in independently reviewed native103605; native111125 reload preserved the whole gameplay state. Historical starting-area coverage remains incomplete. Native proximity prompts now align with the existing reconstructed12m interaction radius; combat melee remains3m. Read `notes/Current Progress.md` before selecting an active save/content package; prior milestone branches are preserved. World entry builds all 46 startup messages from a typed snapshot (native-verified 2026-10-14); the captured templates are still hash-validated at startup but never replayed. Original quest bindings, numerical balance and some rewards remain reconstructed; do not imply historical completeness.
- Use `scripts/cargo.ps1` to select the private Rust toolchain under `../lab/tools`; no global Rust installation is assumed.
- The opt-in live packet/debug viewer, trace formats, source files and filters are documented in `notes/Design/Debug Harness.md`.
- The existing Windows client is initially a compatibility reference. A replacement Rust client follows verified server behavior and asset readers.
- Community repositories are pinned under `../lab/references`. Preserve upstream provenance and license information for any reused material.
- Additional reverse-engineering tools: REA6.3.0 is installed and registered with the existing Ghidra/JDK; MCP reconnection is pending. Ablation2.49.0 is installed but its default PE32 architecture/address mapping failed the Free Realms smoke test, so do not trust its `window` or function context for this client. Local Laya0.4.1 runs on CPU, but scored only3–4/8 on initial function-search diagnostic cases; do not use it to discard functions or establish packet behavior. Read `notes/Research/REA Ablation and Local Laya.md` for exact paths, pins, checks and limitations before using these tools.
- Keep decompiled code and raw game assets in the lab, outside this Git repository. Protocol fixtures in tests are small synthetic messages generated with the supplied serializer.
- Current evidence, limitations, and next acceptance criteria live in `notes/Research/Free Realms Recovery.md` and `notes/Design/Rust Revival Roadmap.md`.
