# 086 Currency and economy audit

Status: bounded audit returned for lead review. Item 086 acceptance is
"Enumerate actual currency types and sinks/sources; invariant checks across all
reward producers". This piece recovers the two currencies from the supplied
client and supplied server builds, enumerates every coin source/sink named by
the recovered protocol and by the current Rust crate, and ships a fail-closed
invariant checker (Rust audit module plus a read-only Python evidence bundle).
It grants nothing, spends nothing and does not modify shared gameplay files.

## Recovered original facts

### Currency types

Both facts below come from the preserved supplied builds:
`lab/decompiled/client-ui/ScriptsBase.lua` (client UI Lua) and
`lab/decompiled/server/...` (managed server assemblies).

| Currency | Scope | Recovered client rows | Recovered wire | Hash-pinned evidence |
|---|---|---|---|---|
| Coins | character | `PlayerInfo_DS.NumCoins = 2` (`_getCoinCount()`), `CoinCount = 16`, `SCCount = 17`; `PlayerInspectDS.Coins = 9`, `CoinBoost = 16`, `StationCash = 17` | `BaseClientUpdatePacket = 38` / `ClientUpdatePacketCoinCount = 19`, one signed 32-bit value; login profile writes `ClientPcData.Coins` (`int`) after `LastName` | `ScriptsBase.lua` sha256 `c7c71c7c…a8f6`; `BaseClientUpdatePackets.cs`, `BasePackets.cs`, `ClientPcData.cs` |
| Station Cash (SC) | account wallet | `PlayerInfo_DS.Wallet = 3` (`_getStationCashCount()`), `SCCount = 17`; `PlayerInspectDS.StationCash = 17` | `PacketBaseInGamePurchase = 66`: `PacketInGamePurchaseWalletInfoRequest = 10` / `…Response = 11`, Station Cash product lists; field layout not recovered | `ScriptsBase.lua`; `PacketBaseInGamePurchases.cs` |

The two getters are the tightest cross-check: `_getCoinCount()` reads
`DS_PLAYER_INFO` row 2 and `_getStationCashCount()` reads row 3, matching
`PlayerInfo_DS.NumCoins = 2` and `PlayerInfo_DS.Wallet = 3`.

Non-currencies found and deliberately excluded:

* `PaidNameChange.hasToken` / `Ui.CheckNameChangeToken(...)` is a boolean
  account entitlement, not a balance.
* `PlayerInspectDS.CoinBoost = 16` is an inspect stat, not a currency.
* `PlayerInfo_DS.VipRank/VipProgress/Viptitle` are membership progress, not
  wallets.

### Coin flows named by the recovered protocol

| Flow | Direction | Recovered evidence |
|---|---|---|
| Coin Shop buy / sell / buyback / transaction history | sink (and source for player sales) | `BaseCoinStorePacket = 165`; client `MarketplaceEnum.COIN_STORE = "7"`, `MarketplaceStrings.COINSHOP_LABEL_TITLE`, `CoinStoreHandler:SetCoinCount()` (invokes the player coin count), `CoinStore_PlayerSell_DS` / `CoinStore_TransactionHistory_DS` |
| Player trade | sink/source | `BaseTradePacket = 59` with `TradeCoinCountPacket = 10` and `TradeUpdateCoinCountPacket = 11` |
| Reward bundle + deferred non-member reward | source | `RewardBasePacket = 50`: `RewardBundlePacket = 1`, `RewardNonBundledItem = 2`, `RewardPacketNonMemberRewardDeferred = 6`, `RewardPacketNonMemberRewardGranted = 7`; client `Ui.GetDeferredCoin()` / `GetDeferredStars()` |
| Coin wheel | source | `PacketClientNotifyCoinSpinAvailable = 171`; client `CoinStoreReferrerElementEnum.COIN_WHEEL = 1`, `ShowCoinWheel` |
| Minigame reward preview (loot wheel) | source | `BaseClient.MiniGame.RewardPreview` row 0: coins are column 3, XP column 2; a positive coin value adds a loot-wheel entry named `CoinsScoreScreen` (icon id 4809) |
| Paid revive | sink | client `RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)` shows "Revive here: %1 coins"; the button stays disabled while the wallet is below the cost (piece 037) |
| Station Cash store order | sink (SC) | `PacketInGamePurchasePlaceOrderPacket = 3`, `…Response = 4`, `PacketInGamePurchaseStationCashProducts* = 14/15` |

Price rows recovered from the client UI: `ItemDefinitionSelect_DS.CoinPurchasePrice
= 25`, `CoinSellPrice = 26`; `MerchantSell_DS.Price = 27`,
`MemberPurchasePrice = 30`; `InGamePurchase_StationCashPriceList_DS` carries
`TotalStationCashAmount = 1`, `PromotionalStationCashAmount = 2`,
`PromotionalStationCashGranted = 3`, `RetailPrice = 4`, `SalePrice = 5`,
`IsOnSale = 6`.

### Recorded self packet

`lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`
(sha256 `e471ab7f…8777`) stores `"Coins": 0` as an integer and contains no
`Wallet`, `SCCount` or `StationCash` key. That matches the client Lua: the SC
wallet arrives separately through the in-game-purchase wallet response, so the
character profile carries coins and the account carries Station Cash.

Note on the source build: `BaseCoinStorePackets.cs`, `BaseTradePackets.cs`,
`RewardBasePackets.cs`, `PacketBaseInGamePurchases.cs` and `ClientPcData.cs` are
decompiled from the user-supplied `OSFR.Server.zip` (the OpenSourceFreeRealms
managed server and its SOE protocol library). They are recovered names, not
historical SOE source.

## Community facts (secondary, pinned wiki)

`lab/references/freerealms-wiki/wiki.sqlite` (sha256 `b9d7b9f1…d911`):

| Page (revision) | Recorded statement |
|---|---|
| Coins (`page_id` 6720, revision 45214, 2021-11-20, <https://freerealms.fandom.com/wiki/Coins>) | "Coins are an in-game currency specifically for Free Realms. They can be used to purchase items from the Coin Shop, and farming. Unlike Station Cash, coins do not cost real currency to obtain, although it is available to spend SC to purchase coins in the SC Marketplace." and "You can earn coins by doing random quests and dungeons throughout the game." |
| Station Cash (`page_id` 16472, revision 45198, <https://freerealms.fandom.com/wiki/Station_Cash>) | "a form of virtual currency used for microtransactions within Free Realms, such as purchasing items through the SC Marketplace", bought from the website. |
| Coin Shop (`page_id` 8991, revision 45215, <https://freerealms.fandom.com/wiki/Coin_Shop>) | the Coin Shop sells wares for coins in contrast to the SC Marketplace's Station Cash. |

These are single-source community statements and are kept separate from the
recovered rows above.

## Reconstruction-side inventory (this crate)

Scanned by `scripts/pieces/piece_086.py` from the current sources; every entry
is graded `reconstructed` because no original payout number is recovered.

| Producer | File:line | Direction | Once-only | Wire guard |
|---|---|---|---|---|
| `Inventory::grant_coins` (the only balance mutation) | `prototype/osfr-tools/src/inventory.rs:427` | balance | - | `checked_add` + `<= i32::MAX` |
| `QuestReward.coins` | `prototype/osfr-tools/src/quests.rs:67` | source | `progress.rewarded` | `definition.coins > i32::MAX as u32` |
| `Trial.reward_coins` | `prototype/osfr-tools/src/quests.rs:119` | source | trial cooldown `next_trial_ms` | `trial.reward_coins > i32::MAX as u32` |
| `EncounterReward.coins` | `prototype/osfr-tools/src/encounter.rs:133` | source | `state.rewarded` | `d.reward_coins > 1_000_000` (reconstructed cap) |
| `OpeningReward.coins` (stage + discovery) | `prototype/osfr-tools/src/opening.rs:46` | source | `progress.rewarded` | `reward.coins <= i32::MAX as u32` |
| `RewardSchedule.coins` | `prototype/osfr-tools/src/kart_racing.rs:261` | source | `claim_reward` consumes the finished session | granted only from a finished, priced track |

Implemented sources in the live gameplay path: `quest_reward`,
`checkpoint_trial`, `encounter_clear`, `opening_stage`, `opening_discovery`.
`kart_race` is a proposal that the lead has not registered in `src/lib.rs`.

## Invariant results

`check_invariants` over the current tree: 0 violations; exactly one coin
balance assignment site; the 32-bit wire guard is present; the once-only guards
exist for quests, opening, encounters and kart claims; no gameplay file grants
Station Cash. The Rust module enforces the same set for one flow list
(`audit`) and for the sink side (`CoinLedger`: no overdraw, no overflow past
`i32::MAX`).

Open findings (stable ids with a status, not completion claims):

| id | status | Meaning |
|---|---|---|
| `station-cash-source-external` | account-payment-only | SC enters only through the account wallet flow; no gameplay producer creates it |
| `coin-shop-sink` | Queued (items 083, 084) | recovered coin sink, not implemented; coins can currently only increase |
| `trade-coin-flow` | Queued (item 085) | recovered coin trade, not implemented |
| `paid-revive-sink` | proposal-only | revive cost and wallet gate recovered (piece 037); nothing debits coins |
| `quest-reward-atomicity` | coins-before-items | `gameplay::apply_quest` credits coins, then grants items after the quest is already marked rewarded; an item failure leaves a partial reward (piece 069 owns the exactly-once boundary) |
| `kart-reward-integration` | not_registered | kart payouts exist as a proposal only |
| `coin-payout-numbers-unrecovered` | gap | no original numeric payout is present in the pinned inputs |

## Gaps

* Original numeric payouts are unknown: the recovered reward and coin-store
  packets name structure, not amounts. Every local amount stays
  caller-supplied or reconstructed.
* Station Cash wire layout is not recovered; only the wallet request/response
  family and product lists are named.
* Coin Shop opcode semantics beyond the names (`CoinStoreBuyFromClientRequestPacket`
  = 5, buyback = 13/15, transaction complete = 6, …) are not yet decoded.
* Wallet top-up is an external payment path (credit card, top-off flow) and is
  out of scope for gameplay restoration.
* `Ui.GetPlayerCoinCount` / `Ui.GetDeferredCoin` are native client bindings used
  by the Lua; the exported pseudocode contains function bodies only, so the
  bindings themselves are not in the pinned native evidence.

## Commands and results

```
cd prototype/osfr-tools
& ..\..\scripts\cargo.ps1 test --test piece_086
  test result: ok. 10 passed; 0 failed

cd ../..
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_086.py"
  Ran 16 tests ... OK

& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_086.py'
  {"currencies": 2, "violations": 0, "implemented_sources": [...], "reports_dir": "lab/reports/piece-086-<stamp>"}
```

Pre-change RED for both suites was recorded: the Rust target failed with
`couldn't find file tests\..\src\restoration\piece_086.rs`, and the Python suite
failed with `FileNotFoundError: scripts\pieces\piece_086.py`.

Lab bundle (private, canonical `lab/reports/piece-086-20261009T052858Z/`) with
`currencies.json`, `flows.json`, `invariants.json`, `findings.json`,
`provenance.json` (input paths, sizes and SHA-256 hashes). Earlier stamps
`20261009T052123Z`, `20261009T052630Z` and `20261009T052735Z` were re-written
with identical content; recursive cleanup is blocked in this environment.

## Integration and native gates

* Lead: register `src/restoration/piece_086.rs` in `src/lib.rs` and drop the
  `#[path]` include in `tests/piece_086.rs`.
* Lead: the audit is a checker, not an economy. Spend paths (083/084/085,
  revive) must not be claimed until they debit coins through a checked sink.
* Native gate: not attempted here. No native UI, save file or game window was
  touched; claiming a playable economy would require an exercised in-game
  purchase or revive.
