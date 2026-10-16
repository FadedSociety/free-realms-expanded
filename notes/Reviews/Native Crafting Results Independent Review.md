# Native crafting results independent review

2026-10-08. Review of the stable Cooking, Smelting and Forging terminal presentation repair. **No blocking findings. Fresh native results, actual Close, Mae turn-in and restart remain pending.**

## Original behavior and reviewed boundary

The original Cooking callback calls `GameOver`. Native GameOver39/18 stores the terminal status, stars, elapsed seconds and total score. Remove39/19 invokes native leave/presentation and emits automatic ACK39/7. The ACK is not a player action and must not release the retained terminal server state. A nonempty score row is required for the original score page; `scoreTotalScore` uses type4, and Lua reads total points from its column5. The original Cooking UI selects score-screen type9.

The original `Ui.LeaveMiniGameOrEncounter` binding and native path end in `CommandPacketClosedMinigameEndScreen`, serialized as exactly four bytes `1a002a00` (26/42). The constructor's extra zero argument is not written by its header-only writer. Conditional encounter41/109 is a separate branch; runtime Cooking encounter-flag mapping is not claimed here.

Independently verified all hashes in the two source proofs, their original pseudocode and AS2/Lua statements,251 original executable instructions with Capstone, both method-table bindings and additional constructor/send instructions. Direct machine checks confirm opcode26, command42 and the dedicated sender's call to the two-short writer.

## Accepted source behavior

- Cooking/Smelting emit their original finish callback,39/18 with zero stars and actual accepted elapsed/score,39/47 with one actual total-score row, then39/19. Forging retains its original delayed `FRServer_GameOver` boundary.
- Terminal Won/Lost state and gameplay interaction gates remain until actual Close. Recognized malformed packet lengths reject; well-formed removal ACKs, including stale or wrong headers, are harmless no-ops.
- Existing exact four-byte26/42 handling clears the active game. The private connection-local presentation flag prevents a second Remove after terminal presentation. It is absent from saved schema.
- Fixed recipe materials/product and quest effects remain atomic and once-only. Duplicate terminal reports and restored terminal state do not replay a recipe. Smelting's final ScoreSet guard preserves the already-presented terminal score. Forging remains authoritative over its board, score and output.
- No new score-based currency, fabricated stars or reward metadata. Original native quit39/6 remains a separately documented cancellation acceptance gap.

## Independent checks

Repeated30 ordinary crafting tests (three explicitly private checks ignored in this ordinary command), all9 Cooking/Smelting/Forging gameplay checks including private simulations, and the mapped17-stage opening/restart integration:40 checks passed, zero failures. The gameplay regressions distinguish automatic ACK retention from genuine Close, reject truncated/extra-byte Close, and verify no duplicate Remove or recipe/reward after Close.

Recompiled16 vectors against unchanged pinned community C# source. The independent output is byte-identical to the checked-in synthetic fixture, SHA256 `eb9fb61d9c3284b47d3c8584dd633a10121508d75c56f7d48d5c96eb79bf6153`. ACK vectors are constructor proofs; run164218 contains no actual39/7 ACK and none is claimed.

Commands used the project cargo wrapper. Cooking/Smelting gameplay used `rust-opening-cooking-original-assets`; Forging used its intended `rust-opening-forging-assets` version (the fire-only test expects three hidden actors); mapped opening used `opening-journal-text-v2/content`. All used private `rust-gateway-reference`. The original serializer was independently rebuilt with local .NET through `lab/tools/native-crafting-results-oracle/NativeCraftingResultsOracle.csproj`.

## Evidence and remaining acceptance

- `lab/reports/native-crafting-results-independent-review.json`, SHA256 `2bbd76bafaeee7783ba3242176253a7d287d3fe725d3127564fbb4b8a9ea6d4e`.
- Reproducible read-only checker `lab/reports/review-native-crafting-results-independent.py`.
- Original lifecycle proof `lab/reports/native-crafting-results-lifecycle-proof.json`, SHA256 `967e974b36806434f9eeaa629ac93bd2360e4502ee91e4f7ea95d36a91ca6012`.
- Original Close proof `lab/reports/native-crafting-close-proof.json`, SHA256 `2f91cb89894117be3a405159c40e9c14c48849c845464294ba1bce28f4b49f2b`.
- Independent vector output `lab/reports/native-crafting-results-vectors-independent.json`.

Run164218 already proves all six original Cooking rounds and the atomic recipe/quest result, but its movie remained frozen and Mae was not turned in. This source review clears the next release gate; it does not replace a fresh native results/Close/turn-in/restart playthrough. Reviewer wrote no shared Rust source, gameplay save, movie, release or native input. One private checker setup assertion initially contained an incorrectly transcribed Close-proof hash; it was corrected using the actual full file hash before the successful verification. No implementation or original evidence was altered.
