# Type8 Mining Gateway Independent Review

Independent acceptance review (2026-10-09, reviewer workspace
`/root/mining_review`). Read-only on production, tests, shared notes and
processes; owned artifacts only: this note and
`lab/reports/mining-score-review-20261009T121417Z/`.

## Reviewed revision (frozen; hashes verified in-session)

| Path | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/gameplay.rs` | `077ef90684d026f5824062065449630fce71789c94ece41243485253290ef9c5` |
| `prototype/osfr-tools/src/lib.rs` | `376f936ad4b8f5272d23df9848084db475332562f8c8390c957c156c45a14eb8` |
| `prototype/osfr-tools/src/mining.rs` | `a66073ca463d84dc6e293a5ee3c8eef8ca861a40d23d7a6eb4acd0e4a0d6e4e2` |
| `prototype/osfr-tools/tests/type8_gateway.rs` | `e1aec6e08eebe0db334ab4d2316534b562e2126fb276689cd4ca763f554af9cd` |

Evidence read: `notes/Research/Type8 Mining Gateway Integration.md` and the
private bundle `lab/reports/type8-gateway-20261009T120421Z/` (RED/GREEN logs,
regression logs, `type8-content/mining.json`).

## Verdict

The dev-tools-only gateway integration is correct within its stated bounds:
optional state is backward compatible, content binding and namespace checks
fail closed, only the accepted Miner job can start a run, a live encounter or
crafting movie blocks entry and an active run blocks unrelated frames, the
server clock owns the timer and results, and rewards exist only as the
engine's server-derived transaction from configured `outputs` (empty by
default). No material finding. The reconstructed board, quota and output
mapping remain labelled reconstructions, and no native/visual acceptance is
claimed here.

## Independently verified

Private probe crate `lab/reports/mining-score-review-20261009T121417Z/probe`
(`tests/mining_probe.rs`), 6/6 passed, log `logs/01-probes.log`:

1. `review_optional_state_absent_and_contentless_snapshot` — a legacy
   `GameplayState` without `mining` deserializes, stays `None` and re-serializes
   byte-identically; a campaign without `mining.json` serializes no `mining`
   key; a mining snapshot without its content file fails closed with
   "mining snapshot requires its content".
2. `review_content_binding_and_namespace_fail_closed` — `game_id 91` is
   refused ("game binding mismatch"), a non-Miner `profile_id` is refused
   ("does not bind the Miner job"), and a state below the mining instance
   namespace is refused ("outside its instance namespace").
3. `review_devtools_miner_gate_and_exclusive_activity` — dev tools off refuses
   with the explicit message and no state change; a non-Miner active job and a
   locked Miner each refuse with their own message; the accepted entry starts
   instance `18_000_001` with the `39/16` info packet; a second `realm mine`
   keeps the same instance. A live cave encounter refuses mining
   ("Finish or exit the active activity"); `realm exit` releases it,
   re-selecting Miner then starts the mining run.
4. `review_client_reports_never_advance_or_grant` — with no live run, `39/5`,
   `39/14` payloads, win/score reports, `26/42`, `39/6` and `39/7` are inert
   with no state change; during a live run, `FRServer_GameWon`,
   `FRServer_ScoreSet`, `FRServer_GameLost` and `FRServer_GameQuit` reports
   (including absurd values) and stale-instance frames change nothing; the
   phase stays `Offered`, `score` stays 0 and no item is granted. The engine
   refuses malformed payloads (`FRServer_ScoreSet\tnot-a-number`,
   `FRServer_Unknown\t1`, a 14-byte `39/5`) without mutating state.
5. `review_timer_authority_results_and_wall_bound` — client end reports cannot
   finish a live level; the server tick runs the 180 s level out to `Lost`,
   `FRServer_GameOver` emits `39/18` + `39/47` + `39/19` exactly once, and
   `FRServer_GameClose` releases the run; a paused run still loses at the
   30-minute wall cap.
6. `review_level_complete_transaction_is_server_derived_and_once` — with
   configured outputs, an `OnNextLevelMsg` at `LevelComplete` completes the
   run and returns the transaction derived from the collected quota
   (`3 clusters x 2 = 6` items); replaying it live, or after a save/reload
   round trip, grants nothing; zero collected clusters and the shipped default
   `outputs: []` grant nothing even at quota.

Focused existing tests re-run in a private target directory, all green
(`logs/02-focused-set.log`): `type8_gateway` 10, `encounter_score` 3,
`encounter` 15 (+2 ignored), `mining` 21, `mining_review` 13.

## Boundaries

- Entry is dev-only (`realm mine` with `--dev-tools`); no recovered world,
  quest or gather binding was invented.
- Board rectangle, quotas, spawn rates, seed and the output/reward mapping are
  reconstructed; `outputs` ships empty until the lead accepts a mapping.
- Client `39/14` payloads are ignored unparsed while the run is `Offered` (only
  `39/5` starts it); from `Launched` onward they are validated and refused on
  error. This is fail-safe, noted for completeness.
- Client pause can pause the timer, but the 30-minute wall bound forces the
  loss, so the client cannot extend a run indefinitely.
- No native/visual claim: the asset list, movie, chain input and results close
  remain the lead's native gates. The full suite (root: 1410 passed / 57
  ignored) was not re-run here; the focused set above was.
