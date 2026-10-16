# Native Combat Results v3

Independent finalized audit accepts the original win/loss score-page lifecycle for the two disclosed synthetic one-hit fixtures. It does not establish a second campaign dungeon completion, original balance or full asset closure. No source, save or native input was changed by this review.

| Run | Verified datagrams | Gateway CRCs | Result row | Held before actual Close |
| --- | ---: | ---: | --- | ---: |
| 163518 | 1876 | 1855 | YOU WIN; 4 defeated; 0 total | 59.547s |
| 163829 | 1882 | 1863 | TRY AGAIN; 3 defeated; 0 total | 59.063s |

Both final traces completely reassemble. Exact terminal order is39/18 GameOver,39/47 score rows,39/19 StateRemove, client39/7 acknowledgement, genuine41/109 `[0,1]` only after recorded Close, then world31/12. All three minigame headers are `[411,-1,411]`. Neither trace contains an earlier exit request or world load while the original results page is held open. The external input sidecars associate the only Close packet with the actual X click. Each live writer ends closed, complete and with exactly the authoritative manifest frame count.

The two 102-byte score payloads contain the original labels `scoreEnemiesDefeated` and `scoreTotalScore`, five fields each: `[-1,0,4,-1,0]` for win defeat count, `[-1,0,3,-1,0]` for loss defeat count, and `[-1,4,-1,-1,0]` for total. Original GameOver fields remain `[0,0,1]` after the won/lost boolean. The win movie also renders its local Goals1/1 row with5000; total remains0. This native display is recorded rather than treated as implemented combat scoring.

Independently viewed both score-page screenshots and both same-session journal screenshots. Defense is green and Pappy talk remains pending after return. The loss screenshot has a visible Gloam Overlord target-health bar overlapping the score title. Lifecycle acceptance leaves this cosmetic defect open.

Each run's character-before hash exactly matches its prepared v3 branch. Preserved083946 source, identity source/copy and content hashes match provenance; source-to-branch leaf differences are exactly the disclosed fixture whitelist. After play, inventory items, quests, opening stages, encounter rewarded set/world enemies/next instance and identity remain unchanged. Loss returns to zone0, health1000 and coins70. Win adds discovery2, whose reward explains coins70→75, and the actual new boss kill adds25 Brawler43 experience; these are explicitly recorded post-play changes, not repeated dungeon rewards. Both active encounters clear only on return. Harness manifests report all three services stopped.

Both runs retain22 unique asset404 paths, including scene tiles and boss spawn/despawn animation requests. The lifecycle proof does not imply those assets rendered or that all original dependencies are available. No retry input was exercised in these native runs; existing ordinary/private retry checks are separate evidence.

Reproducible audit: `lab/reports/verify-native-results-v3.py`. Aggregate `lab/reports/native-results-v3-independent-proof.json` SHA256 `352ff0de04b5e3a92e7009759c895c3ce3cd975dc4a7cd1c870ef8a0b9529bdb`. Per-run proofs are in each `analysis/native-results-v3-independent-proof.json`, win SHA256 `2ba44df04c770a904bfbf9c5400d49059d92ab04861f5cef99a0d066c30c4def`, loss SHA256 `0cc9d16e5d97573cd8d634890571960404ba8647f2d88246685c8e35b0c3cf80`.

The optional live capture's independent source review passed without blocking findings. Its separate isolated proof remains `lab/reports/native-live-capture-proof.json` SHA256 `34c613ab895d731c2901353f93f0f1f29bf15dc1a81a79ee963b4bf042711c36`; these two native runs additionally verify its final count/order/cleanup against authoritative manifests.

Follow-up read-only original target-bar research found the existing32/9 false cleanup route and verified that win sends it while loss does not. See [[../Research/Native Loss Boss Display]]. No cosmetic repair or native reacceptance is claimed yet.
