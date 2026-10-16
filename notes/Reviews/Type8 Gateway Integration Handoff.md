# Type8 Gateway Integration Handoff

Read-only handoff for wiring the repaired TYPE 8 MatchThree engines (Mining 18 and Harvesting 58/109-120/217) into the local gateway. No source, test, shared-state, UI, process or release file was modified. Reviewed revisions (SHA-256 prefix, 2026-10-09T10:38Z):

| File | SHA-256 prefix |
| --- | --- |
| `src/gameplay.rs` | `fd1a9a1d026640cc` |
| `src/gateway.rs` | `60110c143402fc1c` |
| `src/bin/osfr-gateway.rs` | `6fc4c62e77475a71` |
| `src/restoration/piece_018.rs` | `a1b1aa299ca6cf00` |
| `src/mining.rs` | `028b839f5dc6e01b` |
| `src/harvesting.rs` | `4e5196ba35ab99c6` |
| `src/lib.rs` | `1da6866e8e486db7` |

Evidence bundle: `../lab/reports/type8-integration-handoff-20261009T103850Z/` (`symbol-check.log`: 30/30 referenced symbols verified at the cited lines; `handoff-evidence.json`).

Do not launch Mining18 or Harvesting111 yet. The module-level blockers are fixed, but the original world entry and the Farmer job profile are unrecovered (see "Unknown bindings"). Everything below is the smallest hook set the lead can apply once that decision is made.

## Current wiring (verified facts)

- Application frames arrive at `Event::Application` ([gateway.rs:663](../prototype/osfr-tools/src/gateway.rs:663)), which calls `Gameplay::handle_in_world` ([gameplay.rs:540](../prototype/osfr-tools/src/gameplay.rs:540)). With opening content loaded that routes to `handle_opening_inner` ([gameplay.rs:698](../prototype/osfr-tools/src/gameplay.rs:698)); without it, to the legacy `handle`.
- `handle_opening_inner` already routes the shared `[39, 0]` family to the active minigame engine: forging at [gameplay.rs:715](../prototype/osfr-tools/src/gameplay.rs:715), cooking at [gameplay.rs:752](../prototype/osfr-tools/src/gameplay.rs:752), smelting at [gameplay.rs:789](../prototype/osfr-tools/src/gameplay.rs:789), each gated on `<engine>::active()` and paired with the `[26, 0, 42, 0]` leave request. A TYPE8 frame today falls through to the opening handler and does nothing.
- Nothing else knows these games: `rg -i "minigame" src/{gateway,gameplay,session,soe,main}.rs` returns no matches, and `lib.rs` registers neither module (`INFO lib.rs registers mining=False harvesting=False` in `symbol-check.log`).
- Outbound bytes need no new serializer. Gameplay outcomes carry inner packets; the gateway wraps them with `wrapped()` = `[5, 0, 1] + len + inner` ([gateway.rs:421](../prototype/osfr-tools/src/gateway.rs:421)). The engine's `39/16`, `39/17`, `26/12`, `39/18`, `39/47`, `39/19` frames are already inner packets.
- State is one optional field per subsystem on `GameplayState` ([gameplay.rs:48](../prototype/osfr-tools/src/gameplay.rs:48)), each `#[serde(default, skip_serializing_if = "Option::is_none")]`, reassembled in `Gameplay::state()` ([gameplay.rs:329](../prototype/osfr-tools/src/gameplay.rs:329)). `Gameplay::load` ([gameplay.rs:105](../prototype/osfr-tools/src/gameplay.rs:105)) enables a subsystem purely by file presence in the `--content` directory (`smelting.json`, `cooking.json`, `forging-campaign.json` precedent).
- Persistence order is already fail-closed for rewards: `bin/osfr-gateway.rs:197-208` clones the gateway, handles the frame, `save(...)` when `state_changed` ([osfr-gateway.rs:201](../prototype/osfr-tools/src/bin/osfr-gateway.rs:201)), and only then queues replies ([osfr-gateway.rs:207](../prototype/osfr-tools/src/bin/osfr-gateway.rs:207)). A failed save leaves the live gateway untouched.
- Timer: `Gateway::tick_at` ([gateway.rs:709](../prototype/osfr-tools/src/gateway.rs:709)) ticks only while `game.is_opening()`; `Gameplay::tick` ([gameplay.rs:506](../prototype/osfr-tools/src/gameplay.rs:506)) serves forging first and early-returns while smelting/cooking are active. A TYPE8 run bound to the opening world is covered; a run outside it is not.

## Smallest production hooks (exact edits)

1. `src/lib.rs`: add `pub mod harvesting;` and `pub mod mining;`. No `Cargo.toml` change: both modules use only the existing `serde`/`serde_json`.
2. `src/mining.rs:70`: change `const FIRST_MINING_INSTANCE` to `pub const FIRST_MINING_INSTANCE: i32 = 18_000_001;` so the gateway can pass the namespace.
3. `src/gameplay.rs:48` `GameplayState`: add, after `cooking`, `#[serde(default, skip_serializing_if = "Option::is_none")] pub mining: Option<crate::mining::Type8State>,` and the same for `pub harvesting: Option<crate::mining::Type8State>,` (the engine owns the state type; harvesting reuses it).
4. `src/gameplay.rs:69` `Gameplay`: add `mining: Option<crate::mining::Type8Engine>,`, `harvesting: Option<crate::mining::Type8Engine>,`, and `type8_tick_ms: Option<u64>` (mirrors `forging_tick_ms`).
5. `src/gameplay.rs:105` `Gameplay::load`: extend the destructure and add load blocks next to the cooking content check (around `gameplay.rs:272-300`):

```rust
let mining = if directory.join("mining.json").is_file() {
    if opening.is_none() {
        return Err(invalid("mining requires opening content"));
    }
    Some(Type8Engine::load(
        directory, "mining.json", mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE, mining,
    )?)
} else if mining.is_some() {
    return Err(invalid("mining snapshot requires its content"));
} else { None };
// harvesting: CONTENT_FILE, HARVESTING_GAME_ID (see binding decision), harvesting::FIRST_INSTANCE_ID
```

6. `src/gameplay.rs:329` `state()`: add `mining: self.mining.as_ref().map(|m| m.state().clone()),` and the harvesting twin.
7. `src/gameplay.rs:698` `handle_opening_inner`: insert a TYPE8 branch after the smelting branch (~`:798`), copying that block's shape: `[26, 0, 42, 0]` → `cancel()`; `[39, 0]` → `handle(inner)`; the blocked-action guard list (`26/13`, `26/8`, `26/10`, `36/10`, `42/0`) plus the `realm` chat guard while a run is active; route through a new `fn apply_type8(&mut self, kind, result: Type8Outcome, out)` that does `out.changed |= result.changed; out.replies.extend(result.replies); if let Some(t) = result.transaction { self.grant_reward(0, &t.grants, out)?; }`.
8. `src/gameplay.rs:506` `tick`: before the smelting/cooking early-return, tick the active TYPE8 run with `now_ms.saturating_sub(self.type8_tick_ms.unwrap_or(now_ms))` into `Type8Engine::tick`, apply it, and refresh `type8_tick_ms` (250 ms bound like forging).
9. Content writer: harvesting already has `write_content(dir, game_id)`; mining has only `mining_spec()` plus a private `Content`. Add a matching `pub fn write_content(directory: &Path, game_id: i32)` to `mining.rs` (or generate `mining.json` in the content-prep script) so the content directory can hold `mining.json` (game 18) and `harvesting.json` (game 111).

Start hook (entry) — pick one, both are reconstructions:

- **A. Dev-tools command (smallest, first native test).** In the existing `realm ` branch of `handle_opening_inner` (~`:845-860`), behind `self.dev_tools`, add `realm harvest` / `realm mine` that calls the same begin path as a world actor. No `opening.json` edit, no new trigger variant; gate stays off by default like `realm goto`.
- **B. Actor binding (later, closer to original).** Bind a reconstructed world actor action to the run, mirroring the smelting/cooking craft branches (`gameplay.rs:970/997/1026`): proximity + profile check + `<engine>.begin(profile)`, then `apply_type8`. This edits shared `opening.json`, so it needs the lead's binding decision and a material-conservation test (see unknown binding 5).

## Reward atomicity

The engine grants only from the server-side path: quota completion sets `LevelComplete`, and only `OnNextLevelMsg` then emits `OnGameWonMsg` plus `Type8Outcome::transaction`; a second `OnNextLevelMsg` is monotonic and grants nothing. `validate_spec` now refuses an output whose tile value has no goal, so an orphaned proposal fails at load instead of silently dropping grants, and `reward_transaction` returns `io::Result<Option<_>>` for defense in depth. Granting inside the staged gameplay clone means the bundle grant, the run's `completed` counter and the character snapshot are written by `save(...)` before any reply is queued; a crash between save and send leaves nothing to double-grant on reconnect. Remaining gateway gap: nothing yet routes the client's results close for TYPE8 (`26/0/42/0` is handled for cooking/forging/smelting only), so the movie's exit has no dispatch hook until step 7 lands.

## Original content and entry bindings (evidence)

- MiniGameData rows: 58 and 109-120/217 are TYPE 8; `MiniGameTypeData.IS_FLASH=1` → Flash client mode; all fourteen rows sit in activity **group 8** (88 games: Mining 18, 91-100; Harvesting 58, 109-120, 217; Forging 139-149). Source: `notes/Research/original-activity-coverage.json` and `../lab/reports/piece-015-20261009T030000Z/catalogue.json`.
- Farnum's Farm harvesting is **111**: name locale `100834`, description `94150` (stickfruit/stainberry/tumtumnip/spiralmint), icon `908`. The local opening story takes place at Farnum's Farmstead, so 111 is the only regional row with a local world relationship; 109/113/115/117/119 and the nine gap rows have no local binding.
- Launch assets: movie `game_farming.gfx` (`AssetsW_009.pack` 1448), preload list `FarmingAssets.txt` (`Assets_000.pack` 50; 97 names ending in `game_farming.lst`). Mining keeps `game_mining.gfx` / `MiningAssets.txt`.
- Drops: bundle locale `23285..23300` → item definitions `2875..2890` (`Type: 1`, `MaxStackSize: -1`; reissues `30574`/`30575`). Instance namespace `58_000_001` (harvesting) vs `18_000_001` (mining).
- Profiles shipped in the active content dir (`../lab/references/opening-journal-text-v2/content/profiles.json`): ids 1, 2, 4, 11, 12, 14, 16, 32, 35, 43, 45, 48, 49, 52, 120, 137 — Miner 14, Blacksmith 16, Chef 45. There is no Farmer id.

## Unknown bindings to resolve before recommending launch

1. **No recovered world entry.** Nothing original maps Mining18 or Harvesting111 to a door, portal, NPC or node; piece_018 lists both as `Entry::ModuleDriver` ([piece_018.rs:174](../prototype/osfr-tools/src/restoration/piece_018.rs:174), [piece_018.rs:191](../prototype/osfr-tools/src/restoration/piece_018.rs:191)). Activity group 8 is a client catalogue, not a trigger.
2. **Farmer profile missing.** Harvesting's `profile_id` equals its game id (`111`), a placeholder; a real job gate cannot be implemented until the Farmer profile id is recovered.
3. **Content binding choice.** `harvesting.json` is per game id; the lead must pick 111 (Farnum's Farm, local story) and label that 109/113/115/117/119 remain unbound local content.
4. **No TYPE8 capture.** `39/47` results rows and the native results close were never captured for MatchThree; `39/5`/`39/14`/`39/17`/`39/19` are recovered from the movie scripts and the mining oracle only.
5. **Item conservation vs the quest path.** The opening currently grants harvest materials through direct gather interactions (`opening.json` actors `...5068`/`...5069`, `resource_id` 1001/1002, handled at `gameplay.rs:1033`). Replacing or supplementing those with the minigame changes Strongarm Stew material accounting; whichever the lead chooses needs a conservation test before launch.
6. **Reconstructed numbers.** Board geometry (10x9 rectangle + dropzone row), 180 s, quota 15, 5 tiles per bundle, spawn percentages and one bundle per goal remain local choices labelled in `rule_notice`.

## Acceptance tests

Module-level (already green, keep as the regression gate): `cargo test --test harvesting` 17, `--test harvesting_review` 4, `--test mining` 21, `--test mining_review` 13, `--test piece_018` 11 (+2 ignored).

New gateway tests (proposed file `prototype/osfr-tools/tests/type8_gateway.rs`, lead-owned):

1. `type8_frames_route_only_to_the_active_engine`: begin harvesting 111 through the chosen entry; assert a `39/14 OnConnectMsg` reaches the engine, a stale-instance `39/14` is ignored, and a `39` frame with no active run changes nothing.
2. `type8_win_grants_bundles_once_before_replies`: drive quota completion, send `OnNextLevelMsg`; assert replies + `gameplay_changed`, the transaction `(2885,3)(2887,3)(2890,3)(2877,3)` lands in the inventory, a snapshot round-trips, and a replay grants nothing.
3. `type8_older_snapshots_default_absent_and_round_trip`: a snapshot without `mining`/`harvesting` loads unchanged; a snapshot with an active run round-trips (mirror `tests/cooking_gameplay.rs:16`).
4. `type8_content_required_and_gap_profiles_refused`: saved TYPE8 state without its content file fails the load with the module's "requires its content" error; `harvesting::write_content(dir, 58)` still refuses.
5. `type8_close_and_results_release_the_run`: `26/0/42/0` and `39/6` release the run; `FRServer_GameOver` emits `39/18`+`39/47`+`39/19` exactly once.

## Verify

```powershell
rg -n "pub struct GameplayState|fn handle_opening_inner|fn apply_cooking|fn grant_reward" prototype/osfr-tools/src/gameplay.rs
rg -n "fn wrapped|pub fn tick_at|Event::Application" prototype/osfr-tools/src/gateway.rs
rg -n "save\(&options.state, &staged.snapshot\(\)\)\?;|for reply in result.replies" prototype/osfr-tools/src/bin/osfr-gateway.rs
rg -n "lib.rs" -e "pub mod mining;" -e "pub mod harvesting;" prototype/osfr-tools/src/lib.rs   # no matches today
```
