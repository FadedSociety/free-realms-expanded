# Harvesting Gateway Integration

Opt-in Harvesting111/Chef45 gateway candidate. Harvesting stays an unregistered separate candidate: this change adds it to the gameplay/gateway surface only when `harvesting.json` is present, leaves the mining feature and any native run untouched, and performed no release build, restart, UI drive or save/content write.

Pre-edit sources were preserved before the first edit: `gameplay.rs` `077ef90684d026f5…` and `lib.rs` `376f936ad4b8f527…` in `../lab/reports/harvesting-gateway-20261009T150000Z/preserved-before-edit/`. Post-edit freeze: `src/gameplay.rs` `9e8b56ed5f3a2f78…`, `src/lib.rs` `bb1d8755a8b6a606…`, `tests/type8_harvesting_gateway.rs` `39f5f6a88fcaa3ee…`. Untouched: `src/mining.rs` `e6e421d8e7cd0e92…` and `src/harvesting.rs` `272711da0e6b72fc…`.

## What was added

- `src/lib.rs`: `pub mod harvesting;` so the candidate module is reachable by the crate.
- `src/gameplay.rs`:
  - `GameplayState.harvesting: Option<Type8State>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` — a save without the field round-trips byte-identically (proven by `older_snapshots_default_absent_harvesting_state`).
  - `Gameplay.harvesting: Option<Type8Engine>` loaded only when `harvesting.json` sits next to the campaign content; missing content with a saved run fails closed.
  - Load validation: game id `111`, profile `CHEF_PROFILE_ID` (45), namespace `harvesting::FIRST_INSTANCE_ID` (58_000_001), `FarmingAssets.txt` preload and `game_farming.gfx` movie; wrong game, wrong profile or wrong assets refuse the load.
  - Dev-only `realm harvest` (`developer_harvesting`): requires `--dev-tools`, no competing activity (forging/smelting/cooking/mining/encounter), Chef unlocked and active; otherwise an explicit chat refusal with no state change.
  - `apply_harvesting`: the only reward path is the engine's server-side `Type8Outcome::transaction` (configured `outputs` × collected clusters). It re-checks the Chef job, stages consumption/grants inside the gameplay clone, and the gateway persists that clone before replies — client scores, win reports and progress messages never grant.
  - Unified routing on the shared type-8 tick field: `39/x` frames, `26/0/42/0` leaves, `realm exit`, and blocked class/interact/equip/inventory frames while the run is live, exactly like Mining18.

## Tests and evidence

- Test-first RED: the new gateway test file does not compile against the preserved pre-edit sources (`red-oldcompile.log`: `could not find harvesting in osfr_tools`, `no field harvesting on GameplayState`). An earlier partially-implemented run (`first-run.log`, 8 passed / 3 failed) captured the content-binding failures before the fixture was corrected.
- GREEN (`green-focused.log`): `type8_harvesting_gateway` 11 passed; `type8_gateway` (Mining10) 10 passed; `mining` 21 passed; `harvesting` 17 passed; `harvesting_job_binding` 3 passed; `harvesting_review` 4 passed; 0 failed anywhere. `rustfmt --edition 2024 --check` on the three touched files exits 0.
- The eleven gateway tests cover: byte-compatible older saves; dev-gate refusal; wrong/locked Chef refusal; region-111 header identity, Chef profile, `FarmingAssets.txt` and `game_farming.gfx` on the wire; stale-instance inertness; exclusivity against Mining18 and unrelated actions; save/reconnect/cancel/restart with the instance namespace; pause and server-authored timer loss with once-only results; client score/win forgery inertness; quota win granting the configured outputs once per run and once more after restart; full-bag rollback with a retryable `LevelComplete` run; malformed binding refusal; and the shipped region-111 config keeping outputs `2885/2887/2890/2877` with quotas 15/5.

## Findings for the lead

1. **Campaign item manifest gap (content-side).** The private campaign `inventory.json` defines the stew bundles `2875` (gummyfruit) and `2884` (rainleaf), but **not** region 111's recovered bundle definitions `2885/2887/2890/2877`. A live `harvesting.json` run would therefore fail its grant with `unknown reward definition`. The gateway tests prove the transaction mechanics with four manifest-defined stand-in definitions and separately assert the shipped config still binds the recovered ids; adding those definitions to the campaign content is a separate content task (outside this ownership).
2. **No native acceptance performed.** The mining run was active; this task never built release, restarted processes, drove the UI or touched live content/saves. Native launch remains a lead gate.
3. **Ingredient conservation unchanged.** The quest gather path (`gameplay.rs` Chef gather branch) and the stew/quest coupling were not modified; the harvesting outputs stay independent of the stew recipe.
4. **Still open from the preflight:** world entry, 39/16 reward-bundle payload and 26/12 launcher authorship, locale-vs-wiki produce set, and the harvesting score policy.

Private bundle: `../lab/reports/harvesting-gateway-20261009T150000Z/` (`preserved-before-edit/`, `red-oldcompile.log`, `first-run.log`, `green-focused.log`, `gateway-evidence.json`).
