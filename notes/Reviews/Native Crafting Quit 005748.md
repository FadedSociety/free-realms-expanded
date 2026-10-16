# Native crafting Quit005748

Lead acceptance of the narrow Cooking Playing Quit gate. This is a new audit of the finalized capture; the earlier delegated review stopped after a partial prefix scan. Original server response remains a reconstructed local remove-only cancellation policy. This does not establish Smelting/Forging native Quit acceptance or a fresh Cooking win.

## Accepted behavior

Original client instance46000004 emits the exact15-byte current-instance Quit39/6 request `27000684e7bd02ffffffffffffffff` in frame009416 at1820.422s. Rust responds39/19 `27001384e7bd02ffffffff2e000000` in009418 at1820.469s. Client39/7 ACK follows in009420 at1820.531s. No subsequent server39/18 or39/47 reward/result message appears before shutdown.

The actual titlebar X and Leave actions were sent in one guarded input batch at01:28:12UTC: `focus`, `click42028`, `move64028`, `rect`, `click64028`. The recovered original titlebar hides the confirmation after3seconds, which explains why widely separated inspection/click attempts failed. No UI patch was needed. A transient TryAgain animation is visible immediately after removal. Root extracted and inspected the final recording frame at10seconds before its end: it shows the farm world and CookingTable interaction prompt restored.

The immutable `cooking-playing-exit-before.json` records Playing instance46000004. `cooking-world-restored-state.json` differs only by clearing `gameplay.cooking.active`; the whole character, inventory, quests, profiles, encounter, other crafting state, next instance46000005 and completed count1 are unchanged. `character-after.json` equals that world-restored state. Ingredients/output/quest progress were not consumed or granted by Quit.

## Offer Cancel correction

The earlier offer cancellation uses26/42 `1a002a00` in002787 at984.641s, followed by removal of instance46000003 in002789 at984.672s. A later second26/42 follows after removal. Client39/6 at985.656s carries instance0 and is stale. This proves the existing offer-close path, **not Offered current-instance Quit39/6**. Offered Quit remains code-tested without this native gate.

## Evidence and reproducibility

Private bundle: `../lab/reports/crafting-quit-native-005748-20261009T011957Z/`. Run `final-quit-audit.py` with the private Universal Modder Python. The first audit correctly failed on a mistaken assumption of one26/42 in the cancellation window; inspection found two, and the final check requires their exact ordering around removal. The final script exits0 and writes `final-quit-audit.json`, including source/script/state/proof/frame hashes and actual input receipt.

The audit consumes the existing finalized complete reassembly proof in `../lab/reports/native-devteleport-005748-final-20261009T013314Z/`:9524 total datagrams,9507gateway,9505validatedCRCs,26prior checks all passing. It does not rerun the full decoder. Release binding is `e1b9f8de…`; the refined release `ec43e1de…` changes teleport placement, not crafting Quit.

Video extraction command uses installed FFmpeg with `-sseof -10 -frames:v1 -vf scale=640:-1`, output `quit-final-video.png`. Earlier screenshot `cooking-playing-quit-world_small.png` misleadingly contains the still-running movie, and `cooking-cancel-world-final_small.png` belongs to the earlier offer cancellation. Neither was treated as Playing Quit visual proof.

## Follow-up013155

Fresh restart preserved completed count/materials and offered instance46000005. the owner manually played all six rounds and reported that Cooking works well. The final013155 snapshot contains all six round statistics, score4700, phasePlaying, no final GameStats, completed1. Client exited0 before final confirmation persisted. This is human playtest feedback with partial protocol completion, not a new stew award or stage13 completion. Continue a fresh instance through the actual final confirmation and turn-in.
