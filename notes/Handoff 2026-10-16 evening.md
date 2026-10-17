---
date: 2026-10-16
---

# Handoff, 2026-10-16 evening

Stopped at 97% weekly usage. All workers are paused. Start with [[Project Map]], the Build Ledger and this note.

## Committed and pushed today (newest first)

| Commit | What |
|---|---|
| `b3dcbc9` | Per-player power-up drops, big-heart group heal, rides follow riders, all 54 rides buyable |
| `00bae61` | Real player trading with the lead's journaled two-save commit (crash-safe, dupe-proof) |
| `3ae6d0c` | Fishing (guide NPCs), mounts (family 168), Take Me There paths (98/1); **handshake no longer reloads content** (S05 fix) |
| `dc27270` | Power-up drops live, quest pin (47/14), trade close shim, hardened socket tests |
| `5f06115` | Kart NPC entry with reward cooldown, activity portal (throttled), power-up data, inventory cache fix |
| `86b1b0e`, `4a7ab21`, `f24c3ba`, `e9088d3` | Item bar, friend map, Inspect, free revive with server auto-revive, vendor sell fix (IDA-checked) |
| earlier | Broker on VPN, roles, equip rules, emotes, warpstones, gathering reconnect, quest journal, recipe book, pose-save coalescing, Public Transport design |

## Batch 9: tested in a clean worktree, not committed yet

Contents:
- W171: mining and harvesting from real nodes.
- W173: chat channel settings.
- W174: telemetry quiet list and request labels.
- W154's hardened `bot_acceptance_1016.rs`.
- `mining_nodes.json` added to the bundle list.

The files are applied in `project/` and copied into the review worktree `CartoonMMO/review`. Its full suite was running when work stopped; the log is `lab/build/review-b9-full.txt`.

To finish:
1. If the summary line shows 0 failed, stage the review copies of these files and commit:
   - `src/bin/osfr-gateway.rs`, `src/gameplay.rs`, `src/gateway.rs`, `src/harvest_nodes.rs`, `src/social_chat.rs`
   - `tests/app_fuzz.rs`, `tests/bot_acceptance_1016.rs`, `tests/chat_settings.rs`, `tests/gathering_nodes.rs`, `tests/telemetry_labels.rs`
   - `data/content/mining_nodes.json`
   - `scripts/bundle/bundle_common.py`
2. Do not include W176's files (`profiles.rs`, `npc_interaction.rs`, `job_trainers.*`); they are unfinished.

## Paused workers (partial files may be in `project/`)

Briefs are in `lab/runs/luna/prompts/full-W1NN-*.md`; worker rules are in `lab/runs/claude/subagent-rules.md`. Relaunch each from its brief, telling it to check what it already wrote.

| Worker | Task | State |
|---|---|---|
| W172 | Coin Store and Marketplace siblings | Apply script almost done (re-anchoring on `mod radial_routing`) |
| W175 | 45/7 objectives and breadcrumb crosswalk | Mid-build |
| W176 | Job trainers: an ordinary way to unlock Miner, Chef, Fisherman and Kart Driver | Partial `profiles.rs`, `npc_interaction.rs`, `job_trainers.json` and test in `project/`, uncommitted. Review or revert before the next batch |
| W177 | Pets routing | Mid-build |
| W178 | Titles and claim window | Just started |

## Waiting on the owner

- Power-ups: use them from job-bar slot 3 (key 3), shifting the level-gated abilities right? This blocks power-ups slice 3.
- Ride prices: 20 coins per Station Cash point (3,000–19,000 coins), seasonal rides sold all year, V.I.P. and membership gates not enforced.
- Fishing pays every catch with no cap (about one every 2.4 s). Add a cap?
- Gathering when the bag is full: currently the start is refused unless the whole reward fits.

## Next after that

1. Native acceptance session in the real client: `notes/Reviews/Native Checklist 2026-10-16.md`. Almost everything from today is untested natively.
2. Refresh the Project Map and Ledger for batches 6–9: trading, fishing, mounts, rides, chat settings, nodes.
3. Release-build stress run with more bots on a quiet machine, now that handshakes are cheap.

## Lessons recorded today

- Every worker gets its own ~28 GB build dir. Clean `debug/` afterwards; reviews share `lab/build/review`.
- Never kill processes by name, and never generate artificial load. Orphaned CPU burners pinned the machine at 100% for hours.
- Load-sensitive tests: wait on conditions, not fixed sleeps.
- Unauthenticated handshakes must stay cheap. A full content load per handshake was a denial-of-service hole.
