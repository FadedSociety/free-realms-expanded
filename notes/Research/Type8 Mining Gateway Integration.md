# Type8 Mining Gateway Integration

Status 2026-10-09: the repaired Mining18 TYPE8 engine is wired into the local gateway through a
dev-tools-only entry (`realm mine`) for lead native testing. **No native acceptance is claimed yet.**
The reconstructed board has no recovered world entry or quest/gather binding, so the developer
command is the only entry; nothing here claims a historical way into Mining18. The lead owns the
native run, the full suite and the release.

Reviewed revision (SHA-256), frozen for handoff:

| Path | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | `077ef90684d026f5824062065449630fce71789c94ece41243485253290ef9c5` |
| `prototype/osfr-tools/src/lib.rs` | `376f936ad4b8f5272d23df9848084db475332562f8c8390c957c156c45a14eb8` |
| `prototype/osfr-tools/src/mining.rs` | `a66073ca463d84dc6e293a5ee3c8eef8ca861a40d23d7a6eb4acd0e4a0d6e4e2` |
| `prototype/osfr-tools/tests/type8_gateway.rs` | `e1aec6e08eebe0db334ab4d2316534b562e2126fb276689cd4ca763f554af9cd` |

Evidence bundle: `../lab/reports/type8-gateway-20261009T120421Z/` (RED/GREEN logs, regression logs,
format checks, prepared `type8-content/mining.json`, manifest). No gateway, binary, Cargo, save,
shared content, UI, release or other test file was modified.

## What is wired

- `src/lib.rs` registers `pub mod mining;`.
- `src/gameplay.rs`:
  - `GameplayState.mining` is `Option<Type8State>` with `#[serde(default, skip_serializing_if =
    "Option::is_none")]`; older snapshots without the field load unchanged and round-trip byte-for-byte
    (test `older_snapshots_default_absent_mining_state`).
  - `Gameplay::load` enables Mining18 only when `mining.json` sits beside the campaign content. The
    file must bind `game_id 18` and `profile_id 14` (Miner); a saved mining run without its content
    file fails closed ("mining snapshot requires its content"), and a snapshot with more than one
    live minigame is refused.
  - `tick` advances the engine on the server clock with the same 250 ms write bound as forging; the
    authoritative timer runs the level out and never reads a client-reported time.
  - `handle_opening_inner` routes TYPE8 frames only while a run is active: `39/*` to the engine,
    `26/0/42/0` to the cancel path, `realm exit` to cancel, and the same class/equip/interact block
    list as the crafting movies (`26/0/13/0`, `26/0/8/0`, `26/0/10/0`, `36/0/10/0`, `42/0`) plus the
    `realm ` chat guard. Movement and other non-gameplay frames are untouched.
  - `apply_mining` stages the engine outcome; a completion transaction is only the engine's
    server-side derivation from the configured `spec.outputs` and the collected quota, and it is
    credited through the existing `consume_materials` + `grant_reward` inside the staged gameplay
    clone, which the gateway binary saves before any reply is queued. A client score or win report
    never authorizes a grant.
- `src/mining.rs`: `FIRST_MINING_INSTANCE`, `CONTENT_FILE`, `MINER_PROFILE_ID`, recovered locale ids
  (7910 "Mining", 7911 description), `MINING_ICON_ID` (909), `profile_id()`, `Clone` on the engine,
  and the content producer below. `mining_spec()` keeps its reconstructed labels.

## Entry: `realm mine`

Dev tools must be on (`--dev-tools`); the command is refused with an explicit chat message when they
are not. Additional gates, in order: mining content loaded; no live mining run; no live
forging/smelting/cooking/encounter; the Miner job unlocked in the save; the active job is Miner 14.
Success sends the engine's `39/16` info packet plus a short chat note. `realm exit` (or the native
`26/0/42/0` leave) cancels a pre-win run and emits `39/19` once.

Routing summary: client `39/5` -> `39/17` + `26/12 MiniGameFlash`; `39/14` payloads drive the
recovered handshake (`OnConnectMsg` -> `OnServerReadyMsg`, `OnStartGameMsg` -> `OnNextLevelMsg`,
`OnStartLevelMsg` -> board/quota/level init); `39/7` is an inert removal ACK; `39/6` with the current
instance and two `-1` sentinels cancels only a pre-win run; `FRServer_GameOver` emits
`39/18` + `39/47` + `39/19` exactly once; `FRServer_GameClose` releases the run. A frame for a stale
instance id is ignored, and every new run takes the next id in the `18_000_001` namespace.

## Reward authority

Mining pays only from `Type8Outcome::transaction`, which the engine derives from the content's
`outputs` and the server-tracked collected tiles after quota completion and the native
`OnNextLevelMsg`. A replayed `OnNextLevelMsg` is monotonic and grants nothing, a client
`FRServer_ScoreSet`/`GameWon` report is ignored, and a loss never grants. The gateway's staged clone
means the inventory grant and the run state persist together before replies. Mining outputs ship as
`outputs: []` in the prepared default content until the lead accepts a mapping.

## Native content producer

`mining::write_content(directory, 18)` writes `mining.json` for game 18 only (91 and the other TYPE8
variants are refused while their level tables are unrecovered). The prepared output is attached in
the evidence bundle as `type8-content/mining.json`, SHA-256
`25370f048ec87f12c30b38ac338d1585de3c04bbd2c080185981a1d4c8e33c8b`:
`game_id 18`, `profile_id 14`, `preload_file MiningAssets.txt`, `launch_movie game_mining.gfx`,
14x14 reconstructed board, `reconstructed_rules: true`, no outputs. To run natively, copy the private
campaign content directory, drop this file in as `mining.json`, and launch the gateway with
`--dev-tools` plus that content directory.

## Original vs reconstructed

Original: the `39` envelope and message names, payload framing, shape codes, chain rules, queue
discipline, cluster math, the `MiningAssets.txt` / `game_mining.gfx` asset names, client locale ids
7910/7911 and the opening content's Miner-14 job binding.

Reconstructed, labelled in `mining.json` and `src/mining.rs`: the board rectangle, quotas, spawn
rates, seed, RNG-to-tile mapping, instance-id namespace, operational caps and the output/reward
mapping (empty by default). No world entry, quest trigger or gathering binding was added; the
existing opening gather interactions are untouched.

## Tests and evidence

Test-first RED (before implementation): 4 compile failures in `tests/type8_gateway.rs` naming the
missing contracts (`osfr_tools::mining` unregistered, no `GameplayState.mining` field); log
`red-type8-gateway.txt`. GREEN after implementation and scoped formatting:
`cargo test --test type8_gateway` -> **10 passed, 0 failed** (`green-type8-gateway-final.txt`):

`older_snapshots_default_absent_mining_state`, `disabled_dev_tools_refuses_mining_and_changes_nothing`,
`wrong_or_locked_miner_profile_is_refused`, `type8_frames_route_only_to_the_active_run`,
`active_mining_blocks_unrelated_actions_like_crafting`, `save_reconnect_and_cancel_paths_release_the_run`,
`authoritative_loss_results_close_once_and_never_grant`,
`repeat_win_grants_the_configured_bundle_exactly_once_per_run`,
`mining_content_loads_fail_closed_without_a_bound_file`, `mining_write_content_helper_binds_game_and_job`.

Regressions on the shared paths: `mining` 21, `mining_review` 13, `piece_018` 11 (+2 ignored),
`cooking_gameplay` 2, `smelting_gameplay` 7, `dev_teleport` 7 - all passed (logs
`regression-mining-gameplay-final.txt`, `regression-private-games.txt`, `regression-dev-teleport.txt`).
`forging_gameplay` keeps its documented pre-existing, content-gated failure:
`private_fire_extinguishing_is_ordered_nearby_persistent_and_not_reinteractive` panics at
`tests/forging_gameplay.rs:396` because the default `rust-opening-forging` campaign marker has no
`disappear_after_interaction` key - the same failure recorded in
`notes/Reviews/Ordinary Crafting Quit Independent Review.md`, reproduced with the default content and
unrelated to this change. Scoped `rustfmt --check` exits 0 for all four edited files; the lead owns
the full suite and release run.

## Open native gates

1. Real client launch of `game_mining.gfx` through the `realm mine` info packet and `26/12` request.
2. Native chain input, gravity/queue display, ore bins and cluster display against this mirror.
3. `OnStopLevel` / `OnNextLevelMsg` / `OnGameWonMsg` timing and the `FRServer_GameOver` results close.
4. A recovered or explicitly accepted Mining18 output mapping; until then `outputs: []` grants nothing.
5. The original Mining level/quota tables and any historical world entry remain unrecovered; the
   reconstructed board stays labelled as such.

Cross-links: [[Mining Restoration]], [[../Reviews/Type8 Gateway Integration Handoff]],
[[Free Realms Recovery]].
