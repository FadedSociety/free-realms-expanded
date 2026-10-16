# 084 Selling and buyback

## Scope

Recovered the shipped client's coin-store sell and buyback wire contract, then built one bounded,
test-first transaction ledger that cannot duplicate coins or items when a request is replayed or the
character reloads. This is a recovery plus one server-side semantics slice. It grants no item, no
currency and no experience, and it does not claim a playable native flow: the lead owns wiring real
inventory and coin writes.

Two helpers carry real behaviour, both written test-first:

- `VendorLedger::sell` / `VendorLedger::buy_back` reproduce the sale/buyback transaction rules.
- `sell_payout` / `buyback_cost` prices a transaction in exact `i64` coins.

## Evidence

| Source | Path | sha256 |
| --- | --- | --- |
| Shipped client (native pseudocode) | `lab/decompiled/client/FreeRealms/pseudocode.c` | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |
| Shipped client script | `lab/decompiled/client-ui/ScriptsBase.lua` | `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6` |
| Client string mapping | `lab/working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt` | `1da9690fcc034657a1cd980cd96fc18cdc143a2fcf2a4373f8deccd032353e5a` |
| Original locale export (85,932 rows) | `lab/reports/locale-id-to-text.json` | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |
| Leaked server item-definition writer | `lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs` | `202576ff5dfdfa282f1c737f7b2932059e99ed0f2c111cf615ca1920995a7eb4` |
| Leaked server item definition | `lab/decompiled/server/Gateway/Gateway/ClientItemDefinition.cs` | `34bfce2ada6b3a016552d51da3da4e520272b171d1ad5c4f3a33908caf3f90ea` |
| Leaked server coin-store enum | `lab/decompiled/server/LibSOE/SOE/BaseCoinStorePackets.cs` | `deca817276195d56cbfc95179d3f208951adc1bbe846fe0c25104a81464e1c96` |
| Community sell handler | `lab/references/Sanctuary/.../CoinStoreBuyFromClientRequestPacketHandler.cs` | `999c5b3d431099e72da5b56bc9c57e95f48f5f448229bd5c374668a6f22eada7` |
| Community sell request | `lab/references/Sanctuary/.../CoinStoreBuyFromClientRequestPacket.cs` | `c1541f6119cd1d54cd629841ba6694f7ca3fedf665caeaf86abad8abdf0b919c` |
| Community transaction complete | `lab/references/Sanctuary/.../CoinStoreTransactionCompletePacket.cs` | `c8552fa453d04ad932eb4dfe23e8b42b925eae35dc155918efc2559247dc1f31` |
| Community transaction record | `lab/references/Sanctuary/.../CoinStoreTransactionRecord.cs` | `4be544610d08a121a81c8f71301c78b4264880a21133f40560b37dc1d2919919` |
| Community item catalogue (58,800 rows) | `lab/references/Sanctuary/src/Resources/ClientItemDefinitions.json` | `ffe9cd2dfe54e8bd825c299a5e0bf08da52533c7ca9b9d3036c15593310d0480` |
| Community emulator enum | `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` | `f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869` |
| Community emulator item data (1,383 rows) | `lab/references/OpenSourceFreeRealms/ofrserver/Customize/ClientItemDefinitions.json` | `815a5c3d5472c6cc658595318a96116486d8b90e69c41789916d6303cccad42f` |
| Read-only wiki corpus | `lab/references/freerealms-wiki/wiki.sqlite` | `b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911` |

Every hash above is recomputed by `scripts/pieces/piece_084.py` into
`lab/reports/piece-084-20261009T051314Z/piece-084-evidence.json`.

## Original facts

- **Base packet.** `BaseCoinStorePacket = 165` in the leaked server enum and in the client's own
  packet base. The shipped client's constructors call `FUN_00c6e960(<opcode>)` and then install the
  matching vtable, which recovers the sub-opcode table directly from the binary (not from a leaked
  enum): 1 `ItemList`, 2 `ItemDefinitionsRequest`, 3 `ItemDefinitionsResponse`, 4
  `SellToClientRequest`, 5 `BuyFromClientRequest`, 6 `TransactionComplete`, 7 `Open`, 8
  `ItemDynamicListUpdateRequest`, 9 `ItemDynamicListUpdateResponse`, 10 `MerchantList`, 11
  `ClearTransactionHistory`, 12 `BuyBackRequest`, 13 `BuyBackResponse`, 14
  `SellToClientAndGiftRequest`, 17 `ReceiveGiftItem`, 18 `GiftTransactionComplete`. Ghidra prints
  11/12/13 in hex (`0xb`, `0xc`, `0xd`), which is why a decimal-only scan misses them.
- **Direction of the names.** The naming is from the server's point of view: opcode 4 means "sell to
  the client" (a purchase) and opcode 5 means "buy from the client" (a sale). The community
  handlers confirm the mapping.
- **Sell request (5).** `FUN_00c76dd0`, reached from the Lua-facing `FUN_00c77030`, fills four
  32-bit words: two merchant GUID words, one item GUID word, one quantity word (16 bytes total).
  The pinned community reader declares the same three fields.
- **Buyback request (12).** `FUN_00c76c60` fills two 32-bit words and only proceeds when a lookup of
  the first argument reports type 2.
- **Buyback response (13).** `FUN_00c78a10` logs `Buy Back response - tid: %d, result: %d.` and
  treats `result == 1` as the only success; every other value takes the "failed to void transaction
  on client" path and still reaches the `OnBuyBack` script handler.
- **Transaction history.** `CoinStore_TransactionHistory_DS` is
  `BaseClient.CoinStore.TransactionHistory` with columns `Id 0, Type 1, MerchantGuid 2, ItemId 3,
  Quantity 4, TintId 5, Price 6, BoughtBack 7`. `GetValidBuyBackItems` drops every row whose
  `BoughtBack` is `1`, and `PopulateBuyBackData` builds its list only from rows whose `BoughtBack`
  is `0`. `Type` is documented by the community record as 1 = Buy, 2 = Sell.
- **Buyback entry point.** The merchant window's only buyback action is
  `NpcMerchantV2:BuyBackItem(id)` -> `CoinStoreScreen.BuyBackItem(id)`, i.e. the transaction row id
  alone; the native binding accepts one or two numbers. Taken together, a request with quantity 0
  has to mean "everything still outstanding on this transaction" (inference, marked as such below).
- **Sell flow in script.** `MarketplaceStrings.COINSHOP_LABEL_SELL_CONFIRMATION` (30148, "Are you
  sure you want to sell this item?") gates the call, `SellItem` serializes the row and shows the
  confirmation, and the confirmed path calls
  `CoinStoreScreen.SellItem(self.curSellItemGuid, self.curSellItemQuantity)`. Sell-all iterates the
  `PlayerSell` rows and sells each row's guid with its own quantity. `PlayerSell` carries `ItemGuid`
  (0), `Quantity` (4), `Price` (27) and `NoSale` (28); the success alert is locale 436335
  "You sold %1." and the buyback category label is 428367 "Buy Back".
- **Sell price source.** The leaked original server writes `ClientItemDefinition.ResellValue` as the
  37th field of the item-definition packet (`... ClientEquipReqSetId, ResellValue, Unknown36,
  Unknown37, Unknown38`), and the client exposes that value to script as
  `ItemDefinitionSelect_DS.CoinSellPrice = 26`. The packet ordinal and the script column number are
  different numberings and are not claimed to be the same index.
- **Result codes.** 1 Success, 3 InvalidItem, 4 InvalidQuantity, 5 InvalidMerchant, 6 InvalidItemTint,
  7 BalanceNotEnoughForPurchase, 8 ServiceUnavailable, 9 RecipientNotFound, 10 NotAMember, 11
  RecipientCanNotAccept - recorded in the pinned community handler; the client was not decompiled
  far enough to enumerate them independently.

## Community and wiki facts

- The community sell handler validates that the instance exists, that `Count >= Quantity`, deletes or
  decrements the row, pays `ResellValue * Quantity`, then sends the item delete/update, the coin
  count and a transaction-complete packet with `Result = 1` and `TransactionRecord.Type = 2`. It
  never sets `TransactionRecord.Price`.
- No pinned server implements the buyback request: Sanctuary's handler directory has no
  `CoinStoreBuyBackRequestPacketHandler`, and the older fork does not either. Every pinned server
  therefore ships a sell path with an unreachable buyback half, which is the gap this piece fills.
- The community emulator's item file (1,383 rows, all carrying `ResellValue`, hand-maintained per its
  own README: "These 2 documents are half documented so feel free to explore yourself") reproduces
  `max(1, cost / 10)` on 1,229 of its 1,230 priced rows. The single exception is item 56757
  (cost 270, stored 0).
- That tenth-of-cost pattern is **not** the pricing rule. The read-only wiki records ten item pages
  with a machine-readable resell value (the `Coin: 1: <value>` template field): `Zipped Leather
  Jacket` 27, `Rugged Winterwear Jacket` 75, `Buckled Skull Jacket` 27, `Strapped Long-sleeve Shirt`
  27, `Strapped Short-sleeve Shirt` 27, `Tattered Long-sleeve Shirt` 27 and four "Proof of Training"
  tickets at 0 ("Resell Value: Cannot Sell"). The pinned catalogue stores 27 for `Zipped Leather
  Jacket` at cost 400 - agreement between two independent corpora on a value that a tenth-of-cost
  formula would put at 40 - while `Rugged Winterwear Jacket` has a stored cost of 750 and no stored
  resell value at all. The ledger therefore takes the per-item value as data, and the checker fails
  if the module ever introduces a tenth-of-cost formula.

## Conflict

The leaked server enum says `CoinStoreBuyBackRequestPacket = 15`; the shipped client constructs that
class with opcode **12** (`FUN_00c6e960(0xc)`), and the current community packet reader also maps 12
to the buyback request. No coin-store class exists at 15 in the client. The client is the peer this
server has to satisfy, so the piece pins 12 and records 15 as a leaked-enum disagreement rather than
treating either source as authoritative for the other.

## Original, community and reconstructed boundary

- **Original:** the opcode table, the sell request layout, the transaction-history columns and the
  `BoughtBack` filter, the buyback response shape and its success check, the locale rows, and the
  fact that the leaked server sends a per-item `ResellValue` to the client.
- **Community:** the sell handler's arithmetic and validation, its result-code list, the transaction
  record's write order, the item data files and the wiki's per-item resell values.
- **Reconstructed:** the ledger itself - the buyback price (the unit price recorded on the sale,
  because no pinned server stores one), the quantity-0 meaning, the rejection value 2 for the
  buyback response, the effect of the clear-history packet on eligibility, the mapping from
  `NoSale` to result 3, and the rule that transaction ids and item instance GUIDs are monotonic and
  never reused across a clear or a restart. Native acceptance is not claimed.

## Implementation

`prototype/osfr-tools/src/restoration/piece_084.rs` (test-first, `#[path]` included by
`prototype/osfr-tools/tests/piece_084.rs` until the lead registers the module):

- Recovered tables: `SELL_REQUEST_FIELDS`, `BUYBACK_REQUEST_FIELDS`, `BUYBACK_RESPONSE_FIELDS`,
  `TRANSACTION_RECORD_FIELDS`, the opcode constants, `TRANSACTION_TYPE_BUY/SELL`, the result codes
  and `BUYBACK_ROWS_FILTER_RECOVERED`.
- Pricing: `sell_payout(unit_value, quantity)` and `buyback_cost(unit_price, quantity)` in exact
  `i64`; two recovered 32-bit amounts cannot overflow `i64`, so `Overflow` protects the coin balance
  instead, and a refused sale leaves the instance, the history and the balance untouched.
- Ledger: `VendorLedger::grant`, `sell`, `buy_back`, `record_purchase`, `clear_transaction_history`,
  `adjust_coins`, `buyback_rows`, `snapshot`, `restore`. `buy_back(id, 0)` buys the outstanding
  amount; a partial buyback decrements `quantity_remaining`; the row only flips `bought_back` when
  nothing is left; the restored instance keeps its original GUID.
- `VendorError` maps to the recovered wire result codes, with the buyback response using result 1
  for success and the reconstructed value 2 for every rejection.

`scripts/pieces/piece_084.py` re-reads every pinned source, cross-validates the Rust module against
it, and fails closed with a named check. It parses the client's opcode constructors (including hex
and templated vtable lines), the client's own request-builders, the script tables, the locale rows,
the leaked server's item-definition write order, the community handler, the leaked enum conflict,
the community item pattern and the wiki cross-check.

## Checks

| Check | Command | Result |
| --- | --- | --- |
| Rust RED | `scripts/cargo.ps1 test --test piece_084 --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | failed to compile: `couldn't find file tests\..\src\restoration\piece_084.rs` - `lab/reports/piece-084-20261009T051314Z/red-test-rust.log` |
| Rust GREEN | same command | `16 passed; 0 failed; 0 ignored`, exit 0 - `green-test-rust.log`; a first run exposed two wrong expectations and a real partial-write bug in the sale path, both fixed before the green run |
| Rust GREEN after format | same command after `rustfmt --edition 2024` | `16 passed; 0 failed`, exit 0 - `green-test-rust-postfmt.log` |
| Python RED | `lab/tools/wiki-env/Scripts/python.exe -m unittest discover -s tests\pieces -p "test_piece_084.py" -v` before the module-layout check existed | `FAILED (failures=1, errors=3)` - `red-test-python.log` |
| Python GREEN | same command | `Ran 16 tests ... OK`, exit 0 - `green-test-python.log`, repeated after the last edits as `green-test-python-postfmt.log` |
| Evidence document | `lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_084.py --output lab/reports/piece-084-20261009T051314Z/piece-084-evidence.json` | `piece 084 evidence ok: 16 opcodes, 1 wiki agreements, 1229 tenth-of-cost rows, 4 gaps` - `checker-run.log` |
| Format | `rustfmt --edition 2024 src/restoration/piece_084.rs tests/piece_084.rs` | clean, exit 0 |

Python tamper cases in the suite: a drifted client opcode, a script without the `BoughtBack` filter,
a module whose sell request drops a field, a module that derives the price with a tenth-of-cost
formula, a changed leaked enum, a drifted locale row, an absent input and a hash mismatch. Each must
exit 2 with its named check.

## Artifacts

| Artifact | sha256 |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_084.rs` | `c69982748ca4aff324ef354773923f52af52c7559a7ddc32c2cc65a7c8707ebf` |
| `prototype/osfr-tools/tests/piece_084.rs` | `069abefbc495420062b1276e491d99891cdec8ee1658b053525389d6d98e0135` |
| `scripts/pieces/piece_084.py` | `8d4956669df3c2c1004675ffe791bd0a0f1561cffcf68c7457a249e441478b1a` |
| `tests/pieces/test_piece_084.py` | `7e1f1cc16f2c6d439da6e71dde7f6c68dbca303e2359fb593f76da05ee1b6d1a` |
| `lab/reports/piece-084-20261009T051314Z/piece-084-evidence.json` | `5b6dd3d02ac504bc24a85e04b5de9a2d9c4f699fc1513513f1b3f3ffe12d85df` |

Hashes are of the files as first reviewed. The evidence bundle stamps `generated_utc` on every run,
so its hash changes whenever the checker is re-run; the four source artifacts above are stable.
Recompute with `Get-FileHash` before relying on them.

## Limitations and next acceptance

- No live sell or buyback exchange was captured, so the transaction-complete payload was not replayed
  against a real client. Native acceptance stays with the lead and is not claimed here.
- The buyback rejection value (2) is reconstructed: the client only distinguishes `1` from "not 1".
  The quantity-0 meaning is inferred from the merchant window's single-argument call.
- The effect of `CoinStoreClearTransactionHistoryPacket` on the server's own eligibility is
  reconstructed as "history removed, ids keep counting"; only the client-side list behaviour is
  recovered.
- The wiki contributes ten resell values and eight of them have no counterpart in either community
  catalogue, so the sell price for most items still rests on the community data files rather than on
  a first-party price table. Recovering the original item-definition data file would close that.
- The community sell handler never stores a transaction price, so partial buyback pricing remains a
  reconstruction of this piece.
- No shared file, gateway, save, content, launcher or release path was touched.
