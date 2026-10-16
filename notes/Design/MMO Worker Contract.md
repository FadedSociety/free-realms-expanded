# MMO worker contract

## Current concurrency, October 10

Claude Code is the lead. the owner asked for up to 100 DeepSeek workers; the dispatcher (`project/scripts/native_map/dispatch.py`) launches them as separate `codex exec` processes and stops adding workers when free memory drops below its floor. Each worker owns exactly the output paths its prompt names. Workers never dispatch other workers. The October 9 five-worker section below is history.

## Current concurrency, October9

the owner explicitly reduced the limit to **five active subagents total**. This overrides the earlier100 request recorded below. Only the lead dispatches/replaces workers; no descendant spawning. Stop each completed child, review its concrete output, then reuse the freed slot. Prefer focused checks and bounded commands; the lead continues useful integration work while workers run.

the owner requested 100 DeepSeek workers, broad MMO restoration, test-first development, decompilation instructions and verification. Model selection is DeepSeek V4.1 Flash with max reasoning through the configured `crack_builder` role. The lead owns shared integration and the native client. This contract does not establish historical fidelity or gameplay completion.

## Required working method

1. Read project `AGENTS.md`, the relevant research note, and `%USERPROFILE%/.codex/RTK.md`.
2. Read Ponytail and Caveman under `%USERPROFILE%/.codex/skills`. Reuse existing code/community tools and write the smallest implementation that satisfies the assigned acceptance conditions. Preserve input validation, economy invariants and requested functionality. Keep communication concise.
3. Apply relevant Universal Modder skills: game-recon for identifying actual engine/formats; game-research-websearch for community/archive evidence; reverse-engineering for recovered behavior; asset-pipeline for engine-ready asset output; game-automation for lead-controlled native testing. Follow other game skills when their workflow applies. Record which were applied and their concrete use.
4. Original/community readers first. Search XeNTaX/ZenHAX/QuickBMS research before custom format implementation. Preserve original archives and keep raw/decompiled assets in the lab. Existing source references are pinned under `../lab/references`; the read-only wiki corpus is `../lab/references/freerealms-wiki/wiki.sqlite`.
5. Establish a meaningful failing test before changing nontrivial behavior. Then implement and run focused tests. Use `scripts/cargo.ps1`; no new dependency or global installation without a demonstrated need. Independent review checks behavioral evidence, not just test count.
6. Work only in explicitly assigned paths. Workers are not alone in this checkout. Preserve others' edits. Do not edit shared `lib.rs`, gameplay/gateway/Cargo manifests, shared content/save files, launcher, or release binaries unless separately assigned. Use owned `#[path]` integration tests while a module awaits lead registration.
7. Report recovered original facts separately from reconstructed rules. A server state machine is not a playable native minigame. Unknown physics, client messages and historical rewards remain gaps rather than fabricated completions.
8. Finish only with concrete paths, source provenance, commands/results, remaining integration/native gates, and known limitations. Ongoing intent belongs in commentary. A future-tense final response stops the worker prematurely.

## Review and dispatch

Every puzzle piece needs three distinct gates: implementation/tests, independent review, native/integration acceptance where applicable. Only the last gate supports a playable claim. The lead serializes shared-file integration and atomic economy changes. No worker publishes or commits game assets or decompiled source.

On October 8, the saved `features.multi_agent_v2.max_concurrent_threads_per_session` value was verified as 100 after the owner changed it. The live session still rejected a sixth active worker with `agent thread limit reached`. Five workers were running: mining, harvesting, fishing, kart racing and soccer. Housing was attempted but was not created. Reload/restart must be followed by a real spawn check; configuration alone does not prove active capacity. Preserve the queue before any app restart.

## Process ownership

Never kill processes by name, wildcard, broad process list or unverified stale PID. Only the lead controls the shared native client/harness. A worker may stop only its own exact helper PID after verifying that PID and command identify its scoped helper. Read-only review does not require process cleanup. Report a stuck helper before touching any shared service.
