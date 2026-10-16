# Activity coverage receipt review (piece 015)

2026-10-09. **Two inaccuracies: the Smelting77 record contradicts its own capture, and the removed "TYPE 9 Mimic anvil" wording is still shipped in TYPE 8 slot evidence. The three row→source-id bindings, the receipt rule application for 46/139, and catalogue reproducibility otherwise hold.**

Reviewer read only the audit script, the committed catalogue, the four shipped tables, the run receipts and the focused tests. Nothing outside `notes/Reviews/Activity Coverage Receipt Review.md` and `lab/reports/activity-coverage-receipt-review-20261009T103810Z/` was written.

## Finding 1 — Smelting77 record contradicts its captured receipt (material)

`findings.native_acceptance.accepted_rows["77"]` records `kinds: [entry, completion]` and the caveat "no stage5 reward/turn-in was captured; material consumption and the 10-bar output remain a fixed reconstructed recipe". The 173814 capture contains all of it:

| Frames | Packet | Meaning |
| --- | --- | --- |
| 8850 | `38/4` instance 22 | ore stack fully consumed (5 Ore → 0) |
| 8851 | `38/3` instance 23 count 2 | coal stack 5 → 2 |
| 8852 | `38/2` definition 21785 ×10 | Copper Bars ×10 granted |
| 9086 → 9090 | `26/10` id 5, then `38/19` | Cleetus turn-in; coins 120 → 130 |
| 9091 | `38/14` (14,1,25) | Miner XP 25 |
| 9092 / 9093 | `49/4` quest 910005 / `49/12` value 5 | stage 5 completed and rewarded |

The project's own note `notes/Reviews/Native Gear Up and Smelting 173814.md` already states "Cleetus interaction 9086 turns Gear Up in; wallet updates once to 130 and Miner XP once to 25", so the catalogue contradicts its cited evidence. Correction: set `kinds: [entry, completion, reward]`, and rewrite the caveat so it says which *rule values* stay reconstructed (5 ore + 3 coal → 10 bars, and the reward mapping itself, since `MiniGameData.txt` has no reward column) instead of claiming the receipt is absent. The same caveat is duplicated in `scripts/pieces/piece_015.py` (lines 269–271) and must be corrected with it.

## Finding 2 — stale "TYPE 9 Mimic anvil" wording survives in TYPE 8 slot evidence (material)

`scripts/audit-minigame-coverage.py` lines 299–303 (rendered as `types["8"].slot_evidence["forging-boards"]`) still read: "MiniGameData rows named 'Forging: <item>' under TYPE 8 (MatchThree), **distinct from the native-accepted TYPE 9 Mimic anvil**; no packet evidence that they share a family". This contradicts `findings.forging_row_139_binding.resolution` ("the earlier 'TYPE 9 Mimic anvil' wording was wrong and is removed") and `MiniGameData.txt` (`139^8^386081^22054^910^…`). TYPE 9's family is cooking/smelting/music/sword. Correction: replace that evidence string with the TYPE 8 statement (row 139 is the only receipt-bound forging row; the local server-owned MatchThree model drives it; the original service engine class and the other 60 forging rows remain unreceipted/unproven). Focused test gap: `tests/test_minigame_coverage.py::test_forging_139_is_type8_and_not_described_as_mimic_type9` checks only TYPE 9's `family`/`family_evidence`/`restoration_evidence`, so the TYPE 8 slot string passes unnoticed; extend the test to cover it.

## Verified — source-row bindings, entry/completion/reward

Every row's `39/16` info packet carries the source row's `NAME_ID` and the row's game id in the header, and the instance id is prefixed by the row id, so each capture binds to the exact `MiniGameData.txt` row (not a family):

| Row | Source row | Instance | INFO NAME_ID | Entry | Completion | Score | Close |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 46 | TYPE 9, NAME_ID 386023 | 46000002 | 386023 ✓ | frame 1560 | frame 3873 | 5133 (elapsed 101) | 3927 |
| 77 | TYPE 9, NAME_ID 386068 | 77000001 | 386068 ✓ | frame 2614 | frame 8855 | 5497 (elapsed 448) | 8935 |
| 139 | TYPE 8, NAME_ID 386081 | 139000002 | 386081 ✓ | frame 1455 | frame 2943 | 13100 | 3010 |

Rewards captured per row: 46 → 4×`38/4` consumes (instances 15/18/17/16), `38/2` definition 35 ×1, then Mae turn-in `38/19` 120 / `38/2` 14794 / `38/14` Chef 25 / `49/4` 910004 / `49/12` value 4 (frames 3866–3870, 4068–4072). 139 → `38/3` bars 20→10, `38/4` instances 31–33 removed, `38/2` definition 30280 ×1, then Cleetus turn-in `38/19` 146 / `38/14` profile 16 XP 25 / `49/4` 910006 / `49/12` value 6 (frames 2901–2905, 3203–3206). 77 → see Finding 1. The recorded content recipes match the granted definitions exactly (`cooking.json` 35, `smelting.json` 21785, `forging.json` 30280).

## Verified — local repeat ids vs source rows, and non-inference

Local `recipe_id` 2001/2002/2003 are content-side trigger ids (`opening.json` actors 13/15/16 and stage objectives 91000406 / 91000504 / 91000602 / 91001304 / 91001502 / 91001504; `src/gameplay.rs`; `src/restoration/piece_018.rs` ROUTES), bound to the source rows only through the local modules (`cooking.rs` GAME 46, `smelting.rs` GAME 77, `forging.rs` GAME 139). The catalogue keys acceptance on the source ids and never equates them with 2001/2002/2003; the row caveats disclose the reconstructed content, so no family inference is being claimed here. Two residual inconsistencies outside the audit script: `src/restoration/piece_018.rs:156` still calls the forging entry "TYPE 9 mimic anvil movie", and `types["9"].restoration_status` is `native-accepted` for a 54-row family with 2 accepted rows (the record does carry `native_accepted_rows`/`family_gate.scope`, so this is a presentation risk; renaming the type-level field to `strongest_row_status` would remove it).

## Commands and results

```
python -m unittest -v tests.test_minigame_coverage
  -> 20 tests OK (log: focused-audit-tests.log)

python scripts/audit-minigame-coverage.py --out <private>/regenerated-catalogue.json \
    --reports-dir <private>/regenerated-evidence --stamp 20261009T062252Z
  -> games=783 flash=360 native=417 unknown=6
  -> regenerated catalogue SHA256 1886c28ba511de9b39ec64e3839d8cb56d520dd6478aced06f7c5a4e45e7ac5b
     = committed notes/Research/original-activity-coverage.json, byte-identical

python verify_rows.py / inspect_rewards.py / inspect_receipts.py
  -> row-verification.json, receipt-packets.txt, reward-packets.txt (raw frames above)
```

## Evidence

- `lab/reports/activity-coverage-receipt-review-20261009T103810Z/row-verification.json` — per-row binding, entry/completion/close frames and parsed 39/18 fields.
- `.../reward-packets.txt` — every op38/49 packet in each row's window, including the 77 turn-in frames 9090–9093.
- `.../receipt-packets.txt` — minigame-scoped packets per capture; shows why the wire snapshots alone hide the rewards.
- `.../regenerated-catalogue.json` + `regenerated-evidence/` — private, byte-identical regeneration.
- `.../focused-audit-tests.log`, `.../review-findings.json`, `.../artifact-hashes.json`.

Reviewer changed no script, catalogue, test, save, UI or shared lab file.
