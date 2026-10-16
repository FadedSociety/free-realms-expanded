# Original Smelting

Smelting77 now runs through the Rust gameplay layer using the supplied four-stage `game_smelting.gfx` protocol. **Native entry/play/win is still pending.** Packet simulation is not evidence that the movie loads or its real controls work against Rust.

The original movie supplies ore breaking, measuring/pouring, furnace heating and molding controls/physics. Official JPEXS26.3.0 recovered120 AS2 scripts. Native readers and compiled pinned Sanctuary serializers corroborate Info39/16, Payload39/14, Start39/17, StartFlash26/12, GameOver39/18 and Remove39/19. The actual launcher names are `MiniGameFlash` and `game_smelting.gfx`; Info preloads the existing `SmeltingAssets.txt` with278 CRC-matching entries.

The original server phase table was not recovered. The preparer requires explicit `--allow-reconstructed-phases`. Numeric tuning and the fixed ten-bar output are reconstructed; the movie's hardcoded GameWon quantity10 is not proof of an authoritative economic rule. The movie uses a stopwatch, so the nominal90-second row value is not a deadline. Wiki Miner revision46068 supports Copper Ore5 + Singing Coal3 inputs.

The nearby furnace now requires an accepted pending Craft2002 objective, Miner14, proximity and ingredients. Clicking it no longer completes that goal. Winning consumes5 Copper Ore and3 Singing Coal, grants10 Copper Bars and advances the five-bar objective from the server transaction once. Inventory, quest and minigame changes stage together; the existing gateway persists the combined snapshot before sending replies. Class/equipment/world interactions are blocked while the movie is active. Native26/42 closes it without dungeon zoning. Reconnect cancels stale movie state while retaining monotonic attempt IDs and completed count.

Temporary local gathering drops1 Copper Ore and1 Singing Coal only when a valid nearby Miner interaction advances a newly accepted mining objective. Exhausted goals, remote requests and wrong jobs grant nothing. This is a bounded reconstructed drop rule, not recovered original mining loot or a restored mining minigame.

Only missing complete definitions879/21785 were added from pinned Legacy. Existing2739, all previous definitions, default/initial inventory, proven spans and profile records remain unchanged; user saves and identities were not edited. Inventory file SHA changed from `15ba86d35510997b1e377e2291f8e80f02886608d4c100335bb2662d2837eee6` to `e22bf9e9c82520e6d1f640ec07c472b2252f1415ef955cb6f95f573e1cea6323`. Backup: `../lab/references/rust-opening/backups/smelting-20261008T081718830193Z/inventory.json`. Opening content hash is unchanged; optional serde state preserves old-save loading.

Verification: six isolated ordinary tests and the old-save serde test pass; the private content and new Gateway transaction/restart oracle pass explicitly. The full17-step opening packet simulation now drives real smelting packet messages for both smelting goals and passes all stage/restart/discovery checks. The tests prove no reward replay, no repeat farming, malformed-bundle rollback, ingredient/output updates, quest advancement, close and reconnect cancellation. No release build or native input was performed for this integration.

```powershell
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts\prepare-smelting.py --allow-reconstructed-phases --merge-material-definitions
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','smelting_gameplay')
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','smelting_gameplay','private_mining_smelting_reward_atomic_once_restore_and_cancel','--','--ignored','--exact')
```

Next acceptance: native furnace entry, all four stages, normal score screen, material consumption/output, quest turn-in and restart. Capture the movie's actual StatUpdate requests if completion stalls: its AS2 adapter has an unusual missing closing quote in the FScommand string; native parser tolerance is unverified. No chat completion substitutes for this acceptance.

Read-only preparation for the next native gate: [[Smelting Native Control Geometry]] provides original movie dimensions/Lua scaling, initial five-ore targets/table bounds, Fill bucket/65% measure ring, Heat numerical and rendered pivot candidates, and Mold bucket/channel bounds. Coordinates are1280×960 client candidates to verify against fresh screenshots. Proof `lab/reports/smelting-controls-geometry-v2.json` SHA256 `1f181a517269a76d90c74d722e7443b99f4041e53694dd23ef6db674c6e6ea78`; original movie and all gameplay state remain unchanged.

Detailed source/function evidence, hashes, tool/license provenance and exact negative archive searches: `../lab/reports/original-smelting-proposal.md` and `.json`. Full17 simulation: `tests/opening_gateway.rs`, using `OSFR_GATEWAY_REFERENCE` and `OSFR_OPENING_REFERENCE` with its explicit ignored test. This note supersedes the older opening-audit statement that Copper Bar definitions were absent: the complete matching record was subsequently recovered from pinned Legacy.
