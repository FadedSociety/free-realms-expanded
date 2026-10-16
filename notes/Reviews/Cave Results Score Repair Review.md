# Cave Results Score Repair Review

Independent acceptance review (2026-10-09, reviewer workspace
`/root/mining_review`). Read-only on production, tests, shared notes and
processes; owned artifacts only: this note and
`lab/reports/mining-score-review-20261009T121417Z/`.

## Reviewed revision (frozen; hashes verified in-session)

| Path | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/encounter.rs` | `68cfc114b65ae9f8648ef94bf3a566789f772a988daba87c2c02c2cc2de4e65e` |
| `prototype/osfr-tools/tests/encounter_score.rs` | `9c0123feebda5efb46e60a0102e3a4586f85256de0cd3f453fc7be3c5a77e232` |

Evidence read: `notes/Research/Cave Results Score Binding.md` (the read-only
native diagnosis: TOTAL SCORE is the fifth i32, Points, of the `39/47` row
whose Order/Score-Type field is 4) and the private bundle
`lab/reports/cave-score-coherence-20261009T124500Z/` (RED/GREEN logs,
`score-coherence-proof.json`, v4 oracle logs).

## Verdict

The repair is correct and minimal. `OBJECTIVE_AWARD_SCORE: i32 = 5000`
(`encounter.rs:22`) now feeds both `objective_complete` and the Order-4 total
row in `end_scores`: a won run reports 5000, a loss reports 0, the
enemies-defeated row keeps its actual count and zero points, and every other
field in the packet is untouched. No kill points, historical score, reward,
XP or save field was added. No material finding.

## Independently verified

Private probe crate `lab/reports/mining-score-review-20261009T121417Z/probe`
(`tests/score_probe.rs`), 3/3 passed, log `logs/01-probes.log`:

1. `review_win_loss_total_rows_and_wire_bytes` — a real four-kill win produces
   `scoreEnemiesDefeated [-1,0,4,-1,0]` and `scoreTotalScore [-1,4,-1,-1,5000]`;
   a three-kill loss produces `[-1,0,3,-1,0]` and `[-1,4,-1,-1,0]`. Each
   produced packet is byte-identical to the independently compiled v4 oracle
   vector with only the defeat-count i32 patched to the observed count
   (`end-scores-win-state411.bin` + 4, `end-scores-loss-state411.bin` + 3), and
   stays the recovered 102-byte size.
2. `review_v3_v4_vectors_manifests_and_exact_delta` — both bundles verify
   against their own manifests; every vector is byte-identical between v3 and
   v4 except `end-scores-win-state411.bin`, whose last i32 changes from
   `00000000` to `88130000` (5000) with all other bytes equal. v4 win sha256
   `58675e1da6af145bbd8808b2f1c5298b67d4601253b45a475b0a34807ebda838`,
   v4 loss sha256
   `8e69efeb6e47f842a269cd91a6f6246e5dd3489d01d3164de535fd2262cf4820`.
   The original zero-total v3 bundle is preserved unmodified.
3. `review_score_change_does_not_alter_rewards_or_schema` — the winning blow
   still yields exactly the dungeon reward (50 coins, item 1195 x1); presenting
   the results again grants nothing, presents no second `39/47` and leaves the
   encounter state value-identical; the persisted `EncounterState` still has
   exactly its eight pre-existing fields (no score field); a loss grants
   nothing.

Focused existing tests re-run in a private target directory, all green
(`logs/02-focused-set.log`): `encounter_score` 3, `encounter` 15 (+2 ignored);
and the ignored private oracle test with the versioned v4 bundle passes
(`logs/03-v4-oracle.log`, 1 passed).

## Boundaries

- The 5000 is the existing local objective award, an explicitly reconstructed
  policy; the original encounter-411 score table and points remain
  unrecovered, and no per-kill points or historical total are invented.
- The repair changes only the Order-4 row's Points i32; the loss row keeps 0
  and the enemies row keeps Points 0.
- No reward/XP/save path is involved: the score is presentation-only and does
  not feed `EncounterOutcome::rewards` or any persisted field.
- Native visual confirmation of the rendered panel remains the lead's sole-UI
  gate; this review made no UI or process claim.
