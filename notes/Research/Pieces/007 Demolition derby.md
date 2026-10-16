# 007 Demolition derby

Queue item 007. Acceptance: recover original arena/vehicle/damage/scoring data and prevent duplicate
finish/reward.

Status 2026-10-09 02:55 UTC: wire schema, arena/vehicle/damage facts and the subtype-to-class mapping
recovered; a bounded bout contract with once-only finish and reward implemented and tested. Scoring
values stay an explicit gap. No native or gameplay acceptance was attempted.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/derby.rs` | Frame decoder, subtype/length table, vehicle-hit/config/announcer layouts, `DerbySession` |
| `prototype/osfr-tools/tests/derby.rs` | 11 tests; includes the module by `#[path]` until the lead registers it |
| `scripts/pieces/piece_007.py` | Read-only checker over the capture, packet enum, locale export and native class list |
| `tests/pieces/test_piece_007.py` | 12 stdlib tests: hermetic parsers/tampers plus real-corpus pins |
| `../lab/reports/piece-007-20261009T0248Z/` | RED/GREEN logs, `checker/derby-evidence.json`, `checker/provenance.json` |
| `notes/Research/Derby Restoration.md` | Full recovery note, reproduction commands and native gates |

## Recovered original facts

1. **Envelope and subtype map.** All 40,214 opcode-34 bodies of
   `packets_demoderby_minigame.pcapng` re-hash to `demoderby-application-index.json`; the 17
   captured subtypes each have a fixed payload-length set and the original class name from the
   unchanged `BaseVehicleDemolitionDerbyPackets` enum. Subtypes 8/9/17 never occur and stay opaque.
2. **Arena.** Locale ids 6678 (Thunder Falls) and 6705 (Frost Ridge) name the two derby venues;
   3053 is "Demo Derby Lobby". Tutorial text (id 116725, 393338) documents Health, damage and being
   wrecked; id 92695 is the objective "Destroy at least 5 cars".
3. **Vehicles.** Subtype 10 carries the proxied karts `sg_dm_crusher_body_bbec_002.cdt` and
   `sg_dm_ripper_body_bbec_002.cdt`; the first bout assignment uses a packed two-byte name length,
   later ones a `u32` length. Eight participants: the real player key plus seven proxied slots
   (`0x01400662` anchor, slots `0x10`-`0x70`); the player/NPC count frames read 1 and 7.
4. **Damage.** Subtype 16 is `attacker u64, victim u64, impact f32`. 54 of the 92 captured frames
   are proxied pairs (keys swapped, identical magnitude); 38 involve the real player and arrive
   once. The impact unit is unknown.
5. **Scoring is not in the wire.** The shipped UI reads `DerbyResults` columns place, guid, name,
   score, damage, and the native client carries `VehicleDemolitionDerbyResultsDataSource`, but no
   captured frame populates those values and the preserved server has no derby implementation.

## Reconstructed policy

`DerbySession` (local, not historical): lobby/running/finished/abandoned, at most eight unique
participants, hits only while running, and the acceptance-critical guards - the first valid result
table is recorded once (`FinishOutcome::Duplicate` afterwards) and `claim_reward` returns
`RewardOutcome::Granted` exactly once (`AlreadyClaimed` afterwards, `NoResults` before a finish or
after an abandon). The module moves no currency or items; the caller grants only on `Granted`.
Impact totals are a labelled tally, not a health model.

## Commands and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --test derby` before the implementation | RED: 3 compile errors in the shipped naive baseline |
| `scripts/cargo.ps1 test --test derby` after adding the new behaviour tests | RED: 10 errors, unresolved imports/items |
| `scripts/cargo.ps1 test --test derby` | GREEN: 11 passed, 0 failed |
| `scripts/cargo.ps1 test --lib --test derby` | GREEN: 24 + 11 passed |
| `rustfmt --edition 2024 --check src/derby.rs tests/derby.rs` | exit 0, empty output |
| `python scripts/pieces/piece_007.py --out <lab report>` | 12 checks pass, 0 problems, 40,214/40,214 bodies re-hashed |
| `python -m unittest discover -s tests/pieces -p "test_piece_007.py" -v` | GREEN: 12 passed |
| whole-workspace `cargo test` | blocked by other workers' in-flight targets (`piece_017`, `piece_019`, `piece_026`); none include the derby files |

## Open gates

- Native: enter a derby with the original client and verify proxied karts, hits, countdown and the
  result screen on the current release.
- Scoring/damage numbers and the packet that fills `DerbyResults`; health, elimination threshold,
  pickup effects and arena geometry; the meaning of the `SetDerbyConfig` pair (240.0, 940).
- Reward granting and derby/racer progression belong to queue items 016/041; this piece only
  refuses duplicates.
