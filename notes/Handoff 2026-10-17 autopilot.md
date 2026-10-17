---
date: 2026-10-17
---

# Handoff, 2026-10-17: the DeepSeek autopilot

Claude (lead) hit its weekly limit. For the next ~13 hours DeepSeek workers keep building on their own. Start with [[Project Map]], then this note.

## How it works

Everything lives in `lab/autopilot/` (not in Git). Windows Task Scheduler runs `tick.sh` every 5 minutes (task **"CartoonMMO Autopilot"**, via a hidden `run-tick.vbs`). Each tick does five things:

1. **Harvest.** For each finished builder, it takes the builder's working-tree diff into `patches/` and sorts the brief by the builder's last line:
   - `AUTOPILOT: READY` → `ready/`.
   - `AUTOPILOT: NEEDS-LEAD` → `needs-lead/`.
   - Anything else → `failed/`.
   - A run that stopped early is retried, up to 3 attempts.
   - A patch that touches a lead-only (security) file always goes to `needs-lead/`.
2. **Integrate.** `integrate.sh` runs one at a time. It applies up to 4 ready patches to the **`autopilot` branch** (worktree `CartoonMMO/autopilot-int`) and fixes line endings (`normalize.py`). Then it runs the **full suite**:
   - If no new test fails, it commits (failures that already existed on the base are tolerated, and a single flaky failure gets one rerun).
   - If the suite goes red, it reruns the patches one at a time.
   - A patch that conflicts or breaks tests goes back to the queue with the failure attached, up to 3 attempts.
   - **It never pushes and never touches `main`.**
3. **Planner briefs.** Briefs from `incoming/` are numbered into `queue/`.
4. **Builders.** Up to 6 run at once (`MAX_BUILDERS`). Each gets:
   - its own git worktree `CartoonMMO/ap-<id>`, cut from the newest `autopilot` tip;
   - its own build slot `lab/build/ap-slot<N>`;
   - the rules in `worker-rules.md`.

   No new builders launch below 60 GB free disk or 5 GB free RAM.
5. **Planner.** When fewer than 6 briefs are queued, a planner worker writes 8 new briefs (`planner.md`), up to 30 planner runs and 160 builder launches in total.

`keepawake.ps1` stops the PC from sleeping while the autopilot runs. It ends 2 hours after the deadline or when `STOP` exists.

Cargo is capped at 2 jobs and 2 test threads per worker (`lab/tools/cargo/config.toml`), because Rust compiles are what overheat the handheld. The integrator uses 6.

## Controls

- **Stop launching new work:** create `lab/autopilot/STOP`. Running builders finish, and harvest and integration continue.
- **Deadline:** `lab/autopilot/deadline` (Unix seconds).
- **Watch progress:** `lab/autopilot/logs/autopilot.log` (one line per event); integrator logs `logs/int-*.txt`; builder runs `runs/<id>-a<attempt>-<stamp>/` (`prompt.md`, `final.md`, `events.jsonl`).
- **Remove the scheduled task:** `schtasks /Delete /TN "CartoonMMO Autopilot" /F`.

## Lead review when back

1. Read `logs/autopilot.log`, then the `needs-lead/` and `failed/` briefs.
2. Review the `autopilot` branch one commit at a time: `git log main..autopilot`, each commit's diff, and `notes/Autopilot/<id>.md`. Do a ponytail review plus the security pass the lead owns.
3. Merge what passes into `main` and push. Run the full suite in the clean review worktree first.
4. Refresh the Project Map and Ledger (task T009 may already have done part of this).

## Initial queue (34 briefs, T001 to T034)

- **T001 to T009** port today's DeepSeek results onto the branch: coin store siblings, job trainers, 45/7 objectives, pets, titles, guilds, matchmaking lists, jump relay, and the notes refresh. Each waits for its source run's `exit` file.
- **T010 to T034** cover the playtest-night plan:
  - combat jobs: basic attacks and ability ladders;
  - quests: hygiene, objective events, starting chains;
  - enemies and dungeons: roaming enemies, dungeon data resolution, a second dungeon, the shared-world first slice;
  - activities: gathering payout and stations, soccer, chess, derby, the kart audit, the fishing cap switch (off by default; the owner picks the value);
  - economy and progression: vendor refusals, collections and achievements, crafting coverage;
  - social: grouping, live appearance sync;
  - upkeep: bot test hardening, the playtest bundle, warnings cleanup;
  - research: housing and the card game.

Batch 9 (W171 nodes, W173 chat settings, W174 labels, the hardened bot test), D01's bot-inbox fix and the W174 quiet-request fix were committed to `main` by the lead before the autopilot started. They are its base.

## Still waiting on the owner

These are the same as [[Handoff 2026-10-16 evening]]:
- power-ups on key 3;
- ride prices and gates;
- the fishing cap value;
- the gathering full-bag policy;
- kart gate NPC picks.
