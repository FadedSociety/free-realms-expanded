# First playable Rust gameplay slice

The original Free Realms client now runs a complete local gameplay loop against Rust: a guide interaction, a reconstructed three-enemy quest, damage and defeat, a coin/item reward, native equipment changes, a timed checkpoint activity, and restoration after restarting both processes. This is a substantial prototype, not complete original content restoration.

## Verified in the actual client

| System | Observed result |
| --- | --- |
| Guide | Native interaction request targets the nearby Realm Guide; quest acceptance was separately performed through chat and saved |
| Enemy | Training Robgoblin uses surviving definition 1294/model 4; four authoritative hits defeat it and remove the actor |
| Quest | Local quest 900001 progresses to 3/3; completion rewards once |
| Wallet | Quest grants 25 coins; checkpoint activity grants another 10; native Stuff screen shows 35 after restart |
| Item reward | Definition 1195, Wheeler on the Cob, instance 1077 increases from 9 to 10; native Supplies > Food & Drink displays 10 after restart |
| Equipment | Native request 42/3 equips instance 2, Small Stitched Pants, in Adventurer profile 1/slot 4; model changes to long brown pants and restores after restart |
| Activity | Ordered checkpoints completed through real keyboard walking in 18,456 ms; client displays 18.46 seconds; best time and completion count persist |
| Failed activity | Earlier attempt timed out and granted nothing |
| Persistence | Quest, reward flags, wallet, item quantities, equipment, defeats and activity record restore together |

The guide, enemy placement, quest 900001 and checkpoint rules are explicitly reconstructed. The enemy has 100 health, 25 damage per hit, an 8-unit range and a 600-ms cooldown. These are local rules, not recovered historical balance. Attack feedback is chat text; native original combat abilities, enemy AI, retaliation and death animation are not restored. Respawning uses `realm respawn`.

## Evidence

All paths below are relative to the lab at `%USERPROFILE%\Documents\Projects\CartoonMMO\lab`.

- `runs/client-baseline-20261008T031719Z`: first two real attacks and native guide interaction.
- `runs/client-baseline-20261008T032035Z`: first two defeats; subsequent run starts with exactly this saved state.
- `runs/client-baseline-20261008T032359Z`: third defeat, quest completion, item/coin reward, native equipment change and checkpoint completion. All 4,125 frame hashes and 4,123 post-session CRCs validate. Maximum server data frame is 334 bytes within the 512-byte negotiation. See `analysis/gameplay-wire-proof.json`.
- `runs/client-baseline-20261008T033402Z`: category-enabled restart restores progress and populates Stuff; inventory text search subsequently crashes the native client with `0xC0000005`. This run is explicitly unclean.
- `runs/client-baseline-20261008T033628Z`: fresh restart, native equipment and Supplies > Food & Drink inspected; deliberately stopped after 412.656 seconds. `food-drink_small.png` shows 35 coins and Wheeler on the Cob quantity 10. `restored-gear_small.png` shows the restored pants.
- `reports/gameplay-comparison-032359-to-033628.json`: gameplay state is exactly equal across the restart. Position is exact. An intervening client update changed rotation X by one f32 ULP; do not claim whole-character byte equality across all runs.
- `manifests/rust-gameplay-milestone.json`: aggregate evidence and current binary hash.

The gameplay run used binary SHA-256 `eb9b6198df0603e6043d1add4dafe424bf8876abc025e9da02028e2ebc130a41`. The category-enabled final binary is `949f89a53f2dc644d3b58792d6661d9c12e4ae062beb945e8c8f3c0648513b72`. These are different builds; the first did not execute the later category module. No C# game server ran during these checks.

## Community evidence first

Inventory/profile layouts were corroborated with the supplied serializer and pinned Sanctuary/FreeRealms-Legacy sources. Private spans and hashes prove the original 1,386 item instances, 16 profiles and 25-entry quest journal. Item category reference packet 44/2 was missing from the captured startup sequence. Adding the community category tables made the native Stuff list populate; all 2,675 serialized bytes match the unmodified community C# serializer. The 71 categories and 74 associations remain private game-derived content.

The existing MIT FreeRealmsLocaleTools reader, pinned at `ba2ede225ff33edab8e8e0d20c45d4c5156309e5`, read the preserved locale files; 113,286 entries and 85,932 named IDs were exported privately. No new asset-format reader was written for this slice.

FreeRealms-Legacy is pinned at `9e082025391177d5a33ee18c714ddd06b457a695` under `lab/references`, with source hashes and AGPL-3.0 provenance. Sanctuary remains pinned at `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`. Upstream source and raw data remain outside Git.

Archived original soccer, racing and demolition derby captures were also investigated. All 37,327 primary UDP CRCs validate under the independently inferred two-byte CRC model. Soccer has a typed `sg_soccer` arena transition, configuration, teams, controls, ball updates and a goal. These captures do not establish normal match completion or rewards. Soccer is the strongest next original-minigame candidate; see `reports/minigame-protocol-evidence.json`.

## Implementation and checks

Rust now owns typed inventory/equipment, quest progress, local combat, checkpoint timing, chat command handling and a unified version-2 character/gameplay snapshot. A complete application bundle stages its changes; reward flags and inventory commit together. The executable persists before committing live state or queuing reward replies. Invalid commands give feedback without changing progress. Reconnect clears unfinished activity clocks while keeping earned records.

The snapshot bound is 1 MiB. Old version-1 character saves migrate without discarding progress; loading a gameplay save without `--content` is refused. Only one process should write a state file.

Verification: 73 ordinary tests passed; eight private-input oracles are opt-in. The new inventory, quest, combat, complete-loop and category oracles were explicitly run. The complete-loop check covers reward once, malformed bundle rollback, invalid-command feedback, unified saving and a fresh gateway restore. Formatting and release build passed. The final executable passed seven real-UDP session checks in `runs/gateway-session-check-20261008T033733Z-84823f64/report.json`.

## Run this local build

From `%USERPROFILE%\Documents\Projects\CartoonMMO\project`:

```powershell
.\scripts\cargo.ps1 build --release --manifest-path prototype/osfr-tools/Cargo.toml
& '..\lab\tools\universal-modder-env\Scripts\python.exe' scripts/run-local-baseline.py `
  --seconds 600 --rust-reference '..\lab\references\rust-gateway-reference' `
  --content '..\lab\references\rust-gameplay' `
  --state-file '..\lab\state\rust-gameplay-character.json'
```

The harness allows 10–900 seconds, copies the client, serves the asset manifest locally and records traffic. It sends no keyboard/mouse input itself. Separate authorized Universal Modder actions performed these checks. `input_automation: false` describes the harness, not the absence of those separately recorded actions.

Use Enter to open chat and type `realm help`, `realm quest`, `realm attack`, `realm respawn`, `realm race start` or `realm status`. The original client suppresses similar repeated chat messages: after four similar attempts it requires 30 seconds since the last attempt, including suppressed attempts. Native analysis identified `[ChatSpeed]` defaults Any=330, Dup=1000, Dup3=2000 and Max=30000 ms. Do not repeatedly retry a suppressed command. C opens native gear; I opens Stuff. Browse Supplies > Food & Drink for the reward.

The checkpoint course starts near X=-1414.6/Z=351.6 and runs through (-1390.6,351.6), (-1390.6,375.6), then back to the start. Each checkpoint has a 6-unit radius; time limit is 90 seconds, reward 10 coins, cooldown 30 seconds. The server checks order, elapsed time and plausible travel. The successful check used W/S and Q/E strafe; A/D turn. It sent real game inputs, not fabricated positions.

## Known limits and next work

Inventory text search caused one access violation. Category browsing remains usable, but search is not accepted as working. No fault address/dump established a cause. Three owned instances lack definitions in the supplied reference; missing category references also exist upstream. These are hypotheses, not a proven diagnosis. See `reports/native-inventory-search-crash.md`.

Some icons request obsolete delivery URLs and receive 404s. Equipment models and surviving item icons still display. Native item consumption, effects, trading, shops and general rewards are not yet verified. Server-authorized consume logic is tested but does not establish native use behavior.

The original quest 1617, Harassing the Harassers, survives with its Petty Yeti objective and reward. No proven Petty Yeti definition was found, so Robgoblin kills deliberately do not advance it. The original 25-entry journal is preserved; reconstructed quest status uses chat/server state, not a guessed native journal-update packet.

Next: diagnose the search fault with controlled native evidence; recover one original encounter/quest mapping; implement the original soccer lifecycle through normal completion; replace captured initialization incrementally. See [[Gameplay Coverage]] and [[Design/Rust Revival Roadmap]].
