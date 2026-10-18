---
type: playbook
started: 2026-10-18
updated: 2026-10-18
---

# Build Playbook

How features get built from **2026-10-18** on, so the codebase stays lean without another whole-codebase audit. [[Build Plan]] says *what* to build; this note says *how*, and names what enforces each rule. Where things live: [[Project Map]].

## 1. Rules the pipeline enforces

A change that breaks one of these is refused automatically; nobody has to notice it.

| Rule | Enforced by | A break looks like |
|---|---|---|
| One feature's panic or error never stops the server | `contain` in `bin/osfr-gateway.rs`; test `isolation::a_panicking_session_ends_alone_and_the_server_keeps_running`; the ratchet refuses `panic = "abort"` | That player gets an `internal_error` disconnect (saved as it stands); a faulted shared field freezes until it empties |
| Hubs only shrink | `tests/spine_rules.rs` hub ceilings (40 lines of slack) and the per-task hub budget in `tick.sh` | Task requeued: "spine rules grew" |
| Busy rules live in one table | `gameplay/occupation.rs` (a new activity does not compile until every gate is decided); ratchet `busy_outside_occupation` | Build error, or the ratchet fails |
| Files are written atomically | Ratchet `plain_fs_write`: use `character_store::replace_bytes` | Ratchet fails |
| One redaction rule for hex | Ratchet `hex_outside_trace`: use `trace::hex_prefix` | Ratchet fails |
| No test that passed is lost | Integrator no-lost-pass check (fails closed); a removal needs a reason in `approved-removals.txt` | Batch refused |
| Security files are lead-only | `DENY_RE` in `lab/autopilot/common.sh` (it also covers the ratchet files) | Task parked in `needs-lead/` |
| Every builder commit is reviewed | Sol review lane; a BLOCKER becomes an F-fix brief | F-brief queued |
| The code graph is current | Refreshed after every integration; `cbm.py` restarts a stale daemon | — |

The ratchet ceilings in `tests/spine_rules.ceilings` only go down. After every green batch, the integrator lowers them to the new tip and commits them.

**Drift report:** each batch appends every measured value to `lab/autopilot/logs/drift.log` and writes one `integrator: drift ...` line to the autopilot log.

**Not enforced (review catches these):** graph-first, ponytail, test-first with a canary, (R) markers on reconstructed rules, no player names or chat in code, tests or notes, and dormancy (§4).

## 2. How one feature gets built

1. **Brief** (`lab/autopilot/queue/<id>.md`). It names:
   - the goal;
   - the module the feature lives in (never a hub);
   - the gates, as test names;
   - the `Occupation` row, if it is an activity;
   - the content table that holds its numbers (tunable, with sanity bounds).
2. **Builder** (DeepSeek):
   - graph-first and test-first;
   - follows the 17-item checklist in `worker-rules.md`;
   - adds at most 40 hub lines;
   - keeps the ratchet green.
3. **Integrator:** batch, full suite, no-lost-pass, lower the ratchet, refresh the graph.
4. **Sol review** of the commit; blockers come back as F-fix briefs.
5. **Lead review** with `ponytail-review`, plus the full suite in a clean worktree. Then merge to `main` in stages.
6. **Native check** when the client can see the change.

## 3. Where new code goes

- **A feature** is its own module: `src/<feature>.rs`, plus `src/gameplay/<feature>.rs` if it needs `Gameplay` state. A hub gets only the `mod` line and one dispatch arm.
- **An activity** adds an `Occupation` variant and decides every gate.
- **Numbers** (damage, health, prices, cooldowns) go in content tables with hot reload, never in code.
- **Client input** gets `Err` refusals. The isolation boundary is a safety net, not a pattern: never `unwrap` client-controlled data.
- **Files** go through `replace_bytes`, and **hex** goes through `trace::hex_prefix`.

## 4. Build order from here

the owner, 2026-10-18: feature briefs keep flowing alongside the deletion phase. They are not held back, and the integrator's conflict requeue handles the overlaps.

1. **Deletion phase** (restructure plan `lab/autopilot/restructure/plan.md`, one commit per step, pins first). The ratchet ceilings fall as each step lands. Briefs D001–D005 (S1, S3, S4, S5, S6) are queued. `D` ids sort ahead of the fix queue, and the Sol lane reviews them.
   - **Phase A:** S1 encounter dead code, S3 `KeyedBuckets`, S4 delete `joined`, S5 reward snapshot. S2 (delete the template fallback) waits on the owner.
   - **Phase B:** S6, the characterization net, comes first. Then S7 completion appliers, S8 one movie gate, S9 `begin_gather`, S10 tutorial craft, and S12–S14 in encounter.
   - **Phase C:** S15 NPC menu transactions, S16 split `handle_npc_packet`, S17 pets in the character document. Then S19 the crafting module move and S23 the Studio router.
   - **Open bug tickets:**
     - F150 GUID collision (queued).
     - A kart race rewrites the character save on every 50 ms tick (queued as a fix brief).
     - `realm reload all` applies half a reload and then reports "Nothing changed" (open).
2. **Dormancy** (the owner's charter: only active activities and populated instances tick).
   - Full design: `lab/autopilot/restructure/dormancy-design.md`, from a 53-agent map of every per-tick path. Nothing was measured; costs are estimates.
   - **The rule:** a session or instance ticks only when a pure, read-only check of its own live state says something is armed. The check is derived on every call and never cached, because handlers and the shared-field tick arm work without sending a message.
   - **Slice 1 (L004, done):** a shared field nobody is in ticks once more to settle (dropping departed casters' DoTs and kill credits), then sleeps until someone enters. Test `dormant_fields::an_empty_field_settles_once_then_sleeps_until_someone_enters`, canaried both ways.
   - **Next slices, in order:**
     - Hoist world maintenance (trades, invites, fishing hours) to one pass per tick. Delete the invite sweep.
     - Add a `tick` column to the Occupation table, plus `needs_tick` and `tick_needed`. An idle player then skips the deep copy of their game state, which is the largest idle cost.
     - Tick in place: drop the second gameplay copy in `Gateway::tick_at`.
     - An idle `commit_outcome` skips the shared-world copy and the social and chat-settings scans.
     - Populated fields: one copy per due tick, and stage only participants with something armed.
     - A due-time scheduler, so an empty server sleeps until a packet or deadline arrives. This matters on the handheld's battery.
     - Share static content (templates, quest definitions, recipes) between sessions instead of copying it.
   - **Needs the owner first:** freezing roamers that no player is near (behavior change, design step 9).
   - **Must keep ticking:** position flush (the 5 s crash-loss bound), open trades, the knockout revive clock, world-stage enemies, and the exact login (5 s) and reassembly (30 s) deadlines.
3. **Features:** [[Build Plan]] M1 to M4, on this process.

## 5. When to look closer (instead of a full audit)

Run a targeted review of one module, not a sweep, when any of these happen:
- a hub sits at its ceiling for three batches;
- the same Sol blocker class appears three times;
- a task hits its third requeue.

Before each merge to `main`, the lead reviews that stage.

## 6. Waiting on the owner

- Merging to `main` in stages.
- S2: delete the captured-template startup fallback.
- Power-ups on key 3.
- Ride prices and gates.
- The fishing cap value.
- The gathering full-bag policy.
- Kart gate NPCs.
