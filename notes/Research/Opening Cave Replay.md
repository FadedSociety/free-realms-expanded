# Opening Cave Replay

Repair for the reported regression: after the opening campaign finished (`OpeningEngine::active_stage() == None`) the ordinary Darkthorne interaction could not start the cave again, because the guide path only begins a run from the *active* stage's pending `DungeonCompleted` objective.

Reviewed revisions (SHA-256 prefix, 2026-10-09T11:40Z):

| File | SHA-256 prefix | Change |
| --- | --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | `032ead50cbe92e9c` | replay gate + helper extraction |
| `prototype/osfr-tools/src/opening.rs` | `0c2859b6d9961b72` | read-only `stages()` getter |
| `prototype/osfr-tools/tests/opening_cave_replay.rs` | `3c6791eed7fc96b8` | new five-test regression file |

Prerepair baseline: the isolated revert of `gameplay.rs` hashes to `fd1a9a1d026640cc…`, byte-identical to the `src/gameplay.rs` revision recorded in [[Type8 Gateway Integration Handoff]] before this task. No save-schema field was added; `GameplayState` is unchanged.

Evidence bundle: `../lab/reports/opening-cave-replay-20261009T112420Z/`.

The isolated prerepair build lives in `prerepair-113259/` with its cache in `prerepair-target/`; both are private lab artifacts and safe to delete after review.

## What changed

1. **Content-derived replay binding, no save schema.** `completed_dungeon_replay` ([gameplay.rs:1625](../prototype/osfr-tools/src/gameplay.rs:1625)) reads the stages already parsed by `OpeningEngine` through the new read-only `stages()` getter ([opening.rs:427](../prototype/osfr-tools/src/opening.rs:427)) and derives the giver, stage and encounter from the objective itself. No guid, stage id or encounter id is hardcoded (`source-check.log`: no `411` or giver-guid literal in `gameplay.rs`), and existing completed saves gain the replay without a migration record.
2. **Ordinary entry only.** The new branch sits inside the existing `26/0/10/0` interaction handling, after the menu-selection check, and only when the packet is the selection-1 variant: `if let Some(id) = pending { … } else if selection { completed_dungeon_replay(guid) }` ([gameplay.rs:1092-1097](../prototype/osfr-tools/src/gameplay.rs:1092)). The menu (`26/0/8/0`), talk/gather/craft interactions and the magic-door flow ([opening_dungeon_destination](../prototype/osfr-tools/src/gameplay.rs:1541)) are untouched.
3. **Shared start path.** The Brawler-switch plus `EncounterEngine::begin` sequence is extracted into `enter_opening_cave` ([gameplay.rs:1586](../prototype/osfr-tools/src/gameplay.rs:1586)) and used by both the untouched stage-2 first entry and the replay, so both paths behave identically.

## Fail-closed gate

`completed_dungeon_replay` returns `None` (no start, ordinary reply only) unless every condition holds:

- a stage owned by the interacted giver binds `DungeonCompleted` (wrong guid → no replay);
- the whole story chain is finished and every stage is rewarded (`active_stage()` is `None`; a partial chain keeps the existing first-entry path);
- that stage's progress is completed and rewarded (unrewarded chain → no replay);
- the binding is unambiguous (two different encounter ids for one giver → no replay);
- the encounter is not already active and the dungeon is already in `EncounterState.rewarded` (no reward proof → no replay, and repeats require a real exit first).

An unknown encounter id fails inside `EncounterEngine::begin` ("unknown encounter"), which refuses the frame without committing gameplay state.

## Economy and progression

- Repeats grant nothing: the completion reward is emitted only when `rewarded.insert(id)` is new ([encounter.rs:784](../prototype/osfr-tools/src/encounter.rs:784)); a replay clear produces no coins, items or rewarded-set change. The regression test drives all four enemies to death and asserts inventory, stage progress and the rewarded set are unchanged.
- XP disclosure: this build has **no per-enemy XP path**; the only experience grant is `profiles.add_experience(25)` per opening stage reward ([gameplay.rs:1473](../prototype/osfr-tools/src/gameplay.rs:1473)), which fires once per stage because the stage is already rewarded. A replayed dungeon therefore cannot farm XP, coins or items.
- Enemy state: `EncounterEngine::begin` allocates a new instance id, resets `player_health` to maximum and rebuilds all four enemies at full health; the test asserts a fresh instance with every enemy at content max health after both the first replay and the post-exit re-entry.

## Tests

`prototype/osfr-tools/tests/opening_cave_replay.rs` (new, five tests, RED→GREEN):

- RED against the isolated prerepair build: 0 passed, 3 failed, 2 passed — `post_campaign_ordinary_guide_can_start_the_cave_again` fails with "the finished cave must start again" (`red-prerepair-opening_cave_replay.log`).
- GREEN, shipped code: `cargo test --test opening_cave_replay` → 5 passed, 0 failed (`green-opening_cave_replay.log`).
- Regression gates, all green (`green-regression-set.log`): `opening` 7 (+2 ignored), `opening_gateway` 0 (+1 ignored), `encounter` 15 (+2), `cooking_gameplay` 1 (+2), `economy_review` 18, `opening_journal` 6 (+2), `game_store` 1, `gateway` 7 (+1), `forging_gameplay` 1 (+3), `smelting_gameplay` 1 (+1), `piece_017` 7 (+2).
- Scoped `rustfmt --edition 2024` on the three touched files exits 0.

Fixture note: the post-campaign save is crafted through the public `GameplayState` + `Gameplay::load` API (the same documented pattern as `tests/economy_review.rs` and `tests/cooking_gameplay.rs`); it only marks the chain completed/rewarded and the dungeon rewarded. The guide/encounter binding is production-derived from the opening content, so there is no test-only production hook.

## Limits

- Native acceptance is not attempted here; the shared game window stays lead-controlled. No playable claim is made beyond the packet-level gateway behaviour.
- The `opening.rs` change is a single read-only accessor (`stages()`); it exposes the already-parsed content and cannot mutate state.
- The replay keeps the reconstructed cave content (waves, rewards, balance) exactly as the existing encounter module defines it.

## Reproduce

```powershell
& scripts/cargo.ps1 test --test opening_cave_replay
& scripts/cargo.ps1 test --test opening --test opening_gateway --test encounter --test cooking_gameplay `
    --test economy_review --test opening_journal --test game_store --test gateway `
    --test forging_gameplay --test smelting_gameplay --test piece_017
```
