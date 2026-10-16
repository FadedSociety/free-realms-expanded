# 083 Shops and purchases

Status: scoped purchase transaction implemented and locally verified, 2026-10-09 04:25 UTC. Evidence
label: **RECONSTRUCTED server transaction over ORIGINAL coin-store packet facts and recovered catalogue
rows**. This is not a native shop claim; no client has accepted a purchase from this module.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_083.rs` | request parser, catalogue document, purchase ledger |
| `prototype/osfr-tools/tests/piece_083.rs` | 13 owned vectors, `#[path]` include |
| `scripts/pieces/piece_083.py` | independent checker over the preserved lab inputs |
| `tests/pieces/test_piece_083.py` | 12 stdlib checker tests, including negative controls |
| `lab/reports/piece-083-20261009T042532Z/` | RED/GREEN logs, checker bundle, neighbouring suites |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.
Selling and buy-back (item 084), player trading (085), gifts and the Station Cash marketplace are
other packet paths and stay out of this module.

## Why this item exists

Item 083's acceptance is "source catalogue/prices/request packets; stock/ownership/currency
validation and rollback". The recovered coin-store path is the buy path: the client sends
`CoinStoreSellToClientRequestPacket` (family 165, sub-opcode 4) and the server answers with
`CoinStoreTransactionCompletePacket` (sub-opcode 6). The pinned community reference handler
(`lab/references/FreeRealms-Legacy/src/Sanctuary.Gateway/Handlers/BaseCoinStorePacket/CoinStoreSellToClientRequestPacketHandler.cs`)
shows the shape of that exchange and its limits: it looks the item up, charges the balance, writes
the database and only then reports; its own `// TODO: Implement other item types` marks the type
whitelist as unfinished, and it silently rewrites a non-tintable item's tint instead of refusing it.
This item turns that shape into a validated transaction and keeps the unfinished parts as explicit
data or explicit gaps.

## Original facts used

All of the following is read from preserved inputs in the lab. The checker pins each file's SHA-256
and fails when one changes.

- Coin-store packets are family **165**. The decompiled supplied server enumerates
  `CoinStoreItemListPacket = 1`, `CoinStoreItemDefinitionsRequestPacket = 2`, `...ResponsePacket = 3`,
  `CoinStoreSellToClientRequestPacket = 4`, `CoinStoreBuyFromClientRequestPacket = 5` (the sell path,
  item 084), `CoinStoreTransactionCompletePacket = 6`, `CoinStoreOpenPacket = 7`,
  `CoinStoreItemDynamicListUpdateRequestPacket = 8` / `...ResponsePacket = 9`,
  `CoinStoreMerchantListPacket = 10`, `CoinStoreClearTransactionHistoryPacket = 11`,
  `CoinStoreBuyBackResponsePacket = 13`, `CoinStoreSellToClientAndGiftRequestPacket = 14`,
  `CoinStoreBuyBackRequestPacket = 15`, `CoinStoreReceiveGiftItemPacket = 17` and
  `CoinStoreGiftTransactionCompletePacket = 18`
  (`lab/decompiled/server/LibSOE/SOE/BaseCoinStorePackets.cs`).
- The supplied first-party client builds the request and the completion packet
  (`CoinStoreSellToClientRequestPacket::vftable`, `CoinStoreTransactionCompletePacket::vftable`,
  constructor at `lab/decompiled/client/FreeRealms/pseudocode.c`), and it builds the merchant list
  packet (`CoinStoreMerchantListPacket::vftable`), so merchants really are a listed concept.
- The request body is `MerchantGuid u64`, `MerchantUnknown i32`,
  `ItemRecord { Definition i32, Tint i32 }`, `Quantity i32`, read with no trailing bytes
  (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet/BaseCoinStorePacket/CoinStoreSellToClientRequestPacket.cs`
  plus `.../Sanctuary.Packet.Common/ItemRecord.cs`).
- The completion result table is 1 success, 3 invalid item, 4 invalid quantity, 5 invalid merchant,
  6 invalid item tint, 7 balance not enough, 8 service unavailable, 9 recipient not found,
  10 not a member and 11 recipient cannot accept
  (`Sanctuary.Packet/BaseCoinStorePacket/CoinStoreTransactionCompletePacket.cs`). A transaction record
  carries type 1 (buy) / 2 (sell), the merchant guid, a record id, the item record, the quantity, the
  price and a **remaining quantity**
  (`Sanctuary.Packet.Common/CoinStoreTransactionRecord.cs`) - the remaining quantity is the recovered
  evidence that a purchase can consume limited stock.
- Catalogue and price rows: item definitions carry `Cost`, `MemberDiscount`, `MembersOnly`,
  `MaxStackSize`, `IsTintable`, the icon tint and `NoSale`
  (`Sanctuary.Packet.Common/BaseItemDefinition.cs`, `ClientItemDefinition.cs`). The member price is
  `(int)(Cost * (1.0f - MemberDiscount * 0.01f))` when `Cost > 0 && MemberDiscount <= 100`
  (`ClientItemDefinition.GetMemberPurchasePrice()`), i.e. float32 arithmetic with a truncating cast;
  the wiki membership row records the same 10% discount used by [[021 Property acquisition]].
- Store metadata: `ItemDefinitionMetaData` carries `BuyDisabled`, `CoinStoreOnly` and `Hidden`
  (`Sanctuary.Packet.Common/ItemDefinitionMetaData.cs`), so a buy-disabled row is a recovered concept.
- Stock and order fields: the first-party client names `MaxPerOrder`, `MaxPurchasable` and
  `QuantityAvailable` in its store data-source field table
  (`lab/decompiled/client/FreeRealms/pseudocode.c`) and its UI binds the same field numbers
  (`lab/decompiled/client-ui/ScriptsBase.lua`). The Lua also calls the native
  `CoinStoreScreen.CanBuyItem` / `EquipScreen.GetMaxItemsPurchasable` gate before it lets a purchase
  through (`setCoinStoreCanPurchaseItem`), which is why the test fixture treats ownership as a
  per-character cap rather than an unlimited grant.

Bound catalogue facts the checker recomputes from
`lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`
(sha256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`): 58,795 rows, cost
1..=100000, 58,697 rows at a 10% member discount, 98 rows with no discount field, 926 member-only
rows, 1,207 single-use rows and 6,604 tintable rows. The rows used by the Rust vectors are real rows:
1 (Penguin Badge, cost 5, member 4, tint 231), 2 (Gold Farmer Helmet, cost 320, member 288, tintable,
tint 228) and 34 (Heal Over Time Potion 1, cost 206, member 185, member-only).

## What is reconstructed

Every item below is labelled in the module and is a local reconstruction, not a recovered rule.

- **Catalogue document.** `{"schema_version":1,"merchants":[{"guid":..,"items":[..]}]}` in the module's
  accepted schema, bounded to 4 MiB, 4,096 merchants and 65,536 rows. The pinned 28.8 MB
  `ClientItemDefinitions.json` is the source for those rows, not the document itself: the lead
  prepares a merchant document from the rows a capture actually shows. Row fields map one-to-one to
  recovered concepts: `definition_id`, `cost`, `member_discount`, `members_only`, `buy_disabled`,
  `is_tintable`, `icon_tint`, and the reconstructed `quantity_available`, `max_per_order` and
  `owned_limit`. No merchant guid is recovered, so the fixtures use a documented synthetic guid.
- **Check order and refusals.** Merchant known (5) -> item in that merchant's catalogue and not
  buy-disabled (3) -> quantity positive, inside `max_per_order` and inside remaining stock (4) ->
  tint legal for the definition (6) -> membership for a member-only row (10) -> ownership cap (4) ->
  balance covers the total (7). The code name for each result is recovered; the mapping of a specific
  local rule onto a code is reconstructed and said so in the module.
- **Tint refusal instead of rewrite.** The pinned community handler replaces a non-tintable item's
  tint with the definition's icon tint. This module refuses with 6 so a malformed request cannot
  silently buy something the client did not ask for.
- **Staged commit.** The purchase is computed on a clone, the caller's writer receives the finished
  ledger, and only a successful write replaces the live ledger. A failed write leaves coins, stacks,
  stock and the transaction log byte-identical; the RED log below is the evidence that the
  reference-shaped first revision did not.
- **Per-character ownership cap.** `owned_limit` is catalogue data because the native `CanBuyItem`
  predicate is not recovered. It is validated per character and never blocks another character.

## Verification

Commands run from `prototype/osfr-tools` with `scripts/cargo.ps1` (private toolchain under
`../lab/tools`), and from the project root with `../lab/tools/wiki-env/Scripts/python.exe`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test piece_083` (first revision, reference-shaped) | 8 passed, **5 failed** - missing merchant, quantity, stock, membership, ownership and rollback rules | `red-baseline.log` |
| `cargo.ps1 test --test piece_083` | 13 passed, 0 failed, no warnings | `green-final.log` |
| `cargo.ps1 test --lib` | 24 passed, 0 failed | `lib-suite.log` |
| `cargo.ps1 test --test inventory --test character_store --test gameplay` | 12 (+2 ignored) + 8 + 1 ignored passed | `neighbour-suite.log` |
| `python scripts/pieces/piece_083.py --reports <bundle>` | exit 0, `piece 083 checks passed` | `independent-checker.log`, `shops.json`, `findings.json`, `provenance.json` |
| `python -m unittest discover -s tests/pieces -p "test_piece_083.py" -v` | 12 passed | `python-tests.log` |

The RED run is the defect this item closes. The first revision mirrored the pinned community handler:

```
assertion `left == right` failed: a failed write must not leave a purchase behind
  left: ShopLedger { coins: {11: 995}, stacks: {11: [OwnedStack { definition_id: 1, tint: 231, count: 1 }]}, ... stock: {402: {1: 3, ...}} }
 right: ShopLedger { coins: {11: 1000}, stacks: {}, transactions: {}, stock: {402: {1: 4, ...}} }
```

The independent checker is a second implementation of the recovered facts in another language. It
re-derives the request layout from the pinned C# deserializer and compares it with the offsets the
Rust parser actually reads; it compares the module's result and packet constants with the recovered
tables and reports the two gift-only results the module deliberately does not declare; it recomputes
the member prices with float32 rounding and compares them with the values the Rust tests assert; and
it requires the first-party client markers for the stock, order and buy fields. `tests/piece_083.rs`
checker tests mutate the recovered order, a Rust offset, a result code and a marker in turn and
require the checker to fail.

The crate-wide `cargo test` did not complete: the concurrently written `tests/piece_051.rs`,
`tests/piece_062.rs`, `tests/piece_075.rs` and `tests/piece_097.rs` (other workers' exclusive paths)
include module files that do not exist yet, so cargo stops before their targets. No error involves
this item; full log in `full-suite.log`.

## Deliberate gaps and remaining gates

1. **Lead integration.** Register the module in `src/lib.rs` (`pub mod restoration;` plus the module)
   and drop nothing else. The owned test target includes the file by `#[path]` until then.
2. **Real inventory (item 081).** The ledger holds per-definition counts and tints, the same shape as
   `inventory::Inventory` stacks. It does not mint instance ids, so the completion packet's `ItemGuid`
   must come from the inventory instance store. Coins come from the lead-owned economy layer; this
   module only records the debit it was told to apply.
3. **Wire.** Only the request parser is implemented. The completion packet's `DateTimeOffset`
   encoding is unrecovered, so the module returns transaction-record fields and does not serialize
   bytes. Item 025-style writer verification is the gate.
4. **Merchant list and stock payload.** The `CoinStoreMerchantListPacket` (10) body and the
   coin-store item-list payload are not recovered; the merchant table, `quantity_available` and
   `max_per_order` bindings are reconstructed data. The real values must come from a capture.
5. **Native predicate.** `CoinStoreScreen.CanBuyItem` and `EquipScreen.GetMaxItemsPurchasable` are
   native and unrecovered; until they are read, `owned_limit` is data and the client's own gate may
   disagree with the server's cap.
6. **Native acceptance.** No client has opened the coin store against this server, bought an item,
   seen the coin count change or reconnected to the saved ledger. That is the lead's gate.
7. **Selling and buy-back (item 084) / trading (085).** Sub-opcodes 5, 13 and 15 are parsed nowhere
   here, by design.

Cross-links: [[../../MMO Work Queue]], [[../../Design/MMO Worker Contract]], [[021 Property acquisition]].
