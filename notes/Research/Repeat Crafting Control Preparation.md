# Repeat crafting control preparation

October 8, 2026. Read-only preparation for the local repeat crafting stages: stage 13 Cooking at Mae, stage 14 mining, and stage 15 Smelting/Forging at Cleetus and the moved anvil. No UI input, no client control, no save/content/source change by this review. This is a control handoff, not an acceptance claim.

## Current window and base geometry

The current run `client-baseline-20261008T235041Z` recorded its own client rect: `external-inputs.jsonl` record 1 (`focus | rect | fg`) answers `ok / 192 41 1536 1152 / FreeRealms`. The client area is **1536x1152** and the GPU half-shot is **768x576** (captures in that run are 1536x1152 and 768x576).

Every prior input-logged run (13 runs, including all five crafting runs) began with `size 1280 960`. All accepted crafting coordinates in the notes below are **physical client pixels at 1280x960**.

| Mapping | Formula | Evidence |
| --- | --- | --- |
| A, uniform 1.2 (primary) | `client_1536 = client_1280 x 1.2`, `half_shot = client_1280 x 0.6` | measured on the main HUD between a prior 1279x960 capture and the current 1536x1152 capture: energy bar 682x77 to 821x93 (1.204/1.208), Power Hour 344x44 to 413x53 (1.201); the run's own login/News clicks (934,896) and (1456,66) match prior (780,747) and (1208,54) x1.2 |
| B, fixed movie centred (fallback) | `client_1536 = client_1280 + (128, 96)`, `half_shot = client_1280 x 0.5 + (64, 48)` | if a minigame SWF keeps its authored viewport (860x860 for Cooking/Smelting, 727.86x860 for Forging) instead of scaling with the client |

**Calibration gate before the first input in each movie.** Take one screenshot and check a landmark: Cooking Measure bucket registration is (771.2,393.2) under A but (770.7,423.7) under B in client pixels; Forging board cell (3,3) centre is (596.4,288) with pitch 52.2 under A but (625,336) with pitch 43.5 under B. If the screenshot matches A, use A everywhere; if B, use B. Do not mix. As a zero-uncertainty alternative, the lead can restore the accepted base first:

```powershell
& 'lab\tools\universal-modder-env\Scripts\python.exe' 'project\scripts\drive-local-game.py' --run 'lab\runs\client-baseline-20261008T235041Z' "focus" "scanmode on" "size 1280 960"
```

and then use the base columns verbatim (the GPU half-shot becomes about 639x480, as in the prior runs).

## Cooking (stage 13) - accepted 164218/171251

Six ordered rounds; each completion panel needs one click: green arrow `(864,780)` base = `(1036.8,936)` A = `(518.4,468)` half, and/or centre confirm `(600,500)` base = `(720,600)` A = `(360,300)` half.

| Round | Accepted control | Base (1280x960) | Half-shot 768 (A) |
| --- | --- | --- | --- |
| 1 Tenderize Bat Meat | click cycle over moving meat while the hammer is idle | (522,521) (727,529) (445,587) (399,658) (637,597) (639,668) (862,667) (813,598) | (313.2,312.6) (436.2,317.4) (267,352.2) (239.4,394.8) (382.2,358.2) (383.4,400.8) (517.2,400.2) (487.8,358.8) |
| 2 Chop Rainleaf | click the visible knife | (725,610) repeated | (435,366) |
| 3 Smash Gummyfruit | click each fruit while the hammer is idle | (564,588) and (720,588) | (338.4,352.8) and (432,352.8) |
| 4 Measure Brown Broth | hold, lift until the target line, release | press (600,650), lift to (600,490) | press (360,390), lift to (360,294) |
| 5 Add ingredients | drag the requested item into the pot, order Gummyfruit, Rainleaf, Bat Meat | drop (643,521); starts randomized (164218: 650,210 / 768,287 / 991,288; 171251: 876,622 / 888,426 / 1000,196) | drop (385.8,312.6) |
| 6 Stir MeatStew | press in the bowl and sweep 10 counterclockwise loops | press (693,534), centre (626,534), radius 67 | press (415.8,320.4), centre (375.6,320.4), radius 40.2 |
| Results | actual Close X on the score panel | (1140,76) | (684,45.6) |

Phase screenshots to retain: entry/offer, each round start and completion, result total held, closed world frame, then Mae menu + turn-in + restart.

## Smelting (stage 15, first half) - accepted 173814

Four phases in order, each with an advance click `(864,780)` base = `(518.4,468)` half: break ore, fill furnace, heat furnace, mould metal. `SmeltingGameClient.as` lines 422-435 map 101 BreakDownOre, 102 RoundPour(_bPowder=true), 103 HeatFurnace, 104 RoundPour(_bPowder=false); `RoundPour extends RoundMeasure`, so the mould uses the same hold-and-lift gesture as the fill.

| Phase | Accepted control | Base | Half-shot 768 (A) |
| --- | --- | --- | --- |
| 101 Break ore | many clicks across the ore pile, waiting for the smash animation | recorded clicks span x432-852, y604-752 (full list in `controls-handoff.json`) | span x259.2-511.2, y362.4-451.2 |
| 102 Fill furnace | hold, lift to tip the bucket, release before overshooting, let particles land | press (503,318), lift (503,186) | press (301.8,190.8), lift (301.8,111.6) |
| 103 Heat furnace | move back and forth to work the bellows | (843,515) to (757,534) | (505.8,309) to (454.2,320.4) |
| 104 Mould metal | source-proven same gesture as fill; **not recorded in the accepted helper log** | no recorded base; use the fill gesture region | no recorded value |

Gap to respect: the accepted run's helper log contains no phase-104 input even though round 104 completed. Treat the mould control as documented-from-source only, and capture a fresh screenshot before pouring.

## Forging (stage 15, second half) - accepted 173814/181715

Board: hold left at a tile, drag through at least three orthogonally adjacent same-colour idle tiles, release at the last; wait for the explosion, gravity and refill; hammers are not chain colours and must be cleared from below; local quota is 5 hammers in 180 seconds (reconstructed).

```powershell
& 'lab\tools\universal-modder-env\Scripts\python.exe' 'lab\tools\plan-forging-chain.py' '<fresh snapshot.json>' --limit 1 --anchor <X> <Y> --step <DX> <DY>
```

The planner reads `gameplay.forging.active` (14x14 board, idle flags, Playing phase) and returns legal chains only; it sends no input and must be re-run after every settled chain.

| Mapping | Anchor cell (3,3) | Step |
| --- | --- | --- |
| A (uniform 1.2, observed 173814 anchor) | (596.4,288) client = (298.2,144) half | (52.2,52.2) client = (26.1,26.1) half |
| A with static-guide anchor | (590.69,279.03) client = (295.35,139.52) half | (53.75,53.75) client = (26.875,26.875) half |
| B (fixed movie) | (625,336) client = (312.5,168) half | (43.5,43.5) client = (21.75,21.75) half |

A recorded accepted chain (record 8 of 181715, base): (671,370) to (671,414) to (671,458) to (714,458) to (714,501) to (714,544) to (671,544); half-shot A: (402.6,222) (402.6,248.4) (402.6,274.8) (428.4,274.8) (428.4,300.6) (428.4,326.4) (402.6,326.4). The recorded chains are gesture evidence only; the board is randomized per attempt, so plan from the current snapshot.

Results: after the quota the original bin/mold animation runs, then the delayed GameOver and the score page appear; the actual Close is the X at base `(1140,76)` = half `(684,45.6)` (observed closing both the Cooking and Forging result pages).

## Helper commands used by the accepted runs

```powershell
# one recorded input batch (coordinates are physical client pixels)
& 'lab\tools\universal-modder-env\Scripts\python.exe' 'project\scripts\drive-local-game.py' --run '<lab run dir>' "focus" "scanmode on" "move X Y" "mdown X Y" "move X2 Y2" "mup"
# navigation between the farm actors
& 'lab\tools\universal-modder-env\Scripts\python.exe' 'lab\tools\navigate-local-game.py' --run '<lab run dir>' --state '<lab state file>' --target <X> <Z> --expected-zone 0
```

Evidence: `../../../lab/reports/repeat-crafting-control-prep-20261009T001532Z/` (`proof.json`, `controls-handoff.json`, `accepted-controls-extraction.json`, `ui-scale-measurement*.json`, `measure-ui-scale.py`, `extract-crafting-controls.py`, `build-handoff-tables.py`, `crops/`). Prior accepted logs: `client-baseline-20261008T164218Z`, `T171251Z`, `T173814Z`, `T181715Z`.

See [[Native Cooking Controls]], [[Original Cooking Later Controls]], [[Native Smelting Controls]], [[Native Forging Controls]], [[Forging Native Control Geometry]], and [[../Current Progress]].
