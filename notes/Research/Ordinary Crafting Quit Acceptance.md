# Ordinary Crafting Quit Acceptance

Status, October 8, 2026: the ordinary (pre-win) Quit39/6 request is implemented and verified at the narrow service layer for Cooking46, Smelting77 and Forging139, with ordinary regressions in the owned sources. The request bytes come from the preserved original constructor and the pinned community writer. The accepted response is the existing local pre-win cancel policy; no original server Quit6 handler or ordinary-quit capture exists, so the response stays a labelled local choice. Fresh native acceptance remains pending.

## Original request

`Ui.QuitMiniGame` bound method `00c17470` calls `00939100`; when native active-state byte `+0x61` is false the ordinary branch calls `009be8e0`, which calls `009bdc60(6,-1,-1)`. `009bdc60` substitutes the current instance when its third argument is `-1` and serializes through `008a4420`. The pinned community `BaseMiniGamePacket` writer lays out `[short op39][byte subOpCode][int StateId][int GroupId][int GameId]`, so the request is 15 bytes: `27 00 06 <instance i32> ff ff ff ff ff ff ff ff`.

| Service | Live instance | Exact request bytes |
| --- | --- | --- |
| Cooking46 | 46000001 | `27000681e7bd02ffffffffffffffff` |
| Smelting77 | 77000001 | `27000641ed9604ffffffffffffffff` |
| Forging139 | 139000001 | `270006c1f84808ffffffffffffffff` |

Evidence: `../lab/reports/native-crafting-quit-investigation.json` (function extracts, binding table, pinned input hashes), `../lab/reports/native-crafting-quit-community-vectors.json` (unchanged compiled community writer vectors with synthetic constructor arguments), `../lab/reports/native-crafting-close-proof.json`, `../lab/decompiled/client/FreeRealms/pseudocode.c` and `../lab/references/Sanctuary-combat-pr-research/pr-76/src/Sanctuary.Packet/BaseMiniGamePacket.cs`.

## Implementation

`src/cooking.rs`, `src/smelting.rs` and `src/forging.rs` now recognize family sub-opcode `6`. A quit request must be exactly 15 bytes, carry the current active instance with `group == -1` and `game == -1`, and apply only to a non-terminal run (Cooking/Smelting `Offered|Playing`; Forging anything except `Won|Lost`). It delegates to the existing `cancel()`, which takes the run and sends the native state-removal `39/19` without any transaction. Malformed recognized packets error even with no active run; stale, foreign, terminal and replay packets are no-ops; `39/7` ACK and the four-byte results Close26/42 keep their previous meanings.

`src/gameplay.rs` needed no production routing change: both active crafting branches already forward every `39` packet to the owning service. Its added regression drives the real routing and staging path.

Response policy: the accepted behaviour is the local pre-win cancellation policy (remove-only, no loss or score page, no reward, no consumption), explicitly chosen by the task because no original server Quit6 response exists. It is not a recovered historical response.

## Tests and verification

Thirteen new ordinary regressions live with the owned sources: four per service plus one gameplay routing/atomic test. They cover the exact compiled vector, Offered and Playing pre-win runs, 14/16-byte rejection, stale instance/group/game rejection, inactive malformed rejection, idempotent replay, `39/7` ACK versus Close26/42, terminal Won/Lost refusal, and preserved cancellation economics (completed count, recipe, fresh instance, no transaction).

- RED before implementation: `cargo test --lib ordinary_quit` -> 13 failed, 0 passed (`../lab/reports/crafting-quit-20261008-flashmax/witness-preimplementation-red.txt`).
- GREEN after implementation: same filter -> 13 passed (`post-implementation-green.txt`).
- Ordinary suite after formatting: 179 passed, 0 failed, 29 ignored across 33 suites (`ordinary-suite-post-format.txt`). The +25 over the 154-test baseline is the 13 new unit tests plus the same four-per-service tests re-run in each `#[path]`-included integration binary: 13 + 12 = 25.
- Private originals: the ignored private `tests/cooking` original movie phases, `tests/smelting` original movie metadata/recipe, `tests/forging` original content movie metadata + real board roundtrip, two `tests/cooking_gameplay` atomic/once-only checks, `tests/smelting_gameplay` mining/smelting atomic check and two of three `tests/forging_gameplay` checks pass with the private references.
- One ignored private test fails on content selection, not on this change: `tests/forging_gameplay::private_fire_extinguishing_...` requires a versioned actor-repair campaign (the default `rust-opening-forging` marker has no `disappear_after_interaction`; alternatives carry six bindings, not three).
- Request oracle: `../lab/reports/crafting-quit-20261008-flashmax/verify-quit-vectors.py` re-hashes the pinned inputs, re-matches the three original functions, recomputes the 15-byte vectors from the community layout and re-checks the production sources. Boundaries: the C# writer was not recompiled and the original executable was not re-disassembled or run by this task; the script compares against the earlier pinned compiled vectors and records post-format production hashes.

## Routing metadata

Client-recorded `turn_context` for worker thread `01a11de5-3e55-72f3-958b-da829c1b5b0d`: model `deepseek/deepseek-v4.1-flash`, effort `max`; `turn_context` carries no `model_provider` or `reasoning_effort` key (session metadata records `model_provider=openai`). Private file: `../lab/reports/crafting-quit-20261008-flashmax/routing-metadata.txt`.

## Native gate

Fresh acceptance still needs a separate safe pregame branch: launch the original movie, use the actual Pause/Leave control, capture 39/6, apply the candidate response, confirm visible world/control restore with unchanged ingredients, output and quest, then retry and restart. No native client, save, content, movie or UI was touched by this task.
