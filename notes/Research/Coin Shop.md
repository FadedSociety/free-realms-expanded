---
type: research-note
updated: 2026-10-13
---

# Coin Shop purchases (W31)

## Status

The scoped service and focused tests exist in `prototype/osfr-tools/src/coin_shop.rs` and `tests/coin_shop.rs`. The lead still needs to register the module and integrate its inventory snapshot, replies, and commerce ledger with the gateway/save path. No native-client acceptance was run by this worker.

## Recovered client facts

The source of protocol facts is [[Build Guides/Gameplay/Coin Store and Purchases]]. It records the reviewed client addresses and warns that no store traffic was captured (guide lines 14, 96, 201).

- Coin Shop uses family `0xa5`; the packet header contains two signed 16-bit fields. Sub 4 buys, sub 5 sells, sub 6 completes a transaction, sub 12 requests buy-back, sub 13 replies, and sub 14/18 carry a gift request/result (guide lines 50-59, 96, 110-129). The buy request has merchant id, two caller dwords, item id, and quantity. Their meanings are unresolved.
- The sub-6 reader consumes two dwords, then a record: u32, u64, u32, u64, nested block, and three u32. Record field meanings and exact order remain unresolved (guide line 100; `00c74ef0`, `00c71f30`). The Rust reply preserves that reader shape; the zero-length nested block and populated values are reconstructed (R).
- The client result dispatcher accepts numeric values 1 through 11, then sends each through the same virtual call (`00c6f300`). The guide lists the result names in this order: Success, Fail, Fail_InvalidItem, Fail_InvalidQuantity, Fail_InvalidMerchant, Fail_InvalidItemTint, Fail_BalanceNotEnoughForPurchase, Fail_ServiceUnavailable, Fail_RecipientNotFound, Fail_NotAMember, Fail_RecipientCanNotAcceptItem (guide line 56). Numeric-to-name mapping by this order is likely, not proven; the Rust constants are reconstructed (R).
- Sub 13 reads transaction id, result, and a third dword; result 1 succeeds (`00c71e40`, `00c78a10`; guide line 58). The gift response raises the client gift result event (`00c77df0`; guide line 59).
- Marketplace family is `0x42`. Client readers handle status sub 13 as one byte (`00b3bde0`), enable sub 24 as one boolean (`00b3bf30`), payment-source sub 33 as two bytes (`00b3c000`), account-info sub 26 as a status, two length-prefixed strings and a flag (`00b2e080`), and wallet sub 11 through `00b2b450` (guide lines 63-74, 102-104, 131-151).
- Membership product data is requested at sub 22 and returned at 23. Station Cash product data uses 14/15. Currency, state, and country lists use 16/17, 18/19, and 20/21. Empty product/code lists are an allowed server policy in the guide (guide lines 63-74, 175).
- Place-order response reader `00b3b6d0` treats result 1 as success. Reader `00b2ded0` consumes two dwords, a length-prefixed string, and two dwords. The Marketplace request layout is not recovered; sub 3 is refused before parsing and gets a result 0 plus the client-readable message. `NpcMerchantV2` sub 43 request layout is also unresolved; sub 44 result other than 1 raises a client alert (guide lines 63-74, 141-158).
- Membership purchase requests use URAM/web-service classes rather than this gateway path. The guide does not establish their response body or whether the response comes from the gateway (guide lines 73-74, 181). This module cannot stub that separate service.

## Catalogue provenance

The checked-in catalogue was built from reference data only. No FreeRealms-Legacy source code was copied. One exploratory broad search also printed a few C# handler lines; they did not inform implementation behavior, which is based on the accepted client map/build guide or explicit (R) policy.

- Local Legacy revision: `9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0).
- `CoinStoreItems.json` has 17,198 rows with item id, category, and tint group. SHA-256: `a952fe176611a75d2f084e27d4fea328d5d0898eea72efb2f9d57b8fda389388`.
- `ClientItemDefinitions.json` matches all 17,198 ids. Its `Cost` field exists for 16,932; 266 entries have no price. Its `MemberDiscount` is 10 for all listed items, and 2,217 entries explicitly set `NoSale`. SHA-256: `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`.
- `Stores.json` has one Marketplace store. All 1,267 `StoreBundles.json` rows have `CurrencyType: 1`; they are not Coin Shop prices and were not mapped as coins. Hashes: `e4588fb75bbe9ec4f83312f8333591c46b0fe4b59eb01b6c05a71f928959ef0d` and `4f6dc81e97ca65d6571a27c07993c2b9b07d4612b14850656882b6c1b8b2771f`.
- `data/content/stores.json` keeps 17,198 rows. Tags `L1` through `L4` identify the source data fields. `R1` marks unknown or defaulted values; `R2` marks member-price arithmetic; `R3` marks the resale rule. `Cost` is used as the starting regular coin price because these rows belong to `CoinStoreItems`; this mapping has not been checked against a real client transaction.
- Member price is calculated as `Cost * (100 - MemberDiscount) / 100`, integer floor (R). The source percentage is also retained on each row. Sell price is half of `Cost`, integer floor (R). Rows with no `Cost` remain unpriced and cannot be bought or sold. Missing `NoSale` is treated as false (R). The source map and each row's tag list are in `stores.json`.

## Rust behavior and reconstructed policy

- Buy validates catalogue membership, item definition, quantity, price, balance, and bag capacity. It stages a new inventory snapshot, debits coins, calls `Inventory::grant_bundle`, then replaces its state only after all checks pass. `set_member_status` selects the stored member price; its default is non-member until the caller supplies the account flag (R). Sell stages `Inventory::consume` plus `Inventory::grant_coins`; equipped, rented, NoSale, unpriced, and unowned items are refused.
- The first undocumented caller dword in Coin Shop sub 4 is the idempotency key (R). Replays return the cached transaction-complete response without replaying item or coin updates. The in-memory ledger holds up to 4,096 ids and does not evict earlier entries; integration must persist it if retries after restart must be safe.
- Buy-back uses the latest 20 successful sales (R), charges the resale amount again (R), and is single-use. The sale window and prices are server policy, not recovered original rules.
- The client has a Coin Shop gift path, so the module does not invent a missing-protocol refusal. It returns sub 18 with `Fail_ServiceUnavailable` (mapped as code 8, R) because recipient lookup and cross-account atomic save are not available in this module. Neither account changes.
- Marketplace startup replies enable the shell (sub 24) and disable both payment-source bytes (sub 33). Wallet balance is zero (R). Account info defaults to `US`/`USD` (R). Product lists and state/country lists are empty. Marketplace order preview/order and `NpcMerchantV2` purchase requests receive failure replies; no balance or inventory changes.
- The service returns inner application packets. The caller must persist the changed inventory and commerce ledger before sending replies. The existing startup item-list builder in `startup_catalogues.rs` serializes 19-byte rows, but its three words and three flags remain unnamed by the client reader (`00c71510`; guide lines 98-99). This task does not fabricate those values or build the startup list.

## Checks run

- Initial red run: `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W31'; & ..\..\scripts\cargo.ps1 test --test coin_shop` from `prototype/osfr-tools`. Six tests failed against placeholder behavior, as expected.
- Focused run before the member-price flag and assertion were added: 7 passed, 0 failed. It covered exact buy debit/grant and retry, insufficient coins, full bag, unlisted item, sell/buy-back, Marketplace order refusal, and handshake/payment switches.
- Final rerun after adding member pricing stopped before running tests. The first attempt also saw a temporarily missing declared `fishing_service` module; after that shared file appeared, Cargo reported 11 errors there (including `[u8; 8]` where a `Vec<u8>` is extended/passed at lines 626-679) plus E0596 in unowned `src/housing_native.rs:533` (`writer` reborrow). No shared files were edited. Coin Shop tests, including member pricing, remain unexecuted in this checkout state.
- `rustfmt --check` passed for both owned Rust files. PowerShell `ConvertFrom-Json` parsed `stores.json`: 17,198 unique items, 16,932 priced and 266 unpriced; item 8 maps to 15 regular, 13 member, and 7 resale coins.
- Formatting command from the project root: `& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' --edition 2024 'prototype\osfr-tools\src\coin_shop.rs' 'prototype\osfr-tools\tests\coin_shop.rs'` (only owned files).
- Native-map review used `python -I scripts/native_map/nmq.py show 00c77230 00c76a00 00c75ed0 00c78220 00c74ef0 00c71f30 00c6f300`, `python -I scripts/native_map/nmq.py dis 00c6f300 160`, `python -I scripts/native_map/nmq.py dis 00b2ded0 224`, and focused `brief`/`dis` lookups for Marketplace account, wallet, SKU, status, enable, and payment-source readers.

## Lead integration gates

1. Register `coin_shop` and keep one service per character or adapt it to the gateway's transaction boundary. Set membership from the verified account flag. Sync current inventory before commerce handling; save inventory, transaction ids, and buy-back sales atomically before sending its replies.
2. Treat sub-4's idempotency field, result-code mapping, sub-6 record slots, and zero-length nested record as hypotheses until a client session against a stub is captured.
3. Add an explicit URAM membership-purchase refusal at the service that owns those requests. The current gateway module cannot guarantee the membership path returns the not-available response.
4. Add recipient resolution and cross-account persistence before enabling coin gifts. Until then, keep the explicit sub-18 refusal.
5. Verify the startup item-list metadata, displayed member prices, Legacy `Cost` mapping, and resale rule against client-side readers/data. Native client acceptance remains outstanding.
