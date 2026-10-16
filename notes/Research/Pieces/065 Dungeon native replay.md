# 065 Dungeon native replay

Work item 065 owns the native acceptance question: *did one original client run enter the
cave from the world, fight the waves, kill the boss, receive the reward once, exit and
reconnect?* This note records the replay contract, the decoder, the closed gates and the
exact evidence that is still missing. The item is **not** accepted: no preserved run
satisfies all six gates in one capture.

## What this piece ships

| Artifact | Purpose |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_065.rs` | Recovered wire table plus the transcript schema and the six reconstructed gate rules (`evaluate`). |
| `prototype/osfr-tools/tests/piece_065.rs` | Contract tests: wire rows, one synthetic reference replay, one negative case per gate, and grading of the preserved transcripts. |
| `scripts/pieces/piece_065.py` | Decodes ended native runs into `replay-transcripts/*.json` and re-derives every wire row from its pinned source. |
| `tests/pieces/test_piece_065.py` | Parser tests, one end-to-end real-corpus decode, one drift negative control. |
| `lab/reports/piece-065-20261009T053857Z/` | Private bundle: transcripts, facts, provenance and `summary.md`. |

The module is not registered in `src/lib.rs` (lead-owned). The owned test uses
`#[path = "../src/restoration/piece_065.rs"]`, so the real source file is what compiles.
Integration step for the lead: register the module and drop the `#[path]` include.

## Recovered wire contract

Every row below is re-derived by the checker from the artifact named in the row's `source`
field; a mismatch or a missing hash fails the run.

| opcode/sub | packet | direction | source |
| --- | --- | --- | --- |
| 41/114 | `EncounterDetailsResponsePacket` | S2C | Sanctuary combat PR 76 pinned source |
| 41/107 | `EncounterZoneIsReadyPacket` | S2C | Sanctuary combat PR 76 pinned source |
| 41/106 | `EncounterStatePacket` | S2C | Sanctuary combat PR 76 pinned source |
| 41/2 | `EncounterPacketPlayerEnter` | S2C | Sanctuary combat PR 76 pinned source |
| 41/133 | `EncounterPacketIsFighting` | S2C | Sanctuary combat PR 76 pinned source |
| 41/109 | `EncounterParticipantRequestExitPacket` | C2S | `client-baseline-20261008T154226Z` frame 1403, exact body `29006d000000000001000000` |
| 39/17, 39/18, 39/19, 39/47 | minigame start / result / remove / score rows | S2C | Sanctuary combat PR 76 pinned sources |
| 32/7 | `CombatPacketAttackProcessed` | S2C | Sanctuary combat PR 77 pinned source |
| 32/9 | `CombatPacketEnableBossDisplay` | S2C | Sanctuary combat PR 77 pinned source |
| 35/5 | `PlayerUpdatePacketUpdateHitpoints` | S2C | Sanctuary combat PR 74 pinned source |
| 26/10 | encounter interaction selection (101 exit, 103 attack, 104 start) | C2S | `client-baseline-20261008T081219Z` |
| 125 | standalone movement | S2C | `client-baseline-20261008T081219Z` |
| 38/1 | player health | S2C | `client-baseline-20261008T081219Z` |
| 31 | zone transition | S2C | `client-baseline-20261008T073831Z` |
| 10, 13 | `ClientFinishedLoading`, `ClientIsReady` | C2S | pinned Sanctuary tree |

The pinned community files live under `lab/references/Sanctuary-combat-pr-research`
(PR 74/76/77) and `lab/references/Sanctuary`; their SHA-256 values are checked against
`SOURCE.json`. The full table with hashes is written to the bundle's `provenance.json`.

## Recovered native flow order

The one preserved world-to-cave entry (`client-baseline-20261008T073831Z`) shows the order
the rebuilt server must produce:

| time | frame | row |
| --- | --- | --- |
| 105.344s | 2061 | server `31` zone transition to zone 411 |
| 106.750s | 2408 | `41/114` launch details (instance 1) |
| 106.750s | 2410 | `41/107` zone ready |
| 106.750s | 2412 | `41/2` player enter |
| 106.750s | 2427 | `32/9` boss display on for the 500 HP enemy |
| 107.688s–107.703s | 2441–2445 | `125` enemy movement for all four enemies |
| 107.703s | 2446 | `38/1` player at 980/1000 |
| 115.657s | 2536 | first voluntary client `26/10` attack (choice 103) |

The same run is the documented failed acceptance: the enemies engage and the player is
already at 500/1000 HP (frame 2530) before the first voluntary attack at 115.657s, and the
run ends in a loss at 121.813s (session 1) and 167.391s (session 2).

## Gates (reconstructed rules)

The transcript schema and the six rules are reconstructed; the rows they read are
recovered. A gate fails on contradictory rows and reports `NotEvidenced` when the rows are
absent, so a missing capture cannot pass.

1. **Fresh entry** – the run starts in the world, zones into the cave, then launch → zone
   ready → player enter; no player damage and no enemy movement before the first voluntary
   client selection (start 104 or attack 103).
2. **Combat** – at least one processed player attack damaged an enemy, at least one enemy
   movement row exists, and the player is still standing at the terminal result.
3. **Boss** – exactly one tallest enemy, a `32/9` display row for it, and 0 HP before the
   terminal result.
4. **Rewards** – the terminal `39/18` result is a win, every enemy is at 0 HP, the
   `39/47` score rows report the real `scoreEnemiesDefeated` count, and the encounter id
   [411] is in the persisted rewarded set.
5. **Exit** – a genuine client close (`41/109` or guide exit 101) follows the results, then
   a world zone transition, and the persisted session ends in zone 0 with no active
   encounter.
6. **Reconnect** – a later session for the same identity fingerprint starts in the world
   with the encounter cleared and the rewarded set unchanged.

`an_earlier_loss_does_not_invalidate_the_winning_attempt` records the retry rule: gates
2–4 grade the last `39/18` result, which is what the client left the encounter with.

## Grading the preserved runs

`scripts/cargo.ps1 test --test piece_065 -- --nocapture` grades every transcript in the
newest bundle. Nothing is accepted:

| run | entry | combat | boss | rewards | exit | reconnect |
| --- | --- | --- | --- | --- | --- | --- |
| 073831 world entry | fail (damage before voluntary attack) | fail (defeated) | fail (500 HP at end) | fail (loss) | fail (no close) | not evidenced |
| 075754 motion retest | fail (started in cave) | not evidenced (no player attack) | fail | fail (loss) | fail | not evidenced |
| 081219 in-cave win | fail (started in cave) | pass | pass | fail (no `39/47` rows) | pass | not evidenced |
| 091836 win fixture | fail (started in cave) | fail (no enemy motion) | pass | fail (no `39/47` rows) | pass | not evidenced |
| 092345 loss fixture | fail (started in cave) | fail (no enemy damage) | fail (400 HP at end) | fail (loss) | pass | not evidenced |
| 154226 results v2 | fail (started in cave) | fail (no enemy motion) | pass | fail (no `39/47` rows) | pass | not evidenced |

The `<run>.reconnect.json` transcripts append the later same-character session
`client-baseline-20261009T013155Z` (identity `12cf0d44…`, zone 0, rewarded `[411]`). With
that session the reconnect gate passes for the runs that recorded the reward themselves
and stays `NotEvidenced` for the two runs that never earned it. The gate never passes a run
whose rewarded session is the last one.

## Smallest missing evidence

One fresh lead-driven run has to produce, in a single capture:

1. world start (zone 0) with `31 → 41/114 → 41/107 → 41/2` and no damage or enemy motion
   before the first voluntary `26/10` selection;
2. player-hit damage, enemy movement and all four enemies at 0 HP including the 500 HP
   boss with `32/9` display on;
3. the repaired results lifecycle: `39/18` win → `39/47` with
   `scoreEnemiesDefeated = 4` → `39/19` → genuine `41/109` or guide exit;
4. world return zoning to zone 0 with the encounter cleared and `rewarded = [411]` once;
5. a second session of the same character, or a `--connect-run` session supplied to the
   checker, with the reward set unchanged.

Until that exists, 081219 remains the strongest completion evidence (combat, boss, exit
and once-only reward) and 073831 remains the strongest entry evidence; neither run closes
the acceptance by itself. Original dungeon scripting, waves, balance, boss mechanics and
reward tables stay with items 059–063 and are not reconstructed here.

## Commands

```powershell
# decode the preserved runs into the private bundle
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_065.py' `
  --connect-run client-baseline-20261009T013155Z

# grade every transcript (prints one line per run with --nocapture)
& 'scripts\cargo.ps1' test --test piece_065

# checker contract tests
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_065.py" -v
```

## Limits

* The zone id 411 is a local assignment; only the encounter id 411 has supplied locale
  support (`notes/Research/Starting Dungeon Audit.md`).
* Combat is reconstructed; the balance, timing, damage values and the once-only 50-coin
  reward are local policy, not recovered original behavior.
* 091836/154226 are one-hit synthetic health fixtures. Their passing boss/exit gates prove
  the rules read the wire, not that the original encounter was replayed.
* The checker reads captures only. It sends no packet, launches nothing and writes only
  its own report directory.
* The capture format depends on `analysis/gateway-decode-proof.json`; runs recorded without
  it (for example 20261009T013155Z) cannot be decoded and are reported as skipped.
