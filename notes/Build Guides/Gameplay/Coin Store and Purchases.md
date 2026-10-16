---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Coin Store and Purchases

Part of [[Build Guides]] › Gameplay. Related: [[Items and Inventory]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the native-map labels for a function. Where a label says `unknown` but the function's pseudocode was read in this pass, the line says "label unknown, pseudocode read". Source tags: *client* = read from the shipped client (function pseudocode or bytes in this pass or in an accepted shard, the shipped `Resources` files, client locale text, or a Lua name table decoded from `FreeRealms.exe`); *video* = a reviewed frame record (frame keys are given); *wiki* = the fan wiki. Precedence is client, then video, then wiki. *Reconstructed* (R) is our own server policy, not original behaviour. Several function labels borrow packet names from an emulator packet catalogue; that catalogue is not used here. Sub-opcode numbers are taken only from constructors and switch statements in the client. No store traffic was ever captured, so every statement about what the original server sent or decided is a hypothesis built from what the client requires. The revival takes no real payments (see "What the Rust server must do"). No player names, chat or player-chosen names are recorded. Related: [[Items and Inventory]], [[Pets]], [[Mounts]], [[Housing]], [[Trade]], [[Quests and Objectives]].

### Overview

There are three separate places to spend money in the client, with two currencies. The code keeps them in separate packet families and separate Lua screens.

- **Coin Shop (coins).** The in-game shop that takes coins, the currency earned by playing. The client's own text calls it the "Coin Shop"; the code, logs and packets say `CoinStore`. Players open it from the Shopping button on the game dock, then the Coin Shop button (client locale tutorial text). It sells clothes, weapons, pets, furniture, jewellery, potions, recipes, farming and crafting supplies, flair shards and style cards (wiki). It has categories, a search box with filters and sorting, an item preview, Buy, Sell (items show a "Resell Value" in the inventory), Buy Back, gifting to another player, and a transaction history (client Lua name tables). Members see a regular price and a lower member price with a "You saved" line (video `1TNyzyI62nc/00252`, `1TNyzyI62nc/00270`).
- **Station Cash Marketplace (Station Cash, "SC").** The paid shop, opened from the shopping-cart icon on the game dock (client locale; video ad text `e4XXSnnive0/00236`). The client's text lets players buy Station Cash with a card or a store card, buy items and pets, claim codes, and upgrade to a Membership. The code calls the shop layer `InGamePurchase`. Its front page has a "Hot New Items" category, an Item of the Day and a Bundle of the Day, billboards (promo panels), a VIP section, filters for New, Limited Time and Sale, rentals ("rent or buy"), gift orders, and an order preview before purchase (client locale and Lua name tables). A wallet line shows the Station Cash balance (client locale). SC bought items go to the character's inventory when Buy is clicked (client locale tutorial). The wiki gives the rate as one US dollar to 100 SC (read from stripped template values; see Open questions).
- **Vendors.** NPC merchants use the same two paths. A Coin Shop request names a merchant by a 64-bit id (client, see Network). A separate set of Station Cash family messages drives an "NpcMerchantV2" window with a merchant list, limited stock and a Buy button, then a close message (client, dispatcher `00b48a10`, confirmed). The wiki describes a travelling merchant who offers four to six random Marketplace items in limited quantity, sometimes at a discount or in rare colours; whether that is what the client's merchant messages drive is an inference.
- **Membership offers.** The client offers membership in several places: an Upgrade button and upsell panel (video `yl6WBIErAsY/00028`: monthly price, more jobs, more characters, leaderboards), member-only items and prices, and member trials for pets and mounts. Membership is bought with Station Cash from the wallet or with a card, and renews monthly until cancelled (client locale). Wiki prices were 4.99 dollars for 1 month, 12.99 for 3, 24.50 for 6, 29.99 for 12 and 34.99 for lifetime; lifetime stopped being sold in May 2013, and on 2014-02-04 every player was upgraded to Member (wiki). The video shows "Lifetime Membership $34.99" on shop windows.
- **Currency facts.** Coins and Station Cash are different balances. A coin shortfall in the Coin Shop gives a coin message; an SC shortfall gives "Not Enough Station Cash" and an Add Funds prompt (client locale). The wiki says coins can also be bought with SC in the Marketplace (wiki, single source); the client locale has entries named "1,000 coins for Coin Shop", "10,000 coins for Coin Shop" and "100,000 coins for Coin Shop" that read like such coin packs, but nothing in the code ties them to a sale.
- **Real money.** The client sends card, wallet and membership requests to payment web-service clients (CAS and URAM) with their own endpoint URLs (`00d6d4b0`, confirmed), not through the game-gateway messages listed below, and opens web pages for funding and membership (client, see below). The revival will not do this. The server only has to make the Marketplace usable without it: grant or stub the wallet, hide the funding paths, and answer the game-gateway messages.

### How the client implements it

#### Main classes and entry points

| Class or object | Role | Evidence |
|---|---|---|
| Message id `0xa5` (165), Coin Shop | The client packet switch `009471a0` sends id `0xa5` to a method thunk `00a10a10` (likely), and only when the local game-state object exists (the same `+0x3e0` object the pet family needs). That calls the dispatcher `00c791d0`. | `00c791d0` label unknown, pseudocode read; `00a10a10` likely |
| Coin Shop dispatcher `00c791d0` | Builds a base packet with `00c6e960` (stores `0xa5` at +4 and the sub-opcode at +8), reads the 4-byte header with `00c71d90`, and switches on the sub-opcode: 1, 3, 6, 7, 9, 10, 11, 13, 17, 18. | label unknown; pseudocode read |
| Message id `0x42` (66), Marketplace | `009471a0` sends it to `HandleInGamePurchasePacket` `00b48a10` with no player-object gate. The switch handles sub-opcodes 2, 4 to 9, 11, 13, 15, 17, 19, 21, 23, 24, 26, 28, 31 to 33, 37, 39 to 42 and 44 to 46. | `00b48a10` confirmed |
| `HandlerCoinStore` (Lua event target) | Script events raised by the Coin Shop code: `OnBuy`, `OnSell`, `OnBuyBack`, `OnGift`, `DisplayStoreItems`, `DisplayStoreItemCategory`, `DisplayStoreItemGroup`, `OnItemSortResponse`, `OnItemFilterResponse`, `OnInitializeSearchEngine`. A separate `HandlerMerchant:SetPlayerCoin` pushes the coin total to the merchant window, and `ReceivedItemGift` is raised by the gift handler (its target object is not shown). | strings in `00c78220`, `00c77df0`, `00c764f0`, `00c774a0`, `00c77630`, `00c77810`, `00c76e70`, `00c78c10` (likely) |
| Lua `CoinStoreScreen` (22 entries, table `01828568`) | `CanBuyItem`, `BuyItem`, `SellItem`, `BuyBackItem`, `SelectItem`, `SelectItemCategory`, `GetItemCountInCategory`, `SelectInventoryCategory`, `AddSearchFilter`, `RemoveSearchFilter`, `ClearAllSearchFilters`, `ApplySearchFilters`, `InitializeSearchEngine`, `IsSearchEngineInitialized`, `SearchItems`, `SortItems`, `SelectItemDsForActiveFilters`, `OnCoinStoreOpened`, `GiftItem`, `CreateGiftOrder`, `AddGiftItemToOrder`, `PlaceGiftOrder`. | table decoded from the binary in this pass |
| `CoinStoreDataSource` (ctor `00cf2440`) | Composite UI data source with children `CoinStoreItems`, `SearchResults`, `PlayerSell`, `ActiveFilters`, `TransactionHistory`. | `00cf2440` likely; children confirmed (S0423, S0424) |
| `CoinStoreTransactionHistory` (ctor `01087540`) | Client-side ledger: four record tables of 100 entries each (`01087540` confirmed). A record goes into the table picked by its first dword (`010875e0` confirmed); the oldest is dropped when the table is full; `01087060` reduces a record's count when it is bought back; `01087200` empties all tables. | confirmed |
| `ClientInGamePurchaseManager` (ctor `00b41ac0`) | The Marketplace brain: caches (stores, bundles, currency, country, state, Station Cash SKUs, membership SKUs, accepted card types), the order objects, the wallet and the payment state. | `00b41ac0` confirmed |
| Marketplace startup `00b46b90` | Gate that asks for whatever the client still lacks: account info first, then currency codes, then (country and state lists for billing forms), then the SKU lists. It is also a refetch callback after the caches are cleared (`00b47270`, likely). | `00b46b90` confirmed |
| `ClientInGamePurchaseStoreManager` (ctor `00c41fc0`, `OnDataUpdate` `00c41270`) | Owns the store, bundle, group, category, billboard and shortcut definition maps and reloads them from the data files below. | confirmed |
| Lua `InGamePurchaseStoreScreen` (66 entries, table `01829248`) | Order and wallet calls (see Network), `SelectStoreBundle`, `SelectStoreBundleCategory`, `SelectStoreBundleGroup`, `SelectStoreBundleVipLevel`, three filter toggles (New, Limited Time, Sale), `SelectBillboard`, search and sort, `EquipStoreBundle`, `SaleViewed`, `RequestServerStatus`, `IsMarketplaceEnabled`, `IsStoreAvailable`. | table decoded this pass |
| Lua `MarketplacePortalScreen` (10 entries, table `018289f0`) and `PromotionalScreen` (table `01828c10`) | Portal open and close events, the gamedock, claim and membership screen events; `IsCodeRedemptionEnabled` and `RedeemCode`. | tables decoded this pass |
| `InGamePurchaseSearchEngine` (ctor `00cc4c80`) | Local search over bundle, item and locale text (`00cc50f0`); maps marketplace error codes to locale keys (`00cc5470`). | confirmed |
| `ClientPromotionalManager` (ctor `00a07b80`) | Promotions: a bundle cache, named data sources, code redemption. Message id `0x72` goes to `00a08e60`. | confirmed ctor |
| Merchant windows | Lua modules `MerchantSell` (`Populate`, `SortItems`, `BuyFromMerchant`, `BuyFromUi`, `Close`; `00c0a890`) and `MerchantBuy` (`Populate`, `SellToMerchant`, `SelectInventoryCategory`; `00c0a9a0`), with data sources `00cfd8c0` and `00cfeea0`. Despite the names, the buy-from-merchant calls sit in the module called `MerchantSell`. `Close` ends the interaction by sending a clear-merchant command (`00c0ad30`, likely). | confirmed registrars |
| Payment services | CAS (login ticket and web links) and URAM (card and membership requests) clients, set up by `00d57aa0` and `00d6d4b0`; configured by `00b3df80` (all confirmed). | confirmed |

#### Coin Shop: opening, browsing, buying

1. **Opening.** The Lua call `OnCoinStoreOpened` ends in `00c76d20`, which logs the referrer id and context and sends a dynamic-list update request (sub 8, header only). The server can also push the store open with an Open message (sub 7) naming an item id list, a category or an item group; `00c78890` (confirmed) routes to `DisplayStoreItems`, `DisplayStoreItemCategory` or `DisplayStoreItemGroup` in that order and logs an error if none is set.
2. **Item lists.** The item list (sub 1) carries two lists: static items (kept in a 4096-bucket map) and dynamic items (a 256-bucket list). The client logs the two counts (`00c75fc0`, confirmed). A dynamic update (sub 9) replaces the dynamic list (`00c76100`, confirmed; the code clears the old list first).
3. **Item definitions.** Entries in these lists are only ids and flags (see Data). When the client needs the real item definitions for a page it sends sub 2 with the ids (`00c76b10`, confirmed; builder `00c77410`, likely) and gets sub 3 back. The reply has a completeness flag; if it is false the client logs how many ids it received against how many it expected (`00c77090`, confirmed). Search, filter and sort complete locally once definitions are present and otherwise first request them (`00c77b80`, likely; completion `00c76e70`, confirmed).
4. **Buying.** Lua `BuyItem` reaches `00c77230` (label unknown, pseudocode read). It finds the item in a 1024-bucket table (bucket = item id and `0x3ff`; the entry's id is at +0xb4). A missing item logs "Failed to find item definition" and stops. If the entry's flag byte at +0x9c is zero the quantity is forced to zero. An item id below 1 or a negative quantity logs "Invalid item record" and stops. Otherwise it builds the request (sub 4) and sends it only when the client transport (client +0x280) exists.
5. **Selling.** Lua `SellItem` (`00c77030`, confirmed) calls `RequestSellItem` (`00c76dd0`, confirmed), which fills a sub 5 request and sends it (`00c76820`, confirmed).
6. **Result.** The server answers a buy or sell with a Transaction Complete (sub 6, `00c78220`, label unknown, pseudocode read). The client logs success or failure (with type, result, transaction id, items and quantity), calls into the history ledger, and raises `OnBuy` or `OnSell` (a kind value of 1 or 2 in the payload picks one). The same function also refers to `NpcMerchantV2:OnBuy`, `MerchantPurchased`, `PurchaseFailed` and `NotificationHandler:AlertMessage`; which branch raises which was not traced. The result code goes through `00c6f300` (likely): it switches on codes 1 to 11 and every arm makes the same indirect call, so the code itself does not show which code means what. The client's own result names, in the order the function refers to them, are: Success, Fail, Fail_InvalidItem, Fail_InvalidQuantity, Fail_InvalidMerchant, Fail_InvalidItemTint, Fail_BalanceNotEnoughForPurchase, Fail_ServiceUnavailable, Fail_RecipientNotFound, Fail_NotAMember, Fail_RecipientCanNotAcceptItem (strings in `00c6f300`). Mapping them to 1 to 11 in that order is likely, not proven. The client locale has a coin message for the shortfall: "You don't have enough coins to complete this transaction."
7. **Buy Back.** Lua `BuyBackItem` goes to `RequestBuyBackItem` (`00c76c60`, likely). It looks up the history record by id and only proceeds when the record's first dword (its table index) is 2, probably the sale table (the clear-history log counts purchase and sale entries separately); it then sends sub 12 with the record's transaction id (record +0x10) and one caller value. The reply (sub 13, `00c78a10`, label unknown, pseudocode read) logs transaction id and result; when the result is 1 it voids the transaction in the history (`01087060` decrements the record) and raises `OnBuyBack` with result and id.
8. **Gifts.** Lua `CreateGiftOrder`, `AddGiftItemToOrder` and `PlaceGiftOrder` build a gift order; `RequestGiftPurchase` (`00c79370`, likely) refuses a record whose merchant words equal the session merchant, or whose count is not positive, and sends sub 14 (`00c790a0`, label unknown). A gift result (sub 18, `00c77df0`) logs success or failure and raises `OnGift`. A recipient online gets a gift item message (sub 17, `00c764f0`, confirmed), which raises `ReceivedItemGift` once per entry.
9. **History.** `Clear Transaction History` (sub 11, `00c752d0`, confirmed) logs how many purchase and sale entries it is dropping and empties the ledger.
10. **Merchant list.** Sub 10 (`00c78fb0`, likely) refreshes the player's coin shown in the merchant window and rebuilds the merchant list data (`00cff0b0`, likely). The body was not decoded.

#### Marketplace: startup, catalogue, orders

1. **Startup.** `00b46b90` (confirmed) sends the account info request (sub 25, `00b45010`, label unknown) if no account info is cached. The reply (sub 26, `00b45f60`, label unknown, pseudocode read) sets an account-info flag and then triggers the currency codes request (sub 16, `00b44e60`). The state codes request (sub 18, `00b44fa0`, speculative) is sent when the state list is empty. SKU lists are asked for after account info is set: Station Cash products (sub 14, `00b43c40`, confirmed) and membership products (sub 22, `00b43d10`, confirmed). The wallet request (sub 10, `00b43bc0`, likely) refreshes after any funding or membership result. The code does not require a fixed arrival order and re-requests anything that is still empty.
2. **Enable switches.** Sub 24 carries one byte that enables or disables the Marketplace (`00b3bf30`, confirmed, calls `00b37360`). Sub 33 carries two bytes that enable payment sources, stored at manager +0x3e830 and +0x3e831 (`00b3c000`, confirmed); Lua exposes them as `IsPaypalPaymentSourceEnabled` and `IsSmsPaymentSourceEnabled`, but which byte is which is not read. A server status request (sub 12, ctor `00b23160`, likely) is answered with one byte (sub 13, `00b3bde0`, confirmed).
3. **Catalogue.** The server pushes the catalogue in pieces: store bundles (sub 5, parsed by `00b48700`, label unknown, with an inner tag 1 for a full store and 2 for one bundle update), category groups (sub 6), category tree (sub 7, `00b48040`, confirmed), exclusive partner bundles (sub 8, `00b47f20`, confirmed), bundle groups (sub 9), Item of the Day (sub 32, `00b47690`, confirmed), Bundle of the Day (sub 41, `00b477e0`, confirmed) and sale displays (sub 40; each entry is eight dwords in memory, then `00b39660` picks the active sale). All bundle adds go through `00c40360` (likely). Missing bundle content is requested in batches (sub 27, `00b47b20` and `00b48120`, both likely; the log says it streams items per request) and the answer is most likely sub 28 (dispatcher case 0x1c, handler `00b40050`, which updates the cached bundle table; label unknown).
4. **Local marketing data.** Portal categories, billboards and billboard panels do not come from the network. `00c39c80` points the data updater at `MarketingDataClientSchema.xml`, and three loaders read the tables named `mrkt_store_portal_category_vw`, `mrkt_store_billboard` and `mrkt_store_billboard_panel_vw` in their log strings (`00c3ff10` label unknown, `00c40410` and `00c400c0` confirmed; S0390, S0391). The shipped files that hold those rows are listed in Data and formats.
5. **Preview and place order.** The Lua order calls (`CreateNewOrder`, `AddStoreBundleToOrder`, `AdjustStoreBundleQuantityInOrder`, `AddGiftMessageToOrder`, `AddCouponCodeToOrder`, `PreviewOrder`, `PlaceOrder`, `CancelOrder`) build a `ClientInGamePurchaseOrder` (ctor `00d5e2f0`, confirmed) with a detail map; each detail holds three ids or quantities (`00d77210`, confirmed), which for gift orders are store id, bundle id and quantity. The preview answer (sub 2, `00b3b450`, confirmed) raises `OnOrderPreviewResponse`. The place-order answer (sub 4, `00b3b6d0`, label unknown) routes success to the merchant or item callbacks and failure to `PurchaseFailed`, an alert and the buy-button callbacks. The constructors for the two requests exist (sub 1 `00b2c610`, confirmed; sub 3 `00b243d0`, likely; a place-order ticket, sub 36, `00b2a7f0`, likely) but no sender that serialises them was found, so the request layouts are not known.
6. **Bundle content and equipping.** `EquipStoreBundle` (`00b38ea0`, confirmed) equips already-owned bundle items and reports per-category results; it does not buy. A preview of a bundle on a mannequin or pet goes through `CallStoreItemPreview` (`00b35320`, confirmed) and `StorePreviewDispatch` (`00910da0`, confirmed), which handles kinds for pets, an expired rental, a mannequin and a player inspect.
7. **Wallet and funding.** The wallet answer (sub 11, `00b3e120`, confirmed) marks the wallet ready and raises `OnWalletInfo`. Funding by card or Steam has client paths (`00b33760` likely, `00b30200` likely), and Lua entries exist for PayPal and SMS funding. `BuildFundWalletUrl` (`00d57e30`) and `BuildMembershipUrl` (`00d57f80`) (both confirmed) build web links and `00b38d00` (confirmed) handles the outcome of launching the browser, which "does not establish remote payment completion".
8. **Membership purchase.** `PreviewSubscriptionPurchaseWithStationCash` (`00b33a50`, confirmed) and `...WithCreditCard` (`00b33900`, confirmed) ask for a preview; `RequestPurchaseSubscription` (`00b342f0`, confirmed) starts the purchase and emits a telemetry message (`00ade100`, confirmed) that the server can ignore. The request classes for membership preview and purchase are URAM web-service requests (`DispatchUramRequestByType` `00d6d830`, confirmed). The response callbacks are manager methods (`HandleMembershipPreviewResponse` `00b44370` and `HandleMembershipPurchaseResponse` `00b44af0`, both confirmed) that raise events and trigger a wallet refresh; the code does not show whether the replies arrive over the gateway or the web client. The client handles a membership activation message (id 100), which fires `MembershipHandler:OnMembershipActivated` (client, `009471a0`); when the server sends it is not shown. A subscription info message (id `0xbd`, ctor `0089c200`, confirmed) stores two flag bytes on the player object (+0x468 and +0x469 of the game-state object, pseudocode read). A VIP info message (id `0xba`, ctor `008b1970`, confirmed) writes two dwords on a proxied character and then raises a UI event (likely the loyalty display). The login account info (`00719330`, confirmed) also caches the member flag.
9. **Merchants.** Sub 42 (merchant list; reader `00b40a60`) stores a merchant id pair and a dword, builds the `MerchantSellBundles` data source from a counted list and raises `NpcMerchantV2:Show`. Sub 44 is the answer to a buy; any result other than 1 raises an alert and `NpcMerchantV2:EnableBuyButton`. Sub 45 updates remaining stock (`00b3c0d0`, confirmed: after the header it reads two dwords). Sub 46 raises `NpcMerchantV2:Hide` with no body read. The buy request is sub 43 (ctor `00b2a290`, label unknown); no sender was found. All read from the `00b48a10` switch (confirmed).
10. **Promotions and codes.** Message id `0x72` goes to `00a08e60`: sub 1 is a key-code redemption notice (`00a08d70`, confirmed, mentions loot card codes) and sub 2 is a promotional bundle (`00a08b10`, label unknown). Member-upgrade data sources named `BaseClient.MemberUpgrade.PetTrial` and `...MountTrial` are registered by `00a07ff0` (likely). `RedeemCode` and the select-option step appear to use the URAM request classes (`UramRedeemCodeRequest`, `UramSelectCodeRedemptionOptionRequest`); the result becomes the `OnCodeRedemption` event (`00a08520`, label unknown; `00a088c0`, confirmed).

### Data and formats

**Shipped resource files (read by the client, not sent).** Read from `lab/runs/client-baseline-*/client/Resources` in this pass.

| File | Contents | Source |
|---|---|---|
| `Marketing/MarketingDataClientSchema.xml` | Schema for four client-only tables: `storeBillboard` (billboard_id, location_id), `storeBillboardPanel` (store_billboard_id, name_id, image, image_tint, display_order, shortcut_id, shortcut_type, shortcut_value, store_id, oob_data), `storePortalCategory` (category_id, name_id, description_id, image, image_tint, display_order, shortcut_id, shortcut_type, shortcut_value, store_id) and `configuration` (name, value). | client |
| `Marketing/ClientStorePortalCategories.txt` | Six rows, caret-separated. Category ids 1 to 6, display order 1000 to 1500 in steps of 100, every shortcut of type 3 with store id 1; shortcut values 59, 115, 139, 116, 45, 41. | client |
| `Marketing/ClientStoreBillboards.txt` | Two rows: billboard 1 at location 1, billboard 2 at location 2. | client |
| `Marketing/ClientStoreBillboardPanels.txt` | Six panels: four on billboard 1 (display orders 1, 1, 2, 3) and two on billboard 2 (display orders 0 and 200). Image ids 56, 55, 91, 6900, 49, 48. Five panels use shortcut type 3 (values 116, 59, 83, 139, 59) and one, on billboard 2, uses type 4 with value 0. The `oob_data` is an XML snippet with `view_requirement_id`, `content_id` (91 and 92 on two panels, 0 elsewhere) and `is_members_only` (1 on the last row only). | client |
| `Marketing/Configuration.txt` | Empty. | client |
| `MarketingBillboards.txt` | One row: image id 2, display order 1 (loader not mapped). | client |
| `ItemGroupPromos.txt` | Header line only, no rows. | client |

The name ids in these tables are glossary ids and were not resolved to text. `00c387f0` (speculative) validates shortcuts: type 1 resolves a bundle, type 2 a store entry, type 3 a category key; other types log an error. What type 4 means is unknown.

**Data-source columns (names present in the client; row order is not recovered).** `StoreBundles` (also used by group, VIP-level, Item of the Day, Bundle of the Day and merchant bundle sources): ItemId, StoreId, CategoryId, Description, Price, ImageSetId, ImageTintValue, IsMembersOnly, IsOnSale, SalePrice, IsRentable, RentalPeriod, RentalPrice, HasTermsAndConditions, TermsAndConditions, MaxPerOrder, NumEntriesTotal, ItemListDetails, FirstItemId, HasContentDefinitions, LimitedEditionId, DisplayUseItNow, EquippableState, CanPreview, MaxPurchasable, IsNewItem, IsViewOnly, IsTintable, AvailableTintGroupId, TintId, GenderUsage, MembersOnlyPrice, VipRankRequirement (`00997840`, confirmed). Item of the Day and Bundle of the Day add QuantityAvailable, TimeRemaining, IOTDPrice and Message (`00997b60`, confirmed). `StationCashPriceList`: TotalStationCashAmount, PromotionalStationCashAmount, PromotionalStationCashGranted, RetailPrice, SalePrice, IsOnSale (`00997eb0`). `SubscriptionPriceList`: Description, PriceLocalCurrency, PriceStationCash, IsVatTaxable, MarketingDescription, ProductLegalText, Highlight, SortOrder (`00997f20`). Other lists: category groups, portal categories, billboard panels (BillboardId, ImageId, IsMembersOnly, ShortcutType, ShortcutValue, IsVislble), gift order history (PurchaserName, PurchaserGuid, GiftMessage, TimeStamp, ItemEntryCount), gift order entries (StoreId, StoreBundleId, Quantity), credit cards, country codes (CountryCode, LanguageLocale, CountryLocale, CountryName) and state codes (StateName, StateCode). Coin Shop items expose: Coin Purchase Price, Coin Sell Price, Item Guid List, No Sale, Can Preview, Available Tints Group ID, Member Purchase Price (`00cf07f0`, confirmed). The coin transaction history columns are MerchantGuid, ItemId, Quantity, TintId, Price, BoughtBack, QuantityRemaining (`00cf0a80`, confirmed).

**Record sizes in client memory (not wire sizes).** Membership SKU 0x214 bytes (`00b250d0`, confirmed); the country reader steps 128-byte records and the state reader 72-byte records (`00b3d930`, `00b3d9c0`, confirmed); a further record array uses 0x60-byte records (Station Cash SKUs or card attributes, not settled; S0340, S0342).

**Coin Shop wire header.** Every Coin Shop message starts with two signed 16-bit values: the family (`0xa5`) and the sub-opcode (`00c71d90`, label speculative, pseudocode read; writer `00c75580`, label unknown). The Marketplace family uses the same two 16-bit reads (`00b2dd60`). This differs from the pet family's header. Byte order on the wire is unverified.

**Coin Shop item entry (inside sub 1 and sub 9).** Each list is an i32 count followed by entries. An entry is a u32 key, then three u32 and three u8 booleans (`00c71510`, pseudocode read, 19 bytes). The list element type is named `ItemDefinitionMetaData` in the client's template names (S0400). The field meanings are not recovered, so it is not known whether the coin price, sell price, member price and No Sale flag listed above travel in this entry or in the item definitions of sub 3.

**Coin Shop transaction payload (sub 6).** After the header: u32 at +0xc, u32 at +0x10, then a record read by `00c71f30`: u32 kind (1 selects `OnBuy`, 2 selects `OnSell`), u64, u32, u64, a nested block (`008cb540`), then three u32. The client's log calls the fields transaction id, result, items and quantity; their exact order is not settled (label of `00c74ef0` unknown).

**Marketplace account info (sub 26).** After the header: u32 status (1 means success), then two length-prefixed strings, then one flag byte (`00b2e080`, label unknown). The log prints the strings as a country and a currency, and the byte as a number; a likely reading is default country, default currency and a parental-password flag.

**Marketplace wallet info (sub 11).** After the header: u32 status, then a record of u8, u32, u32, length-prefixed bytes, length-prefixed bytes and a final u8 (`00b3a6e0`, `00b2b450`, label unknown). The meaning of the fields is not recovered.

### Network

**Envelope.** Coin Shop messages are message id `0xa5` and Marketplace messages are `0x42`. Outbound messages are built into a large serializer buffer, wrapped in `PacketTunneledClientPacket` (or its world variant) and sent only when the client transport at client +0x280 exists (S0401). The exact framing of the tunnel is outside this page.

**Coin Shop (family 0xa5).** The class names come from the client's RTTI. Note the naming: "SellToClient" is the shop selling to the player (a purchase) and "BuyFromClient" is the player selling.

| Sub | Class | Direction | Body as read or written | Evidence |
|---|---|---|---|---|
| 1 | ItemList | server to client | Two lists (static, dynamic) as above. | ctor `00c6f440` confirmed; handler `00c75fc0` confirmed |
| 2 | ItemDefinitionsRequest | client to server | i32 count, then count x u32 item ids. | ctor `00c6ea90`, sender `00c76b10` confirmed |
| 3 | ItemDefinitionsResponse | server to client | u8 flag (likely a completeness flag, from the log text), then item definitions in the item system's format. | ctor `00c6ebc0`; handler `00c77090` confirmed; reader `00c74e40` |
| 4 | SellToClientRequest | client to server | u64 merchant id, u32 caller value, u32 item definition id, u32 quantity, u32 caller value (28 bytes with header). | ctor `00c6fef0`, sender `00c76a00` confirmed; body `00c75ed0` and `00c77230` pseudocode read |
| 5 | BuyFromClientRequest | client to server | u64 merchant id, u32 item instance id (one word), u32 quantity (20 bytes with header). | ctor `00c6ff90`, sender `00c76820` confirmed; body `00c75a70` and `00c76dd0` pseudocode read |
| 6 | TransactionComplete | server to client | Two u32, then the record above. | ctor `00c6eda0` confirmed; handler `00c78220` label unknown |
| 7 | Open | server to client | An item id list, a category structure, a u32 item group id, and two u32 passed unchanged to all three display routes (likely the referrer id and context named in the open log). The handler uses the first non-empty of id list, category, item group (offsets read from `00c74f70` and `00c78890`; likely). | ctor `00c74490`, handler `00c78890` confirmed; body `00c74f70` label unknown |
| 8 | DynamicListUpdateRequest | client to server | Header only. | ctor `00c6ee60`, sender `00c76710` confirmed |
| 9 | DynamicListUpdateResponse | server to client | Dynamic list (count and entries). | ctor `00c6f500`, handler `00c76100` confirmed |
| 10 | MerchantList | server to client | Not decoded. | ctor `00c6ef90` confirmed; handler `00c78fb0` likely |
| 11 | ClearTransactionHistory | server to client | Header only. | ctor `00c6f040`; handler `00c752d0` confirmed |
| 12 | BuyBackRequest | client to server | u32 transaction id, u32 caller value (12 bytes with header). | ctor `00c701d0`, sender `00c76600` confirmed; body `00c759c0` |
| 13 | BuyBackResponse | server to client | Three u32: transaction id, result (1 is success), a third value passed on. | ctor `00c702b0` confirmed; reader `00c71e40`; handler `00c78a10` |
| 14 | SellToClientAndGiftRequest | client to server | u64 merchant id, u64 recipient id, i32 count, then count x (u32, u32) entries (item id and quantity by analogy with sub 4). | ctor `00c75130`; sender `00c790a0`, writer `00c764b0`, entries `00c75b30` (all label unknown or likely) |
| 17 | ReceiveGiftItem | server to client | As decompiled: a u64, an i32-length block, a u64 into the same field, then one record (u64, u64, nested block); layout not settled. | ctor `00c74590`; handler `00c764f0` confirmed; reader `00c758a0` |
| 18 | GiftTransactionComplete | server to client | u32 result, then one record (u64, u64, nested block). | ctor `00c743c0`; handler `00c77df0`; reader `00c76390` |

**Marketplace (family 0x42).** Sub-opcode numbers are those in constructors and in the `00b48a10` switch.

| Sub | Direction | What it is | Evidence |
|---|---|---|---|
| 1 | client to server (unproven) | Preview order request | ctor `00b2c610` confirmed |
| 2 | server to client | Preview order response (four dwords in the constructor `00b27e20`, likely) | handler `00b3b450` confirmed |
| 3 | unproven | Place order (holds an order pointer) | ctor `00b243d0` likely |
| 4 | server to client | Place order response | ctor `00b2f8d0` confirmed; handler `00b3b6d0` label unknown |
| 5 | server to client | Store bundle base (inner tag 1 full store, 2 single bundle) | `00b48700` label unknown |
| 6, 7, 9 | server to client | Category groups, category tree, bundle groups | `00b44d70`, `00b48040`, `00b45560` |
| 8 | server to client | Exclusive partner bundles | `00b47f20` confirmed |
| 10, 11 | client to server, server to client | Wallet info request and response | `00b43bc0` likely; `00b3e120` confirmed |
| 12, 13 | client to server, server to client | Server status request; response is one byte | `00b3bde0` confirmed |
| 14, 15 | client to server, server to client | Station Cash products request and SKU list | `00b43c40`, `00b45660` confirmed |
| 16, 17 | client to server, server to client | Currency codes | `00b460c0` likely |
| 18, 19 | client to server, server to client | State codes | `00b44fa0` speculative; `00b46a50` |
| 20, 21 | client to server, server to client | Country codes | `00b46200` likely |
| 22, 23 | client to server, server to client | Membership products request and SKU list | `00b43d10`, `00b45ba0` confirmed |
| 24 | server to client | Enable Marketplace (one byte) | `00b3bf30` confirmed |
| 25, 26 | client to server, server to client | Account info request (carries the client locale string); response above | `00b45010`; `00b45f60` |
| 27, 28 | client to server, server to client | Bundle content request and response | `00b43230` confirmed; `00b40050` |
| 29 | unproven | Client statistics (one dword in the constructor) | `00b23fc0` likely |
| 31 | server to client | Mannequin store bundles (bundle id set, preview kind 5) | `00b3e340` confirmed |
| 32, 41 | server to client | Item of the Day, Bundle of the Day | `00b47690`, `00b477e0` confirmed |
| 33 | server to client | Enable payment sources (two bytes) | `00b3c000` confirmed |
| 36 | unproven | Place order client ticket | `00b2a7f0` likely |
| 37 | server to client | Gift order notification (raises `OnReceiveGiftOrder`) | `00b41fb0` confirmed |
| 38, 39 | client to server, server to client | Item requirements update request and reply | `00c047a0`, `00b3e490` confirmed |
| 40 | server to client | Sale display update | ctor `00b2c1a0` confirmed |
| 42 to 46 | mixed | Merchant list, buy-from-merchant request and response, stock update, hide | see above |

The Lua `Opening` entry in the `InGamePurchaseStoreScreen` table points at `00c047a0`, which is labelled as the item-requirements request sender; the name mismatch is noted but not explained.

**Other families.** Promotions are message id `0x72` (sub 1 key-code notice, sub 2 bundle). Membership activation is id 100; subscription info is `0xbd`; VIP info is `0xba`. Bodies are not decoded beyond what the Marketplace section states.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Scope.** Build the Coin Shop fully; it is the plain coin economy. Make the Marketplace usable without payment (R): a wallet balance the server owns, a catalogue, and no card, PayPal, SMS or Steam path.
2. **Gate and framing.** Send Coin Shop messages only after the player object exists (id `0xa5` is dropped before that). Use the two 16-bit header and mirror the byte layouts above. Capture one real client session against the stub to confirm byte order, because no traffic was ever captured (R).
3. **Coin ledger.** Persist coins per character. On buy (sub 4): check merchant, item, quantity, tint, membership and balance, then reply sub 6 with a result and a transaction id, grant the item and record the purchase. Use the client's result names above. On sell (sub 5): price from the item data, refuse items marked No Sale, record the sale so it can be bought back. On buy back (sub 12): accept only sale records, reply sub 13 with result 1 on success. Clear history (sub 11) is available. Gifts (sub 14): check the recipient, then send sub 18 to the sender and sub 17 to the recipient.
4. **Prices.** The client has fields for purchase price, sell price, member purchase price and a No Sale flag. Whether they travel in the item list entries (sub 1) or in the item definition records (sub 3, see [[Items and Inventory]]) is unresolved, so check with a captured stub session. The video shows member prices about 10 percent below regular (4,999 to 4,499; 9,999 to 8,999). Treat 10 percent as the default member discount (R); the coin price table itself is a content table the client does not ship.
5. **Catalogue content.** Item lists (sub 1, 9) and definitions (sub 3) need real item ids and definition records. Choose what is sold by category and item group; the wiki Coin Shop page is a roster to start from (flair shards, style cards, recipes, farming and crafting supplies, and so on). Open the store from a script or NPC with sub 7.
6. **Marketplace handshake.** Answer, in a sane order: account info (sub 26, status 1, a country and currency string, a flag), currency codes (17), Station Cash SKU list (15, may be empty), membership SKU list (23, may be empty), wallet (11), enable Marketplace (24 set), payment sources (33, both bytes zero) (R). With an empty SKU list the client shows its own "cannot obtain packages" message. State and country codes are only needed for the billing forms; send small valid lists or empty ones (R).
7. **Orders (R).** The preview and place-order request layouts are unknown. Use the real client against a stub to log the bytes, then build sub 2 and sub 4. A purchase debits the SC balance, grants the items, and answers with a result the client maps to success or failure.
8. **Catalogue pushes.** Send store bundles (5), category tree (7), category groups (6), groups (9), Item of the Day (32), Bundle of the Day (41), sale displays (40) and content (28). Billboards and portal categories come from the shipped files, so the server should keep its bundle category ids consistent with the shortcut values in them (portal values 59, 115, 139, 116, 45, 41; panel values 116, 59, 83, 139) or the type 3 shortcuts probably point nowhere (the validator treats type 3 as a category key, `00c387f0`, speculative).
9. **Merchants.** For the NPC vendor window use sub 42 (list), 44 (answer to a buy), 45 (stock) and 46 (hide). Stock is limited and drops with each sale (R). The coin vendors use the Coin Shop messages with the vendor's id.
10. **Rentals (R).** Support `IsRentable`, `RentalPeriod` and `RentalPrice`, expire the item server-side and send the preview kind the client uses for an expired rental.
11. **Membership (R).** Do not take payments. The login account info carries the member flag (client `00719330`; the login server is outside this page); also provide membership activation (id 100), subscription info (`0xbd`) and VIP info (`0xba`). A simple, honest option is to treat every account as a member, as the original game did from 2014-02-04 (wiki); a configurable flag keeps the member-only gates testable. Leave the web links unconfigured.
12. **Promotions and codes (R).** Stub the Claim window: an empty claim list. Redemption goes through URAM request classes, not the gateway, so a code system needs either a local service or a client change; do not promise it until that path is traced.
13. **Errors.** Every refusal should end in a visible reply (a result code, a failure alert, a status byte), never silence.

### Video and wiki evidence

**Video frames reviewed:** the frames that show a Coin Shop or Marketplace panel, item tooltips with sell values, claim windows, Item of the Day panels and ad cards. Names over characters are omitted.

- **Coin Shop prices (video).** The Coin Shop detail panel shows a grey regular price, a red member price and "You saved": 4,999 / 4,499 (saved 500) for one house item, and 9,999 / 8,999 (saved 1,000) for another (`1TNyzyI62nc/00270`, `1TNyzyI62nc/00254`, `1TNyzyI62nc/00252`). The vendor is not named on screen. A related list view shows pet feeding bowls at 15 coins regular and 13 for members ([[Pets]]).
- **Station Cash prices (video).** Station Cash item panels show a struck-through regular price, a Member Discounts line of about 10 percent, a Your Price and Total Price line, and an Add Station Cash button (`1TNyzyI62nc/00122`, `1TNyzyI62nc/00132`, `1TNyzyI62nc/00152`, `g7pncDFzIrw/00230`, `g7pncDFzIrw/00248`). Example prices: 89, 134, 135, 270 and 899 SC. The Item of the Day panel shows a 200 SC price on a pet (`WSpXvhq_VCY/00266`), a 550 / 275 rides price whose sale value is unclear (`fjZluIfbUIo/00084`), a house at 375 SC (`d2tpgGq1hxE/00104`) and a card booster pack at 249 SC in an ad (`fjZluIfbUIo/00058`). An ad card gives rides as "From 450 - 550 Each" (`e4XXSnnive0/00236`).
- **Sell values (video).** Inventory tooltips show "Resell Value" with a coin icon: 27 for several common clothes (`zNMmxK6l5jI/00190`, `6mmERh2D8yc/00110`), 75 for a jacket (`Sa3wS4GrqKw/00068`), and 660 for a power shard that also says Cannot Sell and Cannot Trade (`Qg7_4br2H80/00448`). Empty inventory tabs show a "Visit the Marketplace or Coin Shop" hint (`PvUWhApOxWQ/00092`).
- **Claim window (video).** A code field and a Pending Claims list with a CLAIM button; the text says each claim item can only be claimed once per account (`4h2eVoje3Eg/00288`, `4h2eVoje3Eg/00290`, `4h2eVoje3Eg/00294`).
- **Membership upsell (video).** "Upgrade to a Membership" for 4.99 dollars a month, with more jobs, leaderboards, three characters and extra items and quests (`yl6WBIErAsY/00028`). Many world frames show an Upgrade prompt at the screen edge, and shop windows show a "Lifetime Membership $34.99" banner.
- **Bank (wiki).** The wiki says bank slots can be bought with coins in ten ordered tiers (2 slots for 1,000 coins up to 100 slots for 50,000, per the template values) or with SC (a 50-slot upgrade for 100 SC). The tier text is not fully clear, so check it before using it. Bankers are NPCs in Seaside, Sanctuary and Snowhill (wiki).
- **VIP (wiki).** Members earn VIP points daily; ranks 1 to 5 unlock titles, marketplace items and wearing costumes across jobs. The client has a `VipRankRequirement` column and a VIP bundle view (client), which fits.
- **Coin sources (wiki, client locale).** Coins come from quests and dungeons (wiki) and from the Coin Wheel (client locale: spin it daily to win coins to spend at the Coin Shop). The locale also has entries named "1,000 coins for Coin Shop", "10,000 ..." and "100,000 ..." (use unknown).
- **Station Cash sales (client locale).** Sales run as banners: 30 percent off for members, 50 percent off pets or all bundles, double SC from store cards, and a bonus SC day (up to 4,000 SC). The wiki records Marketplace prices dropped to 60 percent of regular from 2014-01-28 to 2014-03-17.
- **Platform text (client locale).** The locale has PlayStation wallet variants of the Marketplace text; the Windows client uses the Station Cash text.

### Open questions and contradictions

- **No captured traffic.** The header (two 16-bit values), all body layouts and the byte order are read from the client only. Treat every layout above as unverified until a stub session is logged.
- **Name.** The client's locale says "Coin Shop" almost everywhere and "Coin Store" once; the code, logs and packet classes say `CoinStore`. This page uses "Coin Shop" for the feature and `CoinStore` for code names.
- **Label versus code.** The label of `00c78a10` says the buy-back response has two dwords; `00c71e40` reads three after the header. The label of `00c71d90` is speculative, but the code is plain.
- **Result codes.** The mapping of codes 1 to 11 to the client's result names is inferred from reference order, not proven (`00c6f300`).
- **Transaction payload.** The order of the two dwords and the record fields in sub 6 is not settled, and the item-list entry fields (3 u32 and 3 booleans) are unnamed.
- **Unknown caller values.** Subs 4 and 12 carry caller-supplied dwords (`BuyItem` is described as taking three callback values). What the server is meant to do with them is unknown.
- **Order requests.** No sender was found for Marketplace sub 1, 3, 36, 43 or 29. The place-order and buy-from-merchant layouts are unknown.
- **Merchant list and open body.** Coin Shop sub 7 and 10 are only partly decoded; the exact meaning of the three trailing dwords in sub 7 is open.
- **Local data versus network.** A shard note says the store definitions are fed from local marketing files; the dispatcher shows bundles, groups and categories also arrive by sub 5, 6, 7 and 9. This page reads it as: portal categories and billboards are local, bundles are server-sent.
- **Shortcut type 4.** One billboard panel uses type 4 with value 0; its meaning is not recovered.
- **Merchants.** Whether the wiki's travelling merchant uses the `NpcMerchantV2` messages is an inference; the currency of `NpcMerchantV2` purchases (coins or SC) is not read.
- **SC packs.** The wiki lists six price points (5, 10, 20, 25, 45 and 85 dollars) with no numbers beside them in the page text. The template values 500, 1,000, 2,000, 2,500, 5,000 and 10,000 pair with them in order, which would mean bonus SC on the two largest packs, and a first template value of 100 matches "1 dollar". That is an inference from stripped template data.
- **Bank tiers.** The wiki's coin tiers are not fully clear.
- **Statistics.** Session statistics (`00b40630`) and a push of the store data at the end of a store session (`00b485e0`, speculative; `00b49090`, label unknown) are mapped, but whether any of it is sent to the server is unproven.
- **Codes.** Whether code redemption results arrive over HTTP (URAM) or the gateway, and what the redemption request looks like, is open.
- **Tutorial text.** The tutorial strings disagree about the Marketplace icon (a shopping cart, or the marketplace icon on the dock). This is cosmetic.

### Key functions

| Address | Role |
|---|---|
| `009471a0`, `00a10a10`, `00c791d0`, `00c6e960`, `00c71d90` | Client packet switch, Coin Shop thunk (likely), dispatcher, base packet, 16-bit header reader |
| `00c77230`, `00c76a00`, `00c75ed0` | Coin Shop buy: builder, sender (confirmed), body writer |
| `00c77030`, `00c76dd0`, `00c76820`, `00c75a70` | Coin Shop sell: Lua entry, builder, sender, body writer |
| `00c76c60`, `00c76600`, `00c759c0`, `00c78a10` | Buy back: request builder (likely), sender, body writer, response handler |
| `00c78220`, `00c74ef0`, `00c71f30`, `00c6f300` | Transaction complete handler, body reader, record reader, result dispatcher (likely) |
| `00c78890`, `00c74f70`, `00c774a0`, `00c77630`, `00c77810` | Open store packet and its three display routes |
| `00c75fc0`, `00c76100`, `00c76b10`, `00c77090`, `00c76e70` | Item list, dynamic update, definitions request, definitions response, search completion |
| `00c79370`, `00c790a0`, `00c764f0`, `00c77df0`, `00c752d0` | Gift request, sender, gift item, gift result, clear history |
| `01087540`, `010875e0`, `01087060`, `01087200` | Transaction history ledger (confirmed) |
| `00b48a10`, `00b41ac0`, `00b46b90` | Marketplace dispatcher, manager, startup gate (confirmed) |
| `00b45010`, `00b45f60`, `00b44e60`, `00b43c40`, `00b43d10`, `00b43bc0`, `00b3e120` | Account info, currency, SKU and wallet requests and answers |
| `00b48700`, `00b48040`, `00b47690`, `00b477e0`, `00b40050` | Bundle base, categories, Item of the Day, Bundle of the Day, content |
| `00b3b450`, `00b3b6d0`, `00b3bf30`, `00b3c000`, `00b3e490` | Order preview and result, enable switches, item requirements reply |
| `00b342f0`, `00b33a50`, `00b33900`, `00d57e30`, `00d57f80`, `00b38d00` | Membership purchase and preview, payment web links |
| `00c41fc0`, `00c41270`, `00c39c80`, `00c3ff10`, `00c400c0`, `00c40410` | Store manager and the local marketing data loaders |
| `009a8f60`, `00cc4c80`, `00cc50f0` | Marketplace data source registry and search engine |
| `00a08e60`, `00a08d70`, `00a08b10`, `00a07b80`, `00a07ff0` | Promotions dispatcher, key-code notice, bundle, manager, named sources |
| `0089bab0`, `0089c200`, `008b1970`, `00719330` | Membership activation, subscription info, VIP info, account info |
| `00c0a890`, `00c0a9a0`, `00c0ad30` | Merchant Lua modules and clear-merchant sender |
<!-- NARRATIVE END -->

## Mapped functions (generated)

1314 accepted functions: 685 confirmed, 309 likely, 178 speculative, 142 unknown. Network-linked (`wire`): 118.

Subsystems: `game.ingamepurchase` (693), `game.coinstore` (165), `ui.coinstore` (162), `net.ingamepurchase` (137), `net.coinstore` (118), `net.promotions` (39)

### `game.coinstore`

**AppStoreBillboardPanelDefinition** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01086320` | AppStoreBillboardPanelDefinition::CtorArgA | confirmed | Thiscall with one stack arg (passed to base constructor 0110aec0), ret 4. Calls 0110aec0 on this, stores AppStoreBillboardPanelDefinition::vftable at +0x0, zer… |
| `010863e0` | AppStoreBillboardPanelDefinition::CtorDefault | confirmed | Fastcall ctor, ECX = this, no stack args, returns this. Calls 0110af40 on this, stores AppStoreBillboardPanelDefinition::vftable at +0x0, zeroes dwords +0x48, … |
| `01086450` | AppStoreBillboardPanelDefinition::CtorArgB | confirmed | Thiscall with one stack arg (passed to base constructor 0110b080), ret 4. Calls 0110b080 on this, stores AppStoreBillboardPanelDefinition::vftable at +0x0, zer… |
| `01086500` | AppStoreBillboardPanelDefinition::DtorBody | confirmed | Fastcall dtor body (SEH-framed), ECX = this. Stores AppStoreBillboardPanelDefinition::vftable at +0x0, then calls 0110afb0 (base cleanup) on this under the fra… |
| `01086550` | AppStoreBillboardPanelDefinition::DtorAndFreeIfFlag | confirmed | Thiscall with one byte flag arg, returns this (eax). Stores AppStoreBillboardPanelDefinition::vftable at +0x0, calls 0110afb0 (base cleanup) on this, then if b… |
| `00c35140` | AppStoreBillboardPanelDefinitionCtorSetId | likely | Runs a constructor helper for a billboard-panel node, copies the input integer into field offset 100, and returns this. |
| `010863a0` | AppStoreBillboardPanelDefinition::PredicateIdsAndSubObjects | likely | Fastcall, ECX = this, returns a byte-sized bool in eax. Returns 0 unless the dwords at this+0x4 and this+0x8 are both non-negative; then calls vtable slot 0 of… |
| `010865c0` | AppStoreBillboardPanelDefinition::FormatDescription | likely | Thiscall with one stack arg (output string), ret 4. Calls 0110b120 into a local string and formats 'storeBillboardId=%d, imageId=%d, contentId=%d, isMembersOnl… |

**ClientInGamePurchaseManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b471e0` | InitializeStoreDataManager | confirmed | Returns 0 when the store data manager is already flagged ready at [this+0x3e828]; otherwise it registers RequestStartupDataInternal (00b46b90) as a callback (F… |
| `00b47270` | ClearStoreCachesAndRequestStartupData | likely | Frees the store node list at [this+0x3b8ec] with FUN_00b3a930, then frees the five cached arrays/maps at [this+0x3ca34], [this+0x3ce08], [this+0x3d61c], [this+… |
| `00b485a0` | BeginStoreStreaming | likely | Calls StreamStoreItems (00b48120) and then sets the store-streaming flag byte at [this+0x3e82d] to 1. |

**ClientInGamePurchaseStoreManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c41270` | ClientInGamePurchaseStoreManager::OnDataUpdate | confirmed | Logs reload start, clears old portal-category and billboard-panel entries, reloads both definition sets from the supplied source, runs post-load refreshes, and… |
| `00c41fc0` | ClientInGamePurchaseStoreManagerCtor | confirmed | Installs MarketingDataHandler then store-manager vtables, initializes logging and all owned store definition maps, billboard maps, and item/bundle-of-the-day o… |
| `00c42100` | ClientInGamePurchaseStoreManagerDtor | confirmed | Destroys billboard and store maps, item/bundle-of-the-day objects, the store-ID set map and logging state, then restores the MarketingDataHandler base vtable. |
| `00c42240` | ClientInGamePurchaseStoreManager::ScalarDeletingDtor | speculative | Calls the manager destructor and frees the object when flag bit 0 is set. |

**CoinStoreTransactionHistory** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01087060` | CoinStoreTransactionHistory::DecrementRecordCount | confirmed | Thiscall with two args (record id, amount); returns 1 or 0 (eax). Looks the record up by id through 01086f00; if found and the amount is 0 with the count at re… |
| `01087200` | CoinStoreTransactionHistory::ClearAllTables | confirmed | Fastcall, ECX = history object, no stack args. For each of four 100-bucket tables (heads at this+0xc, stride 0x1a4), calls 01086fb0 on the head node until the … |
| `01087370` | CoinStoreTransactionHistory::DtorBody | confirmed | Fastcall dtor body, ECX = this. Stores CoinStoreTransactionHistory::vftable at +0x0, calls 01087200 (ClearAllTables) on the object, then runs the array destruc… |
| `01087540` | CoinStoreTransactionHistory::Ctor | confirmed | Fastcall ctor, ECX = this, returns this. Stores CoinStoreTransactionHistory::vftable at +0x0, constructs four 0x1a4-byte elements at this+0x4 through the EH ve… |
| `010875c0` | CoinStoreTransactionHistory::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Calls 01087370 (CoinStoreTransactionHistory dtor body) on this, then if bit 0 of the flag is set calls the free thunk 0… |
| `010875e0` | CoinStoreTransactionHistory::AddRecordByTableIndex | confirmed | Thiscall with one stack arg (record pointer), ret 4. Reads the dword at record+0x0 as a table index k; for k in 1..5 the table base is this+(k-1)*0x1a4+4. If t… |

**GameCommerce::BaseDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5c1c0` | BaseDefinition::ScalarDeletingDtorAlign0 | confirmed | Virtual deleting destructor (ECX = object, one flag argument): stores the GameCommerce::BaseDefinition vtable at [this] and calls 00401c00 (FreeMemory, alignme… |
| `00d5cc70` | BaseDefinition::ScalarDeletingDtorAlign0 | confirmed | Virtual deleting destructor (ECX = object, one flag argument): stores the GameCommerce::BaseDefinition vtable at [this] and calls 00401c00 (FreeMemory, alignme… |

**GameCommerce::MarketingBundleDefinition** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5ab50` | MarketingBundleDefinition::VirtSlot1FormatText | confirmed | Virtual slot 1 of the MarketingBundleDefinition vtable: builds a 1024-byte text accumulator by calling each entry's virtual slot 1 (vtable +4, the Entry text 0… |
| `00d5bc20` | MarketingBundleDefinition::CtorBody | confirmed | Constructor body (ECX = object): installs the MarketingBundleDefinition vtable, sets the words at +8/+0xc/+0x10/+0x14 from the constants at DAT_01bf2d70/DAT_01… |
| `00d5bd30` | MarketingBundleDefinition::CtorWithId | confirmed | Constructor body with one argument (ECX = object, stack argument param_2): same initialisation as 00d5bc20 (vtables, constants at +8..+0x14, 00d672e0 on the im… |
| `00d5be40` | MarketingBundleDefinition::CopyCtor | confirmed | Copy constructor body (ECX = new object, stack argument = source bundle): installs the MarketingBundleDefinition vtable, calls 00d672e0 on the embedded image m… |
| `00d5bef0` | MarketingBundleDefinition::DtorBody | confirmed | Destructor body: installs the MarketingBundleDefinition vtable, destroys the entry list member at +0x4c through 00d5bb50, releases the IString member (vtable a… |
| `00d5bf90` | MarketingBundleDefinition::VirtSlot2DeletingDtor | confirmed | Virtual slot 2 of the MarketingBundleDefinition vtable: calls the destructor body 00d5bef0 on ECX and, when bit 0 of the flag argument is set, calls 00401c00 (… |
| `00d5b0a0` | MarketingBundleDefinition::AssignFrom | likely | Copy-assigns a marketing bundle from the source at param_2 into this: ints +0x9c, +0xa0, +0xa4, +0xa8, +0xac, +0xb0, +0xb4, +0xb8, +0xbc, +0xc0 and +8, +0xc, +… |
| `00d5a7e0` | MarketingBundleDefinition::PredicateAllEntries | speculative | Returns a byte: 0 unless the ints at this+0x9c and this+0xa0 are positive, the virtual call through the object at this+0x18 (vtable slot 0) returns nonzero, th… |

**GameCommerce::MarketingBundleDefinition::Entry** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5a8d0` | MarketingBundleDefinition::Entry::DtorBody | confirmed | Destructor body: stores the Entry vtable at [this], then the IString<char> vtable at +0x10; if the count at +0x1c is positive it decrements the reference count… |
| `00d5a990` | MarketingBundleDefinition::Entry::CtorBody | confirmed | Constructor body (ECX = object): stores the Entry vtable at [this], zeroes the ints at +4, +8, +0xc, +0x18, +0x1c, sets the string member's buffer pointer at +… |
| `00d5a9c0` | MarketingBundleDefinition::Entry::VirtSlot2DeletingDtor | confirmed | Virtual slot 2 of the Entry vtable, with the destructor body inlined: restores the Entry vtable, releases the string buffer through the IString vtable slot 2 w… |
| `00d5ab20` | MarketingBundleDefinition::Entry::VirtSlot1FormatText | confirmed | Virtual slot 1 of the Entry vtable: calls 00402fd0 (SoeUtil::IStringFormat) on the output object at param_2 with the format 'marketingItemId=%d, gameItemId=%d,… |
| `00d5aef0` | MarketingBundleDefinition::Entry::CopyCtor | confirmed | Copy constructor (ECX = new object, stack argument = source Entry): installs the Entry vtable, sets the IStringNoShare<char> vtable at +0x10 with the shared em… |
| `00d5a920` | MarketingBundleDefinition::Entry::PredicatePositive4And8 | speculative | Virtual slot 0 of the Entry vtable: returns 1 when the ints at this+4 and this+8 are both greater than 0, else 0. Bytes: a short function with no calls. |

**GameCommerce::MarketingDataHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c35710` | GameCommerce::MarketingDataHandler::~MarketingDataHandler | confirmed | Stores the MarketingDataHandler vtable in the object and returns without other field work. |
| `00c35720` | GameCommerce::MarketingDataHandler::ScalarDeletingDtor | confirmed | Stores the MarketingDataHandler vtable, conditionally frees the object when the low flag bit is set, and returns this otherwise. |

**GameCommerce::StoreBillboardDefinition<AppStoreBillboardPanelDefinition>** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c37430` | StoreBillboardDefinitionDtor | confirmed | Installs the derived billboard-definition vtable, destroys its array member, then restores the BaseDefinition vtable. |
| `00c37490` | StoreBillboardDefinition::IsValid | confirmed | Requires positive fields at offsets 4, 8 and 0x14, then calls each panel's first virtual method and returns false if any panel rejects. |
| `00c375d0` | StoreBillboardDefinition::ScalarDeletingDtor | confirmed | Destroys the billboard definition and frees it through 00401c00 with alignment 0 when the low flag bit is set. |
| `00c384e0` | StoreBillboardDefinitionCtor | confirmed | Installs the BaseDefinition and derived billboard vtables, clears two fields, and constructs the panel array member. |
| `00c38cb0` | StoreBillboardFormatDebug | confirmed | Formats each panel into a temporary string, appends those strings, then formats billboard id, location id, panel count and panel text into the output string. |
| `00c38f80` | StoreBillboardDefinitionSetId | likely | Constructs a billboard definition node, copies the input integer into field offset 0x198, and returns this. |
| `00c380a0` | StoreBillboardDefinitionScalarDeletingDtor | speculative | Calls the billboard-definition destructor and frees the object allocation when the low flag bit is set. |

**GameCommerce::StoreBundleCategoryDefinition** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5cab0` | StoreBundleCategoryDefinition::DtorBody | confirmed | Destructor body: stores the StoreBundleCategoryDefinition vtable at [this], calls 00d67250 (ImageDataDefinition destructor, accepted speculative label), then r… |
| `00d5cb70` | StoreBundleCategoryDefinition::CtorBody | confirmed | Constructor body (ECX = object): installs the StoreBundleCategoryDefinition vtable at [this], sets the int at +4 to the constant DAT_01bf2e54 and the int at +8… |
| `00d5ccd0` | StoreBundleCategoryDefinition::VirtSlot2DeletingDtor | confirmed | Virtual slot 2 of the StoreBundleCategoryDefinition vtable: calls the destructor body 00d5cab0 on ECX and, when bit 0 of the flag argument is set, calls 00401c… |
| `00d5ccf0` | StoreBundleCategoryDefinition::CopyCtor | confirmed | Copy constructor body (ECX = new object, stack argument = source): installs the StoreBundleCategoryDefinition vtable, zeroes +4, calls 00d672e0 (ImageDataDefin… |
| `00d5cd90` | StoreBundleCategoryDefinition::CtorWithFieldsAndStrings | confirmed | Constructor body with arguments (ECX = object; stack arguments: two ints and two C strings): installs the StoreBundleCategoryDefinition vtable, stores the firs… |
| `00d5cb20` | StoreBundleCategoryDefinition::PredicateFieldsAndChildSlot0 | speculative | Virtual slot 0 of the StoreBundleCategoryDefinition vtable: returns 1 when the constant DAT_01bf2e54 is at most the int at this+4 and the int at this+8 is non-… |
| `00d5cf00` | — | unknown | Virtual slot 1 of the StoreBundleCategoryDefinition vtable: calls the vtable slot 0 of the object at this+0xc (image member) with a local StringFixed<256> outp… |

**GameCommerce::StoreBundleCategoryGroupDefinition** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5c2c0` | StoreBundleCategoryGroupDefinition::CtorBody | confirmed | Constructor body (ECX = object): installs the StoreBundleCategoryGroupDefinition vtable at [this], zeroes the word at +4, installs the SoeUtil::Set<...StoreBun… |
| `00d5c330` | StoreBundleCategoryGroupDefinition::DtorBody | confirmed | Destructor body: installs the group vtable, then the embedded category-id set's vtable at +8; when the set root at +0xc is non-null it tears the subtrees down … |
| `00d5c3c0` | StoreBundleCategoryGroupDefinition::VirtSlot2DeletingDtor | confirmed | Virtual slot 2 of the StoreBundleCategoryGroupDefinition vtable: calls the destructor body 00d5c330 on ECX and, when bit 0 of the flag argument is set, calls 0… |
| `00d5c540` | StoreBundleCategoryGroupDefinition::VirtSlot1FormatGroup | confirmed | Virtual slot 1 of the group vtable: walks the category-id set at this+0xc in ascending order (leftmost node first, in-order successors through the +0xc/+8 link… |
| `00d5c6e0` | StoreBundleCategoryGroupDefinition::CopyCtor | confirmed | Copy constructor body (ECX = new group, stack argument = source group): installs the group vtable, installs the category-id set vtable at +8, zeroes +0xc and +… |
| `00d5c2e0` | StoreBundleCategoryGroupDefinition::PredicateField4PosField… | speculative | Virtual slot 0 of the StoreBundleCategoryGroupDefinition vtable: returns 1 when the int at this+4 is greater than 0 and the int at this+0x10 is non-negative, e… |

**GameCommerce::StoreBundleCategoryGroupDefinition::Entry** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5c500` | StoreBundleCategoryGroupDefinition::Entry::VirtSlot1FormatC… | confirmed | Virtual slot 1 of the Entry vtable: calls 00402fd0 (SoeUtil::IStringFormat) on the output object at param_2 with the format 'categoryId=%d' and the int at this… |
| `00d5c090` | StoreBundleCategoryGroupDefinition::Entry::PredicateField4A… | speculative | Virtual slot 0 of the StoreBundleCategoryGroupDefinition::Entry vtable: returns 1 when the constant at DAT_01bf2da4 is less than or equal to the int at this+4,… |

**GameCommerce::StoreBundleCategoryMapEntryDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5ce10` | StoreBundleCategoryMapEntryDefinition::VirtSlot1FormatEntry | confirmed | Virtual slot 1 of the map entry vtable: calls 00402fd0 (SoeUtil::IStringFormat) on the output object at param_2 with the format 'parentCategoryId=%d, childCate… |
| `00d5cc50` | StoreBundleCategoryMapEntryDefinition::PredicateField4Field… | speculative | Virtual slot 0 of the StoreBundleCategoryMapEntryDefinition vtable: returns 1 when the int at this+4 is at least the constant DAT_01bf2e6c, the int at this+8 i… |

**GameCommerce::StoreDefinition<AppStoreBundleDefinition,512>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c37e20` | StoreDefinition::RemoveBundleEntry | confirmed | Repeatedly removes the bundle entry at field offset 0x3c through 00b289d0 until that field becomes null. |

**GameCommerce::StorePortalCategoryDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c350e0` | StorePortalCategoryDefinitionCtorSetId | likely | Runs the constructor helper for a portal-category definition, copies the input integer into field offset 0x54, and returns this. |

**SoeUtil::Hash** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c41350` | StoreBundleIdSetHashCtor | confirmed | Constructs the contained typed store-bundle ID hash map and installs the typed Hash vtable. |
| `00c413b0` | StoreBundleIdSetHash::ScalarDeletingDtor | speculative | Destroys the backing store-bundle ID hash map and frees the object when flag bit 0 is set. |

**SoeUtil::HashListMap** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3ec30` | StoreBundleCategoryMapDtor | confirmed | Installs typed category-map vtables, repeatedly removes stored category groups, frees tree sentinel links and backing storage, and restores the generic base vt… |
| `00c3ed10` | StoreBillboardPanelMapCtor | confirmed | Installs generic and typed billboard-panel map vtables, clears the map counters and bucket array, and initializes supporting state. |
| `00c3f760` | StoreBillboardPanelMapDtor | confirmed | Installs derived then base map vtables, drains stored panel entries, frees tree sentinel links and storage, and clears map fields. |
| `00c41410` | StoreDefinitionMapDtor | confirmed | Installs the typed store-ID map vtable and repeatedly removes each StoreDefinition entry through its erase routine. |
| `00c41480` | StoreDefinitionMapCtor | confirmed | Initializes the one-bucket store-definition map, installs its fixed-capacity typed vtable, and initializes embedded pool state. |
| `00c41ed0` | StoreDefinitionMapDtor | confirmed | Destroys every StoreDefinition entry, frees pool/tree storage, then restores the generic base map vtable. |
| `00c3fe20` | StoreDefinitionMapFindOrCreate | likely | Searches the list at manager+0xc094 for a matching key at entry index 0xa01b; allocates a new entry on miss and invokes a virtual update on a hit. |
| `00c3f840` | StoreBundleCategoryMap::ScalarDeletingDtor | speculative | Calls the category-map destructor and frees the map object when flag bit 0 is set. |
| `00c3f860` | StoreBillboardPanelMap::ScalarDeletingDtor | speculative | Calls the billboard-panel map destructor and frees the map object when flag bit 0 is set. |
| `00c41200` | StoreDefinitionMapClearEntries | speculative | Repeatedly removes the current head entry through the store-definition map erase routine until the map is empty. |
| `00c41440` | StoreDefinitionMap::ScalarDeletingDtor | speculative | Destroys every store definition and frees the map object when flag bit 0 is set. |
| `00c41fa0` | StoreDefinitionMap::ScalarDeletingDtor | speculative | Calls the typed map destructor and frees the map object when flag bit 0 is set. |
| `00c3fd50` | — | unknown | Unlinks an entry from the map's active list and free list, decrements count, calls its StoreDefinition destructor, and dispatches the map's virtual release met… |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleCategoryId,1>,GameCommerce::StoreBundleCategoryTree<GameCommerce::StoreBundleCategoryDefinition>::CategoryNode,128,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34ea0` | BundleCategoryNodeMap128Ctor | confirmed | Installs the category-node map vtable, clears map fields, and zeroes 512 bytes of bucket storage. |
| `00c35fb0` | BundleCategoryNodeMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x5c bytes with alignment 4. |
| `00c35ff0` | BundleCategoryNodeMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>,GameCommerce::StoreBundleGroupDefinition,16,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34bd0` | BundleGroupMap16Ctor | confirmed | Installs the bundle-group map vtable, clears map fields, and zeroes its 16 bucket heads. |
| `00c35f00` | BundleGroupMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x80 bytes with alignment 4. |
| `00c35f40` | BundleGroupMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,AppStoreBundleDefinition,2048,-1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c351e0` | AppStoreBundleMap2048Ctor | confirmed | Installs the bundle map vtable, clears map fields, and zeroes 8192 bytes of bucket storage. |
| `00c35e80` | AppStoreBundleMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x130 bytes with alignment 8. |
| `00c35ec0` | AppStoreBundleMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 8. |
| `00c38f00` | AppStoreBundleMapScalarDeletingDtor | confirmed | Clears all bundle nodes through 00b289d0 and frees the map object when the low flag bit is set. |
| `00c389b0` | AppStoreBundleMapClear | likely | Installs the base bundle-map vtable and repeatedly removes the head bundle node through 00b289d0 until empty. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,AppStoreBundleDefinition,2048,512>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c39080` | AppStoreBundleMapCtor | confirmed | Initializes an empty 2048-bucket bundle map, installs the fixed-capacity derived vtable, and initializes its 512-entry node pool. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreId,1>,GameCommerce::StoreDefinition<AppStoreBundleDefinition,512>,1,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c35e20` | StoreDefinitionMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x28074 bytes with alignment 4. |
| `00c35e60` | StoreDefinitionMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashListMap<int,AppStoreBillboardPanelDefinition,16,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34d30` | BillboardPanelMap16Ctor | confirmed | Installs the billboard-panel map vtable, clears map fields, and zeroes its 16 bucket heads. |
| `00c36410` | BillboardPanelMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x6c bytes with alignment 4. |
| `00c36450` | BillboardPanelMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashListMap<int,GameCommerce::StoreBillboardDefinition<AppStoreBillboardPanelDefinition>,64,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34ca0` | StoreBillboardMap64Ctor | confirmed | Installs the billboard map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c361d0` | BillboardDefinitionMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x1a0 bytes with alignment 4. |
| `00c36210` | BillboardDefinitionMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashListMap<int,GameCommerce::StoreBundleCategoryGroupDefinition,64,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34c00` | BundleCategoryGroupMap64Ctor | confirmed | Installs the category-group map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c36010` | CategoryGroupMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x24 bytes with alignment 4. |
| `00c36050` | CategoryGroupMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashListMap<int,GameCommerce::StorePortalCategoryDefinition,64,-1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34c60` | StorePortalCategoryMap64Ctor | confirmed | Installs the portal-category map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c36120` | PortalCategoryMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x5c bytes with alignment 4. |
| `00c36160` | PortalCategoryMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |
| `00c38b70` | PortalCategoryMapDtor | confirmed | Installs the portal-category map vtable and repeatedly removes its head node through 00c371a0. |
| `00c38f40` | PortalCategoryMapScalarDeletingDtor | confirmed | Clears all portal-category nodes through 00c371a0 and frees the map object when the low flag bit is set. |

**SoeUtil::HashListMap<int,GameCommerce::StoreShortcutDefinition,64,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34c30` | StoreShortcutMap64Ctor | confirmed | Installs the shortcut map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c360c0` | ShortcutMapAllocate | confirmed | Allocates from the current or global memory manager, falling back to 0x24 bytes with alignment 4. |
| `00c36100` | ShortcutMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**SoeUtil::HashMap** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3ff00` | StoreBundleIdSetHashMapDtorBase | confirmed | Stores the typed hash-map vtable and delegates cleanup of all contained bundle-id sets to the member destructor. |
| `00c406d0` | StoreBundleIdSetHashMapDtor | confirmed | Runs set-map cleanup, frees tree sentinel links and backing storage, and restores the generic hash-map vtable. |
| `00c41170` | StoreBundleIdHashMapCtor | confirmed | Installs generic then typed hash-map vtables, clears its 1024-bucket array and count, and initializes supporting state. |
| `00c41220` | StoreBundleIdHashMapCleanup | likely | Installs an SEH frame, calls the typed store-bundle ID map destructor, and restores the prior exception-list pointer. |
| `00c40270` | StoreBundleIdSetHashMap::ScalarDeletingDtor | speculative | Calls the typed hash-map destructor and frees the object when flag bit 0 is set. |
| `00c41150` | StoreBundleIdHashMap::ScalarDeletingDtor | speculative | Runs the typed hash-map destructor, then frees the object when flag bit 0 is set. |
| `00c3f880` | — | unknown | Walks all 1024 buckets, destroys each set value, decrements entry count, and clears sentinel and tree storage. |

**SoeUtil::HashMap<int,SoeUtil::Set<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,32,0>,1024,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c34b80` | BundleIdSetMap1024Ctor | confirmed | Installs the map vtable, clears its state fields, and zeroes 4096 bytes of bucket storage. |
| `00c35d60` | StoreBundleSetMapAllocate | confirmed | Requests an aligned allocation from the current or global memory manager, falling back to 0x2ac bytes at alignment 4. |
| `00c35da0` | StoreBundleSetMapFree | confirmed | Releases the supplied block through 00401c00 using alignment 4. |

**StoreBillboardPanelDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3ea10` | StoreBillboardPanelDefinition::ReadFields | confirmed | Reads name_id, image, display_order, shortcut fields, schedule_id, and oob_data from the data source into the panel definition. |

**StoreBundleCategoryDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3e8e0` | StoreBundleCategoryDefinition::ReadFields | confirmed | Reads category_id, name_id, description_id, display_order, image, shortcut fields, and oob_data from the data source into the definition and base fields. |

**StoreBundleCategoryTree** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3fa70` | StoreBundleCategoryTreeCtor | confirmed | Installs BaseDefinition then StoreBundleCategoryTree vtables, constructs its category hash map, zeroes map arrays and counters, and clears two flag bytes. |
| `00c3fb40` | StoreBundleCategoryTree::AppendDebugText | confirmed | Walks category nodes through +0x13 links and appends each parent id, count, and virtual category text to the output string. |
| `00c3fc80` | StoreBundleCategoryTreeDtor | confirmed | Restores the category-tree vtable, clears a state byte, releases auxiliary resources, destroys the category map, and restores the BaseDefinition vtable. |
| `00c3fb20` | StoreBundleCategoryTree::CanRefresh | likely | Returns true when the child pointer at +0xc is null; otherwise calls that child's virtual method and returns its boolean result. |
| `00c3fcf0` | StoreBundleCategoryTree::ScalarDeletingDtor | speculative | Calls the category-tree destructor and frees the object through the memory manager when flag bit 0 is set. |

**StoreDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3f9e0` | StoreDefinition::AssignFrom | confirmed | Copies scalar fields and a subobject from another store definition, removes existing bundle nodes, and rebuilds its bundle map from the source. |
| `00c3fa40` | StoreDefinition::RebuildBundleMap | confirmed | Walks linked bundle entries by +0x120 and invokes the bundle insertion routine for each entry's +0x128 data. |
| `00c3eb30` | StoreDefinition::DeletingDtor | likely | Calls the StoreDefinition destructor and frees the object when the deleting-destructor flag bit is set. |
| `00c3f930` | StoreDefinition::AppendBundleEntry | likely | Allocates a bundle entry through the container allocator, links it into the doubly linked sequence, records the tail, and increments entry count. |

**(no class)** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c400c0` | LoadStoreBillboardPanels | confirmed | Finds mrkt_store_billboard_panel_vw, iterates and populates billboard-panel definitions, validates rows, logs failures or successful count, and releases the so… |
| `00c40410` | LoadStoreBillboards | confirmed | Loads panel definitions first, then reads mrkt_store_billboard rows, creates billboard records, groups related definition entries, validates each result, and l… |
| `00c388b0` | FindStoreBundleDefinition | likely | Finds a store by id, selects one of its 2048 bundle-map buckets, scans collisions, and returns the matching bundle entry or null. |
| `00c38910` | FindStoreBundleDefinitionAlias | likely | Finds a store by id, selects one of its 2048 bundle-map buckets, scans collisions, and returns the matching bundle entry or null. |
| `00c3e890` | InsertBundleDefinitions | likely | Walks the linked entries at param_2+8 through each +0x120 link and calls the store bundle insertion routine for entry+0x128. |
| `00c40360` | UpdateStoreEntryRegistration | likely | Finds an entry by a global key, asks it to update, removes stale tree registration when key identity changes, and refreshes store indexes. |
| `00c74840` | SetPruneNotInTable1024 | likely | Builds a temporary SoeUtil::Set-style list via FUN_00c73020 and FUN_00cf19d0, walks the resulting linked tree (links at +4, +8 and parent at +0xc), and for eac… |
| `00c74bc0` | StoreFilterInsertMatches | likely | Builds a temporary id list: copies the dword key of each node of the list at param_2+8 (next link +0x10) through FUN_00990bc0, collects the ids via FUN_00c3045… |
| `00c75bd0` | StoreRebuildFilteredList | likely | Calls FUN_00c710e0 and, when it returns true, builds a temporary 64-bucket HashList with FUN_00c74a30, fills it through FUN_00c74bc0 (StoreFilterInsertMatches)… |
| `00d5c7b0` | StateBlockDtorCleanup | likely | Releases the string members of the state block at ECX and the block's first member: installs the string vtables and, for each of the four string members at +0x… |
| `00d5c970` | StateBlockCtorBody | likely | Constructor body of the commerce state block (ECX = block): constructs the HashListMap member at offset 0 through 00d5a490, zeroes the byte at +0x1ec and the w… |
| `00c387f0` | ValidateStoreShortcutTarget | speculative | Finds the selected store, checks its shortcut type against bundle, portal-category or category-group lookups, and logs then rejects unknown types. |
| `00c389e0` | ClearBundleMapTree | speculative | Recursively frees both child subtrees of the root using alignment 8, frees the root, then clears the tree root and count. |
| `00c38a20` | ClearStoreIdTree | speculative | Recursively frees both child subtrees of the root using alignment 4, frees the root, then clears the tree root and count. |
| `00c38a60` | ClearBundleGroupTree | speculative | Recursively frees both child subtrees of the root using alignment 4, frees the root, then clears the tree root and count. |
| `00c38aa0` | ClearShortcutTree | speculative | Recursively frees both child subtrees of the root using alignment 4, frees the root, then clears the root pointer and count. |
| `00d5c750` | SetPositiveField1F4 | speculative | Stores the stack argument at this+0x1f4 when it is greater than 0; otherwise leaves the field unchanged. ret 4 (guarded store). |
| `00d5c770` | SetNonNegativeField1F0 | speculative | Stores the stack argument at this+0x1f0 when it is non-negative (>= 0); otherwise leaves the field unchanged. ret 4 (guarded store). |
| `00d5c790` | SetByteField2AC | speculative | Stores the low byte of its stack argument at this+0x2ac unconditionally (mov al, [esp+4]; mov [ecx+0x2ac], al; ret 4). |
| `00d5c7a0` | SetByteField230 | speculative | Stores the low byte of its stack argument at this+0x230 unconditionally (mov al, [esp+4]; mov [ecx+0x230], al; ret 4). |
| `00d5c8e0` | AssignStringField250IfNonNull | speculative | Assigns the C string argument to the string member at this+0x250 through 00402ce0 (SoeUtil::IStringAssign, tail jump) when the pointer is non-null; a null poin… |
| `00d5c900` | AssignStringField26CIfNonNull | speculative | Assigns the C string argument to the string member at this+0x26c through 00402ce0 (SoeUtil::IStringAssign, tail jump) when the pointer is non-null; a null poin… |
| `00d5c920` | AssignStringField1F8IfNonNull | speculative | Assigns the C string argument to the string member at this+0x1f8 through 00402ce0 (SoeUtil::IStringAssign, tail jump) when the pointer is non-null; a null poin… |
| `00d5c940` | AssignStringField288IfNonEmpty | speculative | Assigns the C string argument to the string member at this+0x288 through 00402ce0 (SoeUtil::IStringAssign, tail jump) only when the pointer is non-null and the… |
| `00c36f80` | — | unknown | Walks a linked list at object offset 0x50, looks up each referenced id in the global 2048-bucket bundle map, and returns false on the first missing id. |
| `00c375f0` | — | unknown | Calls the billboard-definition destructor for a stack-adjusted temporary object during exception cleanup. |
| `00c38ae0` | — | unknown | When the byte flag at offset 0xec1 is clear, traverses the tree at this+4 with predicate 00c34ee0 and sets the flag. |
| `00c3eb90` | — | unknown | For each entry in the linked list at param_2+0x50, searches a 1024-bucket chain using key+0xc and removes the corresponding tree value using key at entry+0x9c. |
| `00c3fe70` | — | unknown | Removes an entry from key-based list/tree indexes, inserts a missing tree key if needed, then invokes another index cleanup routine. |
| `00c3ff10` | — | unknown | Finds mrkt_store_portal_category_vw, iterates its rows, allocates and populates category definitions, validates each row, logs failures or the successful count… |
| `00c402b0` | — | unknown | Clears per-entry state, frees a search tree and bundle-id set map, unregisters entries from indexes, runs maintenance sweeps, then marks a cleanup flag after a… |
| `00c77230` | — 🌐 | unknown | Looks up the item definition id (param_5) in the 1024-bucket table reached from this[0]+0x20c+0x64 (bucket id and 0x3ff, chain link +0xb0, id at +0xb4). When f… |
| `010870a0` | — | unknown | Thiscall with one stack arg (source record), ret 4. Sets this+0x10 to 0, stores ItemRecord::vftable at this+0x20 and zeroes +0x24 and +0x28; if the source is n… |

### `game.ingamepurchase`

**AppStoreBundleDefinition** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01078450` | AppStoreBundleDefinition::AppStoreBundleDefinition | confirmed | Fastcall default constructor under an EH frame: calls base constructor 0110aab0 on this, stores AppStoreBundleDefinition::vftable, and zeroes every field from … |
| `010784e0` | AppStoreBundleDefinition::CopyConstruct | confirmed | Thiscall copy constructor with one stack argument (source pointer) under an EH frame: calls base copy constructor 0110ac90 with the source, stores AppStoreBund… |
| `01078650` | AppStoreBundleDefinition::Destruct | confirmed | Fastcall destructor body under an EH frame: stores AppStoreBundleDefinition::vftable, then calls base destructor 0110aa90 on this. Frees nothing. |
| `010786a0` | AppStoreBundleDefinition::InitDefaultAlt | confirmed | Fastcall default constructor under an EH frame: calls base constructor 0110ab20 on this, stores AppStoreBundleDefinition::vftable, and zeroes the same fields a… |
| `01078750` | AppStoreBundleDefinition::DestructFreeIfBit0 | confirmed | Thiscall with one stack byte argument (ret 4): stores AppStoreBundleDefinition::vftable, calls base destructor 0110aa90, and if bit 0 of the argument is set ca… |
| `01078910` | AppStoreBundleDefinition::ParseOobData | confirmed | Thiscall with one char pointer argument (OOB data string) and a 2048-byte fixed string plus a stack reader state. Returns at once when the string is empty. Oth… |
| `00b25980` | AppStoreBundleDefinition::ScalarDeletingDtor | likely | Scalar deleting destructor: calls FUN_01078650 with this in ECX (that body writes AppStoreBundleDefinition::vftable and destroys the GameCommerce::StoreBundleD… |
| `00b26340` | AppStoreBundleDefinition::ScalarDeletingDtorVariant | likely | Second copy of the AppStoreBundleDefinition scalar deleting destructor: calls FUN_01078650 with this in ECX (destructor body writing AppStoreBundleDefinition::… |

**ClientInGamePurchaseManager** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b201f0` | ClientInGamePurchaseManager::VirtSlot18 | confirmed | Loads the object at [ECX+4], loads that object?s field +0x15c, pushes constant 1 and the stack argument, and makes a normal call to 00a08520 before returning w… |
| `00b20210` | ClientInGamePurchaseManager::VirtSlot19 | confirmed | Raw x86 loads ECX from [ECX+4], then loads ECX from [ECX+0x15c], and tail-jumps to 00a088c0. |
| `00b303e0` | ClientInGamePurchaseManager::OnSystemSupportDataInitialized | confirmed | Formats and logs the success flag; on success sets the support-data-ready byte and invokes MarketplaceCallbackHandler.OnSystemSupportDataInitialized. |
| `00b304b0` | ClientInGamePurchaseManager::LogSupportDataProgress | confirmed | Formats the processed-item count into a marketplace support-data progress message and sends it through the configured logger. |
| `00b37f30` | ClientInGamePurchaseManager::HandlePaymentSourcesResponse | confirmed | Logs response fields; on success stores payment-source data and validates default currency against local code map; on failure logs status/message; invokes OnPa… |
| `00b385f0` | ClientInGamePurchaseManager::HandleAddCardResponse | confirmed | Logs AddNewCard success/failure, refreshes payment sources on success, dispatches OnAddCreditCard with status/card id/message, and updates active pending reque… |
| `00b38860` | ClientInGamePurchaseManager::HandleUpdateCardResponse | confirmed | Logs update status, refreshes payment sources on success, dispatches OnUpdateCreditCard with status/source id/message and updates pending result if active. |
| `00b38ab0` | ClientInGamePurchaseManager::HandleDeleteCardResponse | confirmed | Logs deletion status, refreshes payment sources on success, dispatches OnDeleteCreditCard with status/source id/message and updates pending result if active. |
| `00b3df80` | ClientInGamePurchaseManager::ConfigurePaymentService | confirmed | Sets CAS and URAM configuration strings and paths, including platform, authentication certificates, session id, account, theme, and API-version fields on the s… |
| `00b3ff00` | ClientInGamePurchaseManager::HandleCardTypesResponse | confirmed | Virtual slot 14 of ClientInGamePurchaseManager: logs 'RESP (%d, %d, %d, %d) - CardTypes.' from the response fields +4, +8, +0x120 and +0x124; when +8 == 1 it r… |
| `00b41ac0` | ClientInGamePurchaseManager::ClientInGamePurchaseManager | confirmed | Installs CommerceProcessorHandler then ClientInGamePurchaseManager vtables, stores the owner, allocates and initializes Station-Cash store data sources through… |
| `00b41da0` | ClientInGamePurchaseManager::~ClientInGamePurchaseManager | confirmed | Tears down the manager-owned store object and member collections: releases purchase data, clears and destroys sale lists and currency maps, destroys session st… |
| `00b432f0` | ClientInGamePurchaseManager::ScalarDeletingDtor | confirmed | The 27-byte x86 wrapper saves this, calls manager destructor body 00b41da0, tests bit 0 of the flag byte and calls _free(this) when set; otherwise restores thi… |
| `00b43de0` | ClientInGamePurchaseManager::HandleWalletFundResponse | confirmed | Virtual slot 10 logs wallet-fund success or maps the failure result and message, requests wallet information, transfers the response fields into client-side st… |
| `00b44370` | ClientInGamePurchaseManager::HandleMembershipPreviewResponse | confirmed | Virtual slot 15 logs preview success/failure and track id, requests wallet information, marshals response's fields into local typed callback data with per-fiel… |
| `00b44af0` | ClientInGamePurchaseManager::HandleMembershipPurchaseRespon… | confirmed | Virtual slot 16 logs membership purchase success or the result-code failure, requests wallet information, places the success value, order id, and response mess… |
| `00b44ce0` | ClientInGamePurchaseManager::HandleFinalizeThirdPartyOrderR… | confirmed | Virtual slot 20 logs the response result, writes a success log with the track id or maps the failure result code and message, then requests refreshed wallet in… |
| `00b28300` | ClientInGamePurchaseManager::HandleProductUnavailableRemove | likely | Sets ECX to this+0x10 and calls FUN_00c3b790 with DAT_01b5aac8 and param_2 on the stack. If the returned byte is zero, calls FUN_00b201c0 to log the exact Prod… |
| `00b20220` | — | unknown | Eight-byte tail-call thunk: 8b 49 04 (mov ecx,[ecx+4]) e9 98 bc e1 ff (jmp 0x0093bec0); it re-dispatches the virtual call to the object stored at +4 of the man… |
| `00b23de0` | — | unknown | Virtual handler (ClientInGamePurchaseManager vtable slot 21) for the partner-affiliation response: logs RESP (%d, %d, %d, %d) - IsAffiliatedWithPartnerResponse… |
| `00b38150` | — | unknown | Logs preview status and fills Lua args with success, order id, six strings, two scalars and two flags, then calls OnWalletFundPreview. |

**GameCommerce::BaseDefinition** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099d100` | BaseDefinition::DtorBody | confirmed | Restores the BaseDefinition vtable, conditionally frees the object when the deleting flag is set, and returns the object pointer. |
| `00d58300` | GameCommerce::BaseDefinition::Dtor | confirmed | Bytes c7 01 a8 d2 80 01 c3: stores GameCommerce::BaseDefinition::vftable into *this and returns; no base call and no storage release. |
| `00d60840` | GameCommerce::BaseDefinition::~BaseDefinition | confirmed | Fastcall dtor body (7 bytes): stores the GameCommerce::BaseDefinition vftable into *this. Called from five EH unwind funclets, from the portal category destruc… |
| `00997140` | BaseDefinitionDtorBody | likely | Fastcall (ECX=this), 7 bytes: stores the GameCommerce::BaseDefinition vtable (0180d2a8) to [this] and returns (c7 01 imm32; c3). Releases nothing. |
| `00b26120` | GameCommerce::BaseDefinition::ScalarDeletingDtor | likely | Scalar deleting destructor: calls FUN_00d58300 with this in ECX (the 7-byte destructor body that stores GameCommerce::BaseDefinition::vftable) and frees this v… |
| `00d58520` | GameCommerce::BaseDefinition::ScalarDeletingDtorAlign0 | likely | Stores GameCommerce::BaseDefinition::vftable into *this, then calls FreeMemory (FUN_00401c00) with alignment 0 when bit 0 of the flags argument is set; returns… |
| `00d60910` | GameCommerce::BaseDefinition::ScalarDeletingDtor | speculative | Thiscall virtual (vtable slot [2] of StoreShortcutDefinition; the body also appears in BaseDefinition's vtable). Stores the BaseDefinition vftable into *this; … |

**GameCommerce::BaseInGamePurchaseOrder** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d772b0` | GameCommerce::BaseInGamePurchaseOrder::ScalarDeletingDtor | confirmed | Calls the order destructor body 00d76f70 and then calls _free(this) when bit 0 of the flag argument is set. It returns this. |
| `00d77360` | GameCommerce::BaseInGamePurchaseOrder::BaseInGamePurchaseOr… | confirmed | Constructor body: stores the BaseInGamePurchaseOrder vtable at this and initializes its embedded StringFixed<32>, StringFixed<8>, StringFixed<16> and IString m… |
| `00d77480` | GameCommerce::BaseInGamePurchaseOrder::FormatOrder | confirmed | Virtual slot 2 of the order vtable (shared with ClientInGamePurchaseOrder). It walks the details through virtual slots 8 (+0x20) and 9 (+0x24), formats each wi… |
| `00d76e80` | GameCommerce::BaseInGamePurchaseOrder::CheckAllDetails | likely | Virtual slot 1 predicate for the order: returns false unless the dwords at this+0xc, this+0x40 and this+0x78 are all positive. It then reads the count through … |
| `00d76f70` | — | unknown | Stores the BaseInGamePurchaseOrder vtable at this and releases its string members in turn: the IString at +0xcc, IStringFixed<char,8> at +0xb0 and +0x70, IStri… |
| `00d770f0` | — | unknown | Virtual slot 0 of the order vtable (shared with ClientInGamePurchaseOrder). It calls virtual slot 5 (+0x14) first, then releases and resets the string members … |

**GameCommerce::BaseInGamePurchaseOrderDetail** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5e230` | BaseInGamePurchaseOrderDetail::CopyCtor | confirmed | Copy constructor body (ECX = new detail, stack argument = source): installs the BaseInGamePurchaseOrderDetail vtable, copies the ints at +4, +8, +0xc and +0x10… |
| `00d5e2d0` | BaseInGamePurchaseOrderDetail::ScalarDeletingDtor | confirmed | Virtual slot 0 of the BaseInGamePurchaseOrderDetail vtable (scalar deleting shape): calls the destructor body 00d76ef0 on ECX, calls _free (00d82463) on this w… |
| `00d77210` | GameCommerce::BaseInGamePurchaseOrderDetail::BaseInGamePurc… | confirmed | Constructor: stores the BaseInGamePurchaseOrderDetail vtable and three dword arguments at this+4, +8 and +0xc, initializes an IStringFixed<char,16>-style membe… |
| `00d77330` | GameCommerce::BaseInGamePurchaseOrderDetail::FormatDetail | confirmed | Virtual slot 2 of the detail vtable. It formats '(storeId=%d, storeBundleId=%d, quantity=%d, rentalPeriod=%d, tint=%s)' into the string argument through 00402f… |
| `00d5d250` | BaseInGamePurchaseOrderDetail::ScalarDeletingDtor | likely | Scalar deleting destructor (ECX = object): saves ESI, calls the destructor body 00d76ef0 (which stores the BaseInGamePurchaseOrderDetail vtable, shard S0451) o… |
| `00d76e50` | GameCommerce::BaseInGamePurchaseOrderDetail::FieldsSetPredi… | speculative | Virtual slot 1 predicate: returns 1 when the dword at this+4 differs from the global DAT_01bf31f4, the dword at this+8 differs from the global DAT_01bf31e8, an… |
| `00d76ef0` | — | unknown | Stores the BaseInGamePurchaseOrderDetail vtable at this, releases the IString member at +0x38 and the IStringFixed<char,16> member at +0x14 (each drops a share… |

**GameCommerce::CasApi** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d75480` | GameCommerce::CasApi::OnCasAuthTicketResponse | confirmed | Virtual method in CasApi vtable slot 12. It finds the tracker in bucket (argument2 & 0xf) of the list at this+0x190 whose key at node+0x830 equals argument2, a… |
| `00d75b60` | GameCommerce::CasApi::OnSoeAuthTicketResponse | confirmed | Virtual method in CasApi vtable slot 15. It finds the tracker in bucket (argument2 & 0xf) of the list at this+0x190 by key, logging '(SoeAuthTicketResponse) - … |
| `00d76940` | GameCommerce::CasApi::CasApi | confirmed | Constructor: stores the Uram::UploaderHandler vtable, then the GameCommerce::CasApi vtable at offsets 0 and 4, clears the byte at +0x1c, initializes embedded I… |
| `00d76bf0` | GameCommerce::CasApi::AdjustorThunkTo00d76e20 | confirmed | Adjustor thunk: subtracts 4 from ECX (the secondary CasApi vtable sits at object+4) and jumps to the scalar deleting destructor 00d76e20. It does not contain t… |
| `00d76c00` | GameCommerce::CasApi::DtorBody | confirmed | Installs the CasApi vtables at offsets 0 and 4, calls 00d75a50 to cancel and drain all pending request trackers, then releases the IString members of the objec… |
| `00d76e20` | GameCommerce::CasApi::ScalarDeletingDtor | confirmed | Calls the CasApi destructor body 00d76c00 and then calls _free(this) when bit 0 of the flag argument is set. It returns this. |
| `00d749a0` | GameCommerce::CasApi::SendKeepAlive | likely | Builds a local three-string request from the session URL field at this+0x22f4, validates it with CasKeepAliveRequest::Validate (00f7b150), and sends it through… |
| `00d75140` | GameCommerce::CasApi::RequestAuthenticatedUrl | likely | Takes the next tracking id from this+0x18c, repeating the increment while it equals the sentinel DAT_01bf3180, allocates a tracking node through 00d74da0 keyed… |
| `00d751f0` | GameCommerce::CasApi::RequestSoeAuthTicket | likely | Builds a 2048-byte query string by appending 'partnercode' (argument5), 'gamecode' (argument6) and 'sessionid' (this+0x28) through 00d74f20. It then takes the … |
| `00d75940` | GameCommerce::CasApi::PeriodicUpdate | likely | Pumps the uploader through 00f60dc0. Every 299 ms (static tick at DAT_01bf3168) it runs the request-tracker expiry sweep 00d75800. If the initialised flag at t… |
| `00d74730` | — | unknown | Runs only while the byte at this+0x1c is clear. It assigns its argument to a local string, then if the int at this+0x2c is below 1 it logs 'Invalid CAS Authent… |
| `00d74b30` | — | unknown | Builds a local three-string request from this+0x2304 (the getTicket URL field) and the user-agent at this+0x28, validates it with CasGetSessionTicketRequest::V… |

**GameCommerce::CasHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d56d00` | GameCommerce::CasHandler::Dtor | confirmed | Bytes c7 01 6c 0a 84 01 c3: stores GameCommerce::CasHandler::vftable into *this and returns with no base call. |
| `00d56d10` | GameCommerce::CasHandler::ScalarDeletingDtor | confirmed | Stores GameCommerce::CasHandler::vftable into *this, then calls _free(this) when bit 0 of the flags argument is set; returns this and pops one stack argument. |

**GameCommerce::ClientInGamePurchaseOrder** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5e2f0` | ClientInGamePurchaseOrder::CtorBody | confirmed | Constructor body (ECX = order): calls 00d77360 (base constructor, unlabelled), installs the ClientInGamePurchaseOrder vtable at [this], installs the HashListMa… |
| `00d5e3b0` | ClientInGamePurchaseOrder::VirtSlot5DrainDetails | confirmed | Virtual slot 5 of the ClientInGamePurchaseOrder vtable: while the list head at this+0xe4 is non-null, removes it through 00d5d3e0 (RemoveAndDestroyNode of the … |
| `00d5e3e0` | ClientInGamePurchaseOrder::DtorBody | confirmed | Destructor body: stores the ClientInGamePurchaseOrder vtable at [this], destroys the embedded detail map through 00d5e050 (HashListMap ,4,1 body), then calls 0… |
| `00d5e4e0` | ClientInGamePurchaseOrder::ScalarDeletingDtor | confirmed | Virtual slot 7 of the ClientInGamePurchaseOrder vtable (scalar deleting shape): calls the destructor body 00d5e3e0 on ECX, calls _free (00d82463) on this when … |
| `00d5d180` | ClientInGamePurchaseOrder::VirtSlot4LookupValue | likely | Virtual slot 4 of the ClientInGamePurchaseOrder vtable: folds the key argument (param_3 XOR the sign mask of param_3 combined with param_2, so a negative key b… |
| `00d5e450` | ClientInGamePurchaseOrder::VirtSlot3AddOrInsertKeyed | likely | Virtual slot 3 of the ClientInGamePurchaseOrder vtable: folds the 64-bit key (param_2 OR the sign of param_3, XOR param_3) into a 32-bit key, searches the four… |
| `00d5e5e0` | ClientInGamePurchaseOrder::AssignFrom | likely | Copy-assigns an in-game purchase order from the source at the stack argument (ECX = this): calls 00d772d0 on the source first (base assign, unlabelled), drains… |
| `00d5e380` | ClientInGamePurchaseOrder::VirtSlot6GetField0xEC | speculative | Virtual slot 6 of the ClientInGamePurchaseOrder vtable: returns the dword at this+0xec (mov eax, [ecx+0xec]; ret). |
| `00d5e390` | ClientInGamePurchaseOrder::VirtSlot8GetField0xE4 | speculative | Virtual slot 8 of the ClientInGamePurchaseOrder vtable: returns the dword at this+0xe4 (mov eax, [ecx+0xe4]; ret). |
| `00d5e3a0` | ClientInGamePurchaseOrder::VirtSlot9GetArgField0x48 | speculative | Virtual slot 9 of the ClientInGamePurchaseOrder vtable: takes its object argument from the stack (not ECX), returns the dword at arg+0x48 and returns with ret … |

**GameCommerce::CommerceProcessor** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d56d90` | GameCommerce::CommerceProcessor::Ctor | confirmed | Bytes 8b c1 33 c9 c7 00 cc 0a 84 01 89 48 04 88 48 08 88 48 09 c3: stores GameCommerce::CommerceProcessor::vftable into *this, zeroes the dword at +4 and the b… |
| `00d570a0` | GameCommerce::CommerceProcessor::Dtor | confirmed | Stores GameCommerce::CommerceProcessor::vftable into *this and returns; no base call. |
| `00d574f0` | GameCommerce::CommerceProcessor::ScalarDeletingDtor | confirmed | Stores GameCommerce::CommerceProcessor::vftable into *this, then calls _free(this) when bit 0 of the flags argument is set; returns this. |

**GameCommerce::CommerceProcessorHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1ffb0` | CommerceProcessorHandlerVftableCleanup | confirmed | Writes GameCommerce::CommerceProcessorHandler::vftable to the object start and returns. |
| `00b1ffc0` | GameCommerce::CommerceProcessorHandler::ScalarDeletingDtor | likely | Reinstalls the CommerceProcessorHandler vtable; if param_2 bit 0 is set, frees param_1 through 00d82463, then returns param_1. |

**GameCommerce::IMarketingDataElementInstance** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d60a00` | GameCommerce::IMarketingDataElementInstance::~IMarketingDat… | confirmed | Fastcall dtor body (7 bytes): stores the GameCommerce::IMarketingDataElementInstance vftable into *this. Called from four EH unwind funclets. |
| `00d60a10` | GameCommerce::IMarketingDataElementInstance::ScalarDeleting… | confirmed | Thiscall virtual at vtable slot [5] of IMarketingDataElementInstance (read from the binary). Stores the interface vftable into *this; if arg bit 0 is set (code… |

**GameCommerce::ImageDataDefinition** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d67250` | ImageDataDefinition::Dtor | speculative | Installs the image-definition vtable, releases its two shared string fields, then restores the BaseDefinition vtable. |
| `00d672d0` | ImageDataDefinition::HasImage | speculative | Returns whether the 32-bit field at object offset 0x0c is greater than zero. |
| `00d672e0` | ImageDataDefinition::Ctor | speculative | Installs the image-definition vtable, initializes two no-share string members to the shared empty pointer, and clears their lengths. |
| `00d67310` | ImageDataDefinition::ScalarDeletingDtor | speculative | Calls the image-definition destructor and releases the object with alignment zero when the deleting flag is set. |
| `00d67360` | ImageDataDefinition::Format | speculative | Formats the image and tint string fields using image=%s, tint=%s and returns the formatter's result pointer. |
| `00d67390` | ImageDataDefinition::CtorFromInput | speculative | Initializes image definition and its two no-share strings, then copies the two input strings at offsets +4 and +0x14. |

**GameCommerce::InternalCommerceProcessor** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d57070` | AdjustorThunk_sub4_00d57070 | confirmed | Adjustor thunk: bytes 83 e9 04 subtract 4 from ecx (this) and e9 08 00 00 00 jump to 00d57080, the InternalCommerceProcessor scalar deleting destructor. |
| `00d57080` | GameCommerce::InternalCommerceProcessor::ScalarDeletingDtor | confirmed | Calls the InternalCommerceProcessor body destructor 00d56eb0 on this, then calls _free(this) when bit 0 of the flags argument is set; returns this. |
| `00d57ca0` | InitInternalCommerceProcessor | confirmed | Returns 1 after calling vtable slot 1 of the child at this+4 when the byte at this+8 is set. Otherwise it stores its argument at this+4, allocates 0x14 bytes i… |
| `00d56f50` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 20 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x20 and tail-jumps to it with the … |
| `00d56f60` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 24 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x24 and tail-jumps to it (forwarde… |
| `00d56f70` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 28 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x28 and tail-jumps to it (forwarde… |
| `00d56f80` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 2c ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x2c and tail-jumps to it (forwarde… |
| `00d56f90` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 30 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x30 and tail-jumps to it (forwarde… |
| `00d56fa0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 34 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x34 and tail-jumps to it (forwarde… |
| `00d56fb0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 38 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x38 and tail-jumps to it (forwarde… |
| `00d56fc0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 3c ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x3c and tail-jumps to it (forwarde… |
| `00d56fd0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 40 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x40 and tail-jumps to it (forwarde… |
| `00d56fe0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 42 44 ff e0: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x44 into eax and tail-jumps throug… |
| `00d56ff0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 48 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x48 and tail-jumps to it (forwarde… |
| `00d57000` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 4c ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x4c and tail-jumps to it (forwarde… |
| `00d57010` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 50 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x50 and tail-jumps to it (forwarde… |
| `00d57020` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 08 8b 48 04 8b 11 8b 52 54 ff e2: loads this+8, its +4 subobject and that object's vtable, reads slot at offset 0x54 and tail-jumps to it (forwarde… |
| `00d57490` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Bytes 8b 41 04 8b 48 04 ... 8b 50 58 ff d2 c2 0c 00: loads this+4 and its +4 subobject, pushes the three stack arguments in reverse order to the call frame, ca… |
| `00d574c0` | GameCommerce::InternalCommerceProcessor::ForwardChildSlot0x… | speculative | Same shape as ForwardChildSlot0x58 (bytes ... 8b 50 5c ff d2 c2 0c 00): forwards three stack arguments through this+4's +4 subobject to the vtable slot at offs… |
| `00d56eb0` | — | unknown | Sets InternalCommerceProcessor::vftable at *this and at this+4, calls vtable slot 19 (+0x4c) with flag 1 on the owned objects at +0xc and +0x10 when non-null, … |

**GameCommerce::MarketingBundleDefinition::Entry** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b25650` | MarketingBundleDefinition_Entry::ScalarDeletingDtor | likely | Scalar deleting destructor: installs an SEH try level, calls FUN_00d5a8d0 with this in ECX (that body references GameCommerce::MarketingBundleDefinition::Entry… |
| `00b20a90` | — | unknown | SEH-framed constructor wrapper: sets an exception frame and calls FUN_00d5a8d0 with the exception-cookie chain as its only explicit argument; returns nothing. … |
| `00b20af0` | — | unknown | SEH-framed constructor wrapper that returns its first argument: calls FUN_00d5a990 (whose own vtables are GameCommerce::MarketingBundleDefinition::Entry plus S… |

**GameCommerce::MarketingDataElementInstance<SoeUtil::StringFixed<32>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d65160` | MarketingDataElementInstanceString32::Ctor | speculative | Installs the string element-instance vtable, initializes its inline string array and zero counts, then requests capacity eight. |
| `00d65200` | MarketingDataElementInstanceString32::Dtor | speculative | Runs inline string-array cleanup, then restores the common marketing-element-instance base vtable. |
| `00d63a80` | — | unknown | Releases the string-array storage at this[3], then calls the object's virtual deallocator with size 8. |

**GameCommerce::MarketingDataElementInstance<__int64>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d64dc0` | MarketingDataElementInstanceInt64::Ctor | speculative | Initializes the int64 element-instance vtable and embedded inline int64 array, clears count fields, then invokes the allocator for eight elements. |
| `00d64f10` | MarketingDataElementInstanceInt64::Dtor | speculative | Releases embedded int64 array storage, resets its array vtable/state, then restores the base marketing-element vtable. |
| `00d63a40` | — | unknown | Clears the count field and calls the object's virtual deallocator with size 8. |

**GameCommerce::MarketingDataElementInstance<double>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d64f90` | MarketingDataElementInstanceDouble::Ctor | speculative | Initializes the double element-instance vtable and embedded eight-element double array, clears its counts, and calls the capacity helper. |
| `00d650e0` | MarketingDataElementInstanceDouble::Dtor | speculative | Releases embedded double array storage, resets array vtable/state, then restores the base marketing-element vtable. |
| `00d63a60` | — | unknown | Clears the count field and calls the object's virtual deallocator with size 8. |

**GameCommerce::MarketingDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d639e0` | MarketingDataSource::ValidateRecords | speculative | Returns false when the count at +0x18 is below one; otherwise walks the list at +0x10 and returns false on the first failed row check. |
| `00d66330` | MarketingDataSource::BuildRecordSet | speculative | Creates a record-set instance, walks field descriptors, constructs typed element objects for types 1, 2, and 3, appends them to its array, and returns zero. |
| `00d665b0` | MarketingDataSource::Ctor | speculative | Installs the marketing source vtable, initializes the record-set hash/list map and count fields, zeroes 0x80 bytes of bucket data, and resets two helper values… |
| `00d66610` | MarketingDataSource::Dtor | speculative | Restores the source vtable, invokes virtual cleanup for linked records, recursively frees three record trees, and releases all hash-list records. |
| `00d66900` | MarketingDataSource::ScalarDeletingDtor | speculative | Runs MarketingDataSource cleanup and frees the object when the deleting flag is set. |
| `00d63a10` | — | unknown | Walks the list beginning at this[4], saves each next pointer from node+0x98, then calls virtual slot +0x10 on that node. |

**GameCommerce::MarketingDataSourceFlatFileLoader** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d66980` | MarketingDataSourceFlatFileLoader::SetLoadResult | speculative | Passes the stored source pointer and state byte to helper 00d776e0, then clears the completion byte at offset 0x233. |
| `00d66ac0` | MarketingDataSourceFlatFileLoader::Ctor | speculative | Initializes loader and string members, stores input flags, assigns the path, and appends a slash when the path lacks a trailing slash or backslash. |
| `00d66ba0` | MarketingDataSourceFlatFileLoader::GetName | speculative | Returns the static string FlatFileLoader. |
| `00d66bb0` | MarketingDataSourceFlatFileLoader::GetCompleteFlag | speculative | Returns the byte stored at loader offset 0x233. |
| `00d66bc0` | MarketingDataSourceFlatFileLoader::Dtor | speculative | Restores loader vtable, releases shared path and directory strings, resets loader state, releases the string base, and calls helper 00d77700. |
| `00d66cd0` | MarketingDataSourceFlatFileLoader::CanLoad | speculative | When completion byte is zero, reallocates the input buffer to 0x100001 if current length is smaller, then returns one; otherwise returns zero. |
| `00d66d70` | MarketingDataSourceFlatFileLoader::ScalarDeletingDtor | speculative | Calls the flat-file loader destructor body and frees the object when the deleting flag is set. |
| `00d671d0` | MarketingDataSourceFlatFileLoader::LoadAllRecordSets | speculative | Checks loader input and helper status, iterates the marketing source record list, calls the per-record file loader for each node, and returns aggregate success. |
| `00d67220` | MarketingDataSourceFlatFileLoader::Load | speculative | Sets completion flag, stores the source pointer, invokes LoadAllRecordSets, and stores its boolean result at offset 0x232. |
| `00d66a50` | — | unknown | Releases a shared path string when referenced, resets its string and loader fields, and clears the active source pointer and completion flag. |

**GameCommerce::MarketingDataSourceLoader** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d776c0` | GameCommerce::MarketingDataSourceLoader::MarketingDataSourc… | confirmed | Constructor body: stores the MarketingDataSourceLoader vtable at this and clears the dword at this+4. |
| `00d77700` | GameCommerce::MarketingDataSourceLoader::DtorBody | confirmed | Stores the MarketingDataSourceLoader vtable at this and clears the dword at this+4. It does not free the object. |
| `00d77710` | GameCommerce::MarketingDataSourceLoader::ScalarDeletingDtor | confirmed | Virtual slot 0 of the loader vtable: stores the loader vtable, clears the dword at this+4, and calls _free(this) when bit 0 of the flag argument is set. It ret… |

**GameCommerce::MarketingDataSourceLoaderHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5f100` | GameCommerce::MarketingDataSourceLoaderHandler::~MarketingD… | confirmed | Stores GameCommerce::MarketingDataSourceLoaderHandler::vftable into *this (7 bytes, nothing else). Called only from three EH unwind funclets. |
| `00d5f110` | GameCommerce::MarketingDataSourceLoaderHandler::ScalarDelet… | confirmed | Virtual at vtable slot [1] of the handler class (read from the binary). Stores the handler vftable into *this; if arg bit 0 is set (code: if ((param_2 & 1) != … |

**GameCommerce::MarketingDataUpdater** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d5f420` | GameCommerce::MarketingDataUpdater::Startup | confirmed | Thiscall with arg1 = schema file path and arg2 = loader object pointer. Logs 'Starting up' with the loader type (arg2 vtable slot [1] called with this+0x68) an… |
| `00d5f550` | GameCommerce::MarketingDataUpdater::OnLoadComplete | confirmed | Virtual at vtable slot [0] of MarketingDataUpdater (ordinary virtual, body is not a destructor). Sets state +0x278 to 2 and flag +0x288 to arg3; stores the ela… |
| `00d5f890` | GameCommerce::MarketingDataUpdater::Shutdown | confirmed | Fastcall (ECX=this). Logs 'Shutting down'. If the flag at +0x291 is set: unlinks every node of the list at this+0x08 (00d5f6d0), releases the loader at +0x74 (… |
| `00d5f950` | GameCommerce::MarketingDataUpdater::BeginLoad | confirmed | Fastcall (ECX=this), returns 1 on success or 0. Logs 'Loading new marketing data.'; if the saved request time at +0x268/+0x26c differs from the sentinel DAT_01… |
| `00d5fb10` | GameCommerce::MarketingDataUpdater::CheckReloadFile | confirmed | Fastcall (ECX=this). Calls 00704840 on the reload file path at this+0x68 (file-metadata wrapper, flag 0); if it reports a file, logs 'Found reload file (%s) in… |
| `00d5fbf0` | GameCommerce::MarketingDataUpdater::MarketingDataUpdater | confirmed | Fastcall ctor (ECX=this, no arguments). Stores the MarketingDataUpdater vftable at +0x00; zeroes the hash-list header at +0x04..+0x0c and the 16 bucket heads a… |
| `00d5fd30` | GameCommerce::MarketingDataUpdater::~MarketingDataUpdater | confirmed | Thiscall dtor body. Stores the MarketingDataUpdater vftable, calls Shutdown (00d5f890); releases the owned object at +0x27c (vtable slot [0] with 1) and clears… |
| `00d5fe40` | GameCommerce::MarketingDataUpdater::ScalarDeletingDtor | confirmed | Virtual at vtable slot [1] of the updater class (read from the binary). Calls the destructor body 00d5fd30; if arg bit 0 is set (code: if ((param_2 & 1) != 0))… |
| `00d5ff00` | GameCommerce::MarketingDataUpdater::MarketingDataUpdater2Ar… | confirmed | Thiscall ctor with two C-string arguments. Stores the updater vftable at +0x00; zeroes the hash-list header and 16 bucket heads as in 00d5fbf0; assigns arg1 in… |
| `00d60130` | GameCommerce::MarketingDataUpdater::ScheduleNextUpdate | confirmed | Fastcall (ECX=this). Gets the cached wall clock (00702670) and converts it to calendar fields (007026b0); sets the field taken from the refresh hour at +0x254 … |
| `00d60340` | GameCommerce::MarketingDataUpdater::Tick | confirmed | Fastcall (ECX=this), no return value. Switches on the state at +0x278. State 0 (idle): when the forced flag at +0x290 is clear, returns unless the automatic fl… |

**GameCommerce::StoreBillboardPanelDefinition** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0110aec0` | StoreBillboardPanelDefinition::CtorValue | confirmed | Constructor with one stack argument (ret 4): installs the vtable 01892e3c, stores the argument at +4 and zero at +8, constructs member subobjects at +0xc (00d6… |
| `0110afb0` | StoreBillboardPanelDefinition::DtorBody | confirmed | Destructor body (no free): stores the StoreBillboardPanelDefinition vtable, destroys the member at +0x30 (00d60840), destroys the member at +0xc (00d67250), th… |
| `0110b060` | StoreBillboardPanelDefinition::ScalarDeletingDestructor | confirmed | Virtual slot [2] of StoreBillboardPanelDefinition with the deleting-destructor shape: calls body 0110afb0 on this, then calls 00401c00 (free) on the object whe… |
| `0110b080` | StoreBillboardPanelDefinition::CtorCopy | confirmed | Copy constructor (ECX=this, stack argument = source object): installs the vtable, constructs members at +0xc (00d672e0) and +0x30 (00d608b0), copies the dwords… |
| `0110b120` | StoreBillboardPanelDefinition::AppendDumpText | confirmed | Virtual slot [1] of StoreBillboardPanelDefinition: converts the dword at +0x44 through slot [1] of the member at +0x30 into a StringFixed<512> temporary, conve… |
| `0110af40` | — | unknown | Default constructor (ECX=this): installs the vtable 01892e3c, zeroes +4 and +8, constructs member subobjects at +0xc (00d672e0) and +0x30 (00d608b0, with the f… |

**GameCommerce::StoreBundleCategoryDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b29fb0` | StoreBundleCategoryDefinition::SetField54 | speculative | Initializes the object through a helper, then copies one supplied dword into offset 0x54 and returns this. |

**GameCommerce::StoreBundleCategoryGroupDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b25ce0` | StoreBundleCategoryGroupDefinition::ScalarDeletingDtor | likely | Scalar deleting destructor: calls FUN_00d5c330 with this in ECX (that body writes GameCommerce::StoreBundleCategoryGroupDefinition::vftable and releases its So… |

**GameCommerce::StoreBundleDefinition** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0110aa90` | StoreBundleDefinition::DtorBody | confirmed | Destructor body (no free): stores the StoreBundleDefinition vtable 01892d34 into the object at ECX, then tail-jumps to 00d5bef0 for the base-class cleanup. Fiv… |
| `0110aab0` | StoreBundleDefinition::CtorDefault | confirmed | Default constructor (ECX=this): calls base constructor 00d5bc20, installs the StoreBundleDefinition vtable, copies the global DAT_01cb9ecc into the dword at +0… |
| `0110ab20` | StoreBundleDefinition::CtorValue | confirmed | Constructor with one stack argument (epilogue ret 8, only the first argument is read): calls base constructor 00d5bc20, installs the vtable, stores the argumen… |
| `0110ac60` | StoreBundleDefinition::ScalarDeletingDestructor | confirmed | Virtual slot [2] of StoreBundleDefinition with the deleting-destructor shape: stores the vtable, calls body 00d5bef0, then calls 00401c00 (free) on the object … |
| `0110ac90` | StoreBundleDefinition::CtorCopy | confirmed | Copy constructor (ECX=this, one stack argument = source object): sets up a SEH frame, calls base constructor 00d5bc20, installs the vtable, zeroes +0xc8, calls… |
| `0110ad30` | StoreBundleDefinition::AppendDumpText | confirmed | Virtual slot [1] (vtables 01892d34 and 01882f28): appends one formatted diagnostic text to the IString argument through 00402ff0. The values come from this+0xc… |
| `0110ab90` | StoreBundleDefinition::CopyFieldsFrom | likely | Copies state from the source object given as the stack argument into this (ECX): calls base copy 00d5b0a0 with the source, then copies the dwords at +0xc8, +0x… |
| `0110acf0` | StoreBundleDefinition::MatchFieldsD0ToE0NonNegative | speculative | Virtual slot [0] of StoreBundleDefinition, a predicate: calls 00d5a7e0 on this and ignores its result, then returns 1 when the dwords at +0xd0, +0xd4, +0xd8, +… |

**GameCommerce::StoreBundleGroupDefinition** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d58770` | GameCommerce::StoreBundleGroupDefinition::FormatGroupRecord | confirmed | Builds a comma-separated list of entry texts through the entry list at this+0x44 (each via vtable slot 1 of the entry, formatted with '(%s), ' and trimmed of i… |
| `00d58be0` | GameCommerce::StoreBundleGroupDefinition::CtorSentinel | confirmed | Installs the StoreBundleGroupDefinition vtable, stores the global sentinel DAT_01bf2ccc at +4, clears the dwords at +8, +0xc and +0x10, runs FUN_00d672e0, clea… |
| `00d58cf0` | GameCommerce::StoreBundleGroupDefinition::Dtor | confirmed | Sets the StoreBundleGroupDefinition vtable, installs the HashListMap vtable at +0x3c, removes every list node at +0x44 with FUN_00b290e0 until empty, runs the … |
| `00d58d70` | GameCommerce::StoreBundleGroupDefinition::ScalarDeletingDto… | confirmed | Bytes 56 8b f1 e8 78 ff ff ff f6 44 24 08 01 74 0b 6a 00 56 e8 79 8e 6a ff 83 c4 08 8b c6 5e c2 04 00: calls the group body destructor 00d58cf0 on this, then c… |
| `00d58d90` | GameCommerce::StoreBundleGroupDefinition::CopyCtor | confirmed | Installs the StoreBundleGroupDefinition vtable, clears +4, runs the base initialiser FUN_00d672e0, installs the HashListMap vtable at +0x3c, clears the list fi… |
| `00b25c80` | StoreBundleGroupDefinition::ScalarDeletingDtor | likely | Scalar deleting destructor: calls FUN_00d58cf0 with this in ECX (that body writes GameCommerce::StoreBundleGroupDefinition::vftable and tears down its HashList… |
| `00d58af0` | GameCommerce::StoreBundleGroupDefinition::Ctor | likely | Installs the StoreBundleGroupDefinition vtable, stores the argument at +4, clears the dwords at +8, +0xc and +0x10, runs the base initialiser FUN_00d672e0, cle… |

**GameCommerce::StoreBundleGroupDefinition::Entry** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d58400` | GameCommerce::StoreBundleGroupDefinition::Entry::Ctor | confirmed | Bytes 8b c1 c7 00 04 0c 84 01 8b 0d c8 2c bf 01 89 48 04 c7 40 08 00 00 00 00 c3: stores GameCommerce::StoreBundleGroupDefinition::Entry::vftable into *this, c… |
| `00d58740` | GameCommerce::StoreBundleGroupDefinition::Entry::FormatIdAn… | confirmed | Formats the entry's fields at +4 and +8 with the format 'storeBundleId=%d, displayOrder=%d' through FUN_00402fd0 into the output buffer, and returns the output… |

**GameCommerce::StoreBundleOfTheDay<AppStoreBundleDefinition>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099d9d0` | StoreBundleOfTheDay::StoreBundleOfTheDay | confirmed | Installs the StoreBundleOfTheDay vtable, initializes fields including maxAvailable to 0x7FFFFFFF, and calls FUN_0099C2B0(1). |
| `0099e5b0` | StoreBundleOfTheDay::ScalarDeletingDtor | confirmed | Restores the StoreBundleOfTheDay vtable, then BaseDefinition, conditionally frees the object, and returns the object pointer. |
| `009a0180` | StoreBundleOfTheDay::Format | confirmed | Formats 11 integer fields into a wide diagnostic string containing ids, date, availability, price, message id, and store id. |
| `00998450` | StoreBundleOfTheDayAppStoreBundleDtorBody | likely | Fastcall (ECX=this). Stores the GameCommerce::StoreBundleOfTheDay<AppStoreBundleDefinition> vtable (0180d864) at [this], then overwrites it with the GameCommer… |
| `0099b070` | StoreBundleOfTheDayAppStoreBundleIsValid | likely | Virtual slot [0] (body is an ordinary predicate, not a deleting destructor), fastcall ECX=this, no stack args. Same body as 0099afd0: returns 1 when [this+4], … |

**GameCommerce::StoreItemOfTheDay<AppStoreBundleDefinition>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099d8d0` | StoreItemOfTheDay::StoreItemOfTheDay | confirmed | Installs the StoreItemOfTheDay vtable, initializes scalar fields and defaults, and calls FUN_0099C1B0 with argument 1. |
| `009a00a0` | StoreItemOfTheDay::Format | confirmed | Formats 11 integer fields into a wide debug string, including id, date, time remaining, availability, price, message, store, and item ids. |
| `009983c0` | StoreItemOfTheDayAppStoreBundleDtorBody | likely | Fastcall (ECX=this). Stores the GameCommerce::StoreItemOfTheDay<AppStoreBundleDefinition> vtable (0180d850) at [this], then overwrites it with the GameCommerce… |
| `0099afd0` | StoreItemOfTheDayAppStoreBundleIsValid | likely | Virtual slot [0] (body is an ordinary predicate, not a deleting destructor), fastcall ECX=this, no stack args. Returns 1 when [this+4], [this+0xc], [this+8] an… |
| `0099e540` | — | unknown | Restores the StoreItemOfTheDay vtable, then the BaseDefinition vtable, conditionally frees the object when the deleting flag is set, and returns it. |

**GameCommerce::StorePortalCategoryDefinition** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d60470` | GameCommerce::StorePortalCategoryDefinition::StorePortalCat… | confirmed | Thiscall ctor with arg1 = integer stored at +0x04 (portal category id). Stores the StorePortalCategoryDefinition vftable at +0x00; zeroes +0x08, +0x0c, +0x10; … |
| `00d60530` | GameCommerce::StorePortalCategoryDefinition::StorePortalCat… | confirmed | Fastcall default ctor (ECX=this, no arguments). Stores the StorePortalCategoryDefinition vftable at +0x00; zeroes +0x04, +0x08, +0x0c and +0x10; constructs the… |
| `00d605a0` | GameCommerce::StorePortalCategoryDefinition::~StorePortalCa… | confirmed | Thiscall dtor body. Stores the StorePortalCategoryDefinition vftable, runs the BaseDefinition destructor body 00d60840 on the shortcut sub-object at +0x38, run… |
| `00d60650` | GameCommerce::StorePortalCategoryDefinition::ScalarDeleting… | confirmed | Virtual at vtable slot [2] of the portal category class (read from the binary). Calls the destructor body 00d605a0; if arg bit 0 is set (code: if ((param_2 & 1… |
| `00d60670` | GameCommerce::StorePortalCategoryDefinition::CopyFromDefini… | confirmed | Thiscall copy constructor with arg1 = source definition pointer. Stores the StorePortalCategoryDefinition vftable; constructs the image sub-object at +0x14 (00… |
| `00d60720` | GameCommerce::StorePortalCategoryDefinition::DescribeText | confirmed | Virtual at vtable slot [1] of the portal category class (read from the binary). Builds two temporary fixed strings by calling virtual slot [1] of the sub-objec… |
| `00d604f0` | GameCommerce::StorePortalCategoryDefinition::IdsAndSubobjec… | likely | Virtual at vtable slot [0] of the portal category class (read from the binary; ordinary predicate, not a destructor). Returns 1 only when the dword at +0x08 is… |

**GameCommerce::StoreShortcutDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d608b0` | GameCommerce::StoreShortcutDefinition::StoreShortcutDefinit… | confirmed | Fastcall ctor (ECX=this). Stores the StoreShortcutDefinition vftable at +0x00; zeroes +0x04, +0x08 and +0x0c; stores DAT_01bf3004 at +0x10. |
| `00d60970` | GameCommerce::StoreShortcutDefinition::DescribeText | confirmed | Thiscall virtual at vtable slot [1] of StoreShortcutDefinition (read from the binary). Formats 'shortcutId=%d, type=%d, storeId=%d' from +0x04, +0x08 and +0x10… |
| `00d60890` | GameCommerce::StoreShortcutDefinition::IdsValid | likely | Virtual at vtable slot [0] of StoreShortcutDefinition (ordinary predicate). Returns 1 when the dword at +0x04 is positive, the dword at +0x08 is nonzero and th… |
| `00d608d0` | GameCommerce::StoreShortcutDefinition::CopyFieldsFrom | likely | Thiscall with arg1 = source shortcut definition. Copies the dwords at source +0x04, +0x08, +0x0c and +0x10 into this +0x04, +0x08, +0x0c and +0x10. Returns not… |

**GameCommerce::UramAddNewCardResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6aff0` | GameCommerce::UramAddNewCardResponse::~UramAddNewCardRespon… | confirmed | Installs the AddNewCard response vtable, releases the two string members through FUN_00d59180, then calls the response-base destructor. |
| `00d6b690` | GameCommerce::UramAddNewCardResponse::DeletingDestructor | confirmed | Calls FUN_00d6aff0 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e290` | GameCommerce::UramAddNewCardResponse::UramAddNewCardResponse | confirmed | Initializes UramResponseBase, installs the AddNewCard response vtable, and initializes its two string members through FUN_00d59310. |

**GameCommerce::UramDeleteCardResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b070` | GameCommerce::UramDeleteCardResponse::~UramDeleteCardRespon… | confirmed | Writes the DeleteCard response vtable and jumps to FUN_00d6aab0 to release the response-base fields. |
| `00d6b6e0` | GameCommerce::UramDeleteCardResponse::DeletingDestructor | confirmed | Writes the DeleteCard response vtable, calls FUN_00d6aab0, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e370` | GameCommerce::UramDeleteCardResponse::UramDeleteCardResponse | confirmed | Initializes UramResponseBase, installs the DeleteCard response vtable, zeroes the response's trailing fields, and returns this. |

**GameCommerce::UramFinalizeThirdPartyOrderTransactionResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b080` | UramFinalizeThirdPartyOrderTransactionResponse::~UramFinali… | confirmed | Writes the finalize-transaction response vtable and jumps to FUN_00d6aab0 to release the response-base fields. |
| `00d6b710` | GameCommerce::UramFinalizeThirdPartyOrderTransactionRespons… | confirmed | Writes the finalize response vtable, calls FUN_00d6aab0, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e3c0` | UramFinalizeThirdPartyOrderTransactionResponse::UramFinaliz… | confirmed | Initializes UramResponseBase, installs the finalize-transaction response vtable, zeroes its trailing fields, and returns this. |

**GameCommerce::UramHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d56cc0` | GameCommerce::UramHandler::Dtor | confirmed | Bytes c7 01 2c 0a 84 01 c3: stores GameCommerce::UramHandler::vftable into *this and returns. Non-deleting teardown with no base call. |
| `00d56cd0` | GameCommerce::UramHandler::ScalarDeletingDtor | confirmed | Stores GameCommerce::UramHandler::vftable into *this, then calls _free(this) when bit 0 of the flags argument is set; returns this and pops one stack argument. |

**GameCommerce::UramIsAffiliatedWithPartnerResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b090` | GameCommerce::UramIsAffiliatedWithPartnerResponse::~UramIsA… | confirmed | Releases the response's fixed-8 string, resets its buffer and lengths, and calls the UramResponseBase destructor. |
| `00d6b740` | GameCommerce::UramIsAffiliatedWithPartnerResponse::Deleting… | confirmed | Calls FUN_00d6b090 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e410` | GameCommerce::UramIsAffiliatedWithPartnerResponse::UramIsAf… | confirmed | Initializes UramResponseBase, installs the affiliation response vtable, clears its flag, initializes one fixed-8 string, and returns this. |

**GameCommerce::UramManager** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a3e0` | GameCommerce::UramManager::~UramManager | confirmed | Releases the manager's request map, embedded strings, helper state, and logging header while changing the vtable through the base teardown sequence. |
| `00d6a910` | GameCommerce::UramManager::AdjustorThunkTo00d6b4f0 | confirmed | Raw bytes subtract 4 from ECX, then jump to 00d6b4f0; this function is an adjustor thunk and does not contain the target body. |
| `00d6b4f0` | GameCommerce::UramManager::DeletingDestructor | confirmed | Calls FUN_00d6a3e0 as the manager destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6d1a0` | GameCommerce::UramManager::UramManager | confirmed | Initializes the UploaderHandler and UramManager vtables, clears manager flags and request-map buckets, initializes its string members, writes 60000 at +0x28, a… |
| `00d6eba0` | GameCommerce::UramManager::HandleFundWalletPreviewResponse | confirmed | Finds the pending request by tracking ID, formats the result, copies successful response fields into a typed preview response, invokes the request callback, re… |
| `00d6ee10` | GameCommerce::UramManager::HandleFundWalletResponse | confirmed | Finds the pending request by tracking ID, formats the result, copies successful response fields into a typed wallet-fund response, invokes the request callback… |
| `00d68190` | UramManager::LogNullDebug | speculative | When the message argument is null, calls the manager logger at this+0x10c with the fixed debug diagnostic; otherwise returns. |
| `00d681b0` | UramManager::LogNullInfo | speculative | When the message argument is null, calls the manager logger with the fixed info diagnostic; otherwise returns. |
| `00d681d0` | UramManager::LogNullError | speculative | When the error-message argument is null, calls the manager logger with the fixed error diagnostic; otherwise returns. |
| `00d6e920` | — | unknown | Finds the request by tracking ID in its keyed bucket, formats the result, builds a PaymentSources response, copies successful entries into manager state, invok… |
| `00d6f0a0` | — | unknown | Finds the pending request by tracking ID, formats the result, passes successful response strings to a state helper, invokes the request callback, removes track… |

**GameCommerce::UramManager::UramAddCreditCardRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a9c0` | GameCommerce::UramManager::UramAddCreditCardRequest::~UramA… | confirmed | Installs the request vtable, releases its fixed-string members and request state, then restores the UramRequestTracker vtable. |
| `00d6b540` | GameCommerce::UramManager::UramAddCreditCardRequest::UramAd… | confirmed | Calls the UramRequestTracker constructor with type value 2, installs the AddCreditCardRequest vtable, initializes request state, and sets up fixed-24 and fixed… |
| `00d6b5f0` | GameCommerce::UramManager::UramAddCreditCardRequest::Deleti… | confirmed | Calls FUN_00d6a9c0 as the request destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |

**GameCommerce::UramManager::UramDeleteCreditCardRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a090` | GameCommerce::UramManager::UramDeleteCreditCardRequest::~Ur… | confirmed | Calls FUN_00d68510 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d684a0` | UramDeleteCreditCardRequest::Ctor | speculative | Initializes the base request tracker with type 4, installs the delete-credit-card vtable, and constructs its payload. |
| `00d68510` | UramDeleteCreditCardRequest::Dtor | speculative | Installs the derived request vtable, releases its payload through 00f801b0, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramFinalizeThirdPartyOrderTransactionRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a190` | UramFinalizeThirdPartyOrderTransactionRequest::~UramFinaliz… | confirmed | Calls FUN_00d68b90 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68b20` | UramFinalizeThirdPartyOrderTransactionRequest::Ctor | speculative | Initializes the base tracker with type 12, installs the finalize-third-party-order vtable, and constructs its payload. |
| `00d68b90` | UramFinalizeThirdPartyOrderTransactionRequest::Dtor | speculative | Installs the derived vtable, releases payload through 00f80c30, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramFundWalletRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a0d0` | GameCommerce::UramManager::UramFundWalletRequest::~UramFund… | confirmed | Calls FUN_00d686b0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68640` | UramFundWalletRequest::Ctor | speculative | Initializes the base request tracker with type 6, installs the fund-wallet vtable, and constructs the request payload. |
| `00d686b0` | UramFundWalletRequest::Dtor | speculative | Installs the derived vtable, releases its payload through 00f7cfd0, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramGetCreditCardsRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a170` | GameCommerce::UramManager::UramGetCreditCardsRequest::~Uram… | confirmed | Calls FUN_00d68ac0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68a50` | UramGetCreditCardsRequest::Ctor | speculative | Initializes the base tracker with type 11, installs the get-credit-cards vtable, and constructs its payload. |
| `00d68ac0` | UramGetCreditCardsRequest::Dtor | speculative | Installs the derived vtable, releases its payload through 00f7df40, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramIsAffiliatedWithPartnerRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a1b0` | UramIsAffiliatedWithPartnerRequest::~UramIsAffiliatedWithPa… | confirmed | Calls FUN_00d68c60 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68bf0` | UramIsAffiliatedWithPartnerRequest::Ctor | speculative | Initializes the base tracker with type 13, installs the partner-affiliation request vtable, and constructs its payload. |
| `00d68c60` | UramIsAffiliatedWithPartnerRequest::Dtor | speculative | Installs the derived vtable, releases the request payload through 00f80990, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramPaymentSourceRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a050` | GameCommerce::UramManager::UramPaymentSourceRequest::~UramP… | confirmed | Calls FUN_00d68370 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68300` | UramPaymentSourceRequest::Ctor | speculative | Initializes the base request tracker with type 1, installs the payment-source request vtable, and initializes its request payload object. |
| `00d68370` | UramPaymentSourceRequest::Dtor | speculative | Installs the derived request vtable, releases the request payload through 00f7c740, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramPreviewMembershipPurchaseRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a0f0` | UramPreviewMembershipPurchaseRequest::~UramPreviewMembershi… | confirmed | Calls FUN_00d68780 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68710` | UramPreviewMembershipPurchaseRequest::Ctor | speculative | Initializes the base tracker with type 7, installs the membership-preview vtable, and constructs its payload. |
| `00d68780` | UramPreviewMembershipPurchaseRequest::Dtor | speculative | Installs the derived vtable, releases its payload through 00f7e2b0, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramPreviewWalletFundRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a0b0` | GameCommerce::UramManager::UramPreviewWalletFundRequest::~U… | confirmed | Calls FUN_00d685e0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68570` | UramPreviewWalletFundRequest::Ctor | speculative | Initializes the base tracker with request type 5, installs the preview-wallet-fund vtable, and constructs the payload. |
| `00d685e0` | UramPreviewWalletFundRequest::Dtor | speculative | Installs the derived vtable, releases the request payload through 00f7cac0, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramPurchaseMembershipRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a110` | GameCommerce::UramManager::UramPurchaseMembershipRequest::~… | confirmed | Calls FUN_00d68850 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d687e0` | UramPurchaseMembershipRequest::Ctor | speculative | Initializes the base request tracker with type 8, installs the membership-purchase vtable, and constructs the purchase payload. |
| `00d68850` | UramPurchaseMembershipRequest::Dtor | speculative | Installs the derived vtable, releases purchase payload through 00f7eb30, and restores the base tracker vtable. |

**GameCommerce::UramManager::UramRedeemCodeRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a130` | GameCommerce::UramManager::UramRedeemCodeRequest::~UramRede… | confirmed | Calls FUN_00d68920 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d688b0` | UramRedeemCodeRequest::Ctor | speculative | Initializes the base tracker with type 9, installs the redeem-code vtable, and constructs its request payload. |
| `00d68920` | UramRedeemCodeRequest::Dtor | speculative | Installs the derived vtable, releases the request payload through 00f7f080, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramRequestTracker** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a030` | GameCommerce::UramManager::UramRequestTracker::WriteVtableA… | confirmed | Writes the UramRequestTracker vtable, frees this only when flag bit 0 is set, and otherwise returns this; it does not call a destructor body. |
| `00d682f0` | UramRequestTracker::Dtor | speculative | Stores the request tracker vtable pointer and returns without other work. |
| `00d68220` | — | unknown | Stores request type and two constructor arguments, installs the tracker vtable, initializes four linked-list endpoints and counters, then obtains a synchroniza… |

**GameCommerce::UramManager::UramSelectCodeRedemptionOptionRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a150` | UramSelectCodeRedemptionOptionRequest::~UramSelectCodeRedem… | confirmed | Calls FUN_00d689f0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d68980` | UramSelectCodeRedemptionOptionRequest::Ctor | speculative | Initializes the base tracker with type 10, installs the option-selection request vtable, and initializes the payload. |
| `00d689f0` | UramSelectCodeRedemptionOptionRequest::Dtor | speculative | Installs the derived vtable, releases its payload through 00f7f080, then restores the base tracker vtable. |

**GameCommerce::UramManager::UramUpdateCreditCardRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6a070` | GameCommerce::UramManager::UramUpdateCreditCardRequest::~Ur… | confirmed | Calls FUN_00d68440 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d683d0` | UramUpdateCreditCardRequest::Ctor | speculative | Initializes the base request tracker with type 3, installs the update-credit-card vtable, and constructs the request payload. |
| `00d68440` | UramUpdateCreditCardRequest::Dtor | speculative | Installs the derived request vtable, releases its payload through 00f7f970, then restores the base tracker vtable. |

**GameCommerce::UramMembershipPurchaseResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b270` | GameCommerce::UramMembershipPurchaseResponse::~UramMembersh… | confirmed | Installs the membership response vtable, cleans six fixed-8 strings and a fixed-24 string, then calls the UramResponseBase destructor. |
| `00d6b760` | GameCommerce::UramMembershipPurchaseResponse::DeletingDestr… | confirmed | Calls FUN_00d6b270 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e480` | GameCommerce::UramMembershipPurchaseResponse::UramMembershi… | confirmed | Initializes UramResponseBase, installs the membership response vtable, then initializes a fixed-24 string and the record's string collection. |

**GameCommerce::UramPaymentSourcesResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6af30` | GameCommerce::UramPaymentSourcesResponse::~UramPaymentSourc… | confirmed | Installs the PaymentSources response vtable, cleans its item collection and fixed-8 string, then calls the UramResponseBase destructor. |
| `00d6b670` | GameCommerce::UramPaymentSourcesResponse::DeletingDestructor | confirmed | Calls FUN_00d6af30 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e1d0` | GameCommerce::UramPaymentSourcesResponse::UramPaymentSource… | confirmed | Initializes UramResponseBase, installs the PaymentSources response vtable, clears its flags and fixed-8 string, and initializes the payment-source collection. |

**GameCommerce::UramResponseBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6aab0` | GameCommerce::UramResponseBase::~UramResponseBase | confirmed | Installs the response-base vtable, decrements the fixed-256 string reference count, resets its buffer and lengths, and restores the IString vtable. |
| `00d6b610` | GameCommerce::UramResponseBase::DeletingDestructor | confirmed | Calls FUN_00d6aab0 as the response-base destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6df20` | GameCommerce::UramResponseBase::UramResponseBase | confirmed | Installs the response-base vtable, initializes its default status fields and fixed-256 string, zeroes two trailing fields, and returns this. |

**GameCommerce::UramUpdateCardResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b060` | GameCommerce::UramUpdateCardResponse::~UramUpdateCardRespon… | confirmed | Writes the UpdateCard response vtable and jumps to FUN_00d6aab0 to release the response-base fields. |
| `00d6b6b0` | GameCommerce::UramUpdateCardResponse::DeletingDestructor | confirmed | Writes the UpdateCard response vtable, calls FUN_00d6aab0, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e320` | GameCommerce::UramUpdateCardResponse::UramUpdateCardResponse | confirmed | Initializes UramResponseBase, installs the UpdateCard response vtable, zeroes the response's trailing fields, and returns this. |

**GameCommerce::UramWalletFundPreviewResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b630` | GameCommerce::UramWalletFundPreviewResponse::DeletingDestru… | confirmed | Calls FUN_00d6ab10 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6ab10` | — | unknown | Releases the response's fixed-1024 string, six fixed-8 strings, and fixed-24 string, then calls the UramResponseBase destructor. |
| `00d6df70` | — | unknown | Initializes UramResponseBase, installs the wallet-preview response vtable, initializes its fixed-24, six fixed-8 and fixed-1024 string fields, and zeroes trail… |

**GameCommerce::UramWalletFundResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6b650` | GameCommerce::UramWalletFundResponse::DeletingDestructor | confirmed | Calls FUN_00d6ad00 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6ad00` | — | unknown | Releases two fixed-1024 strings, six fixed-8 strings, and a fixed-24 string, then calls the UramResponseBase destructor. |
| `00d6e090` | — | unknown | Initializes UramResponseBase, installs the wallet-fund response vtable, initializes its fixed-24, fixed-8 and two fixed-1024 strings, and zeroes trailing field… |

**InGamePurchaseDataSource::AcceptedCreditCardTypes** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997fc0` | AcceptedCreditCardTypesFieldNameForIndex | confirmed | Virtual slot [1] of AcceptedCreditCardTypes, one stack arg (index), no calls. Switch: 0 Id, 1 CardName, 2 CardSymbol, 3 CardIconId, 4 CsvHelperIconId, 5 CardNu… |
| `009a74c0` | InGamePurchaseDataSource::AcceptedCreditCardTypes::~Accepte… | confirmed | Destructor body: writes AcceptedCreditCardTypes::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FU… |
| `009a8020` | InGamePurchaseDataSource::AcceptedCreditCardTypes::Accepted… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs AcceptedCre… |
| `0099ada0` | AcceptedCreditCardTypesSlot1RefreshFrom3ca34 | likely | Virtual slot [1] of AcceptedCreditCardTypes, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x… |

**InGamePurchaseDataSource::CountryCodes** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00998050` | CountryCodesFieldNameForIndex | confirmed | Virtual slot [1] of CountryCodes, one stack arg (index), no calls. Switch: 0 Id, 1 CountryCode, 2 LanguageLocale, 3 CountryLocale, 4 CountryName; any other ind… |
| `009a7580` | InGamePurchaseDataSource::CountryCodes::~CountryCodes | confirmed | Destructor body: writes CountryCodes::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60;… |
| `009a80e0` | InGamePurchaseDataSource::CountryCodes::CountryCodes | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs CountryCode… |
| `0099adc0` | CountryCodesSlot1RefreshFrom3ce08 | likely | Virtual slot [1] of CountryCodes, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3ce08 into … |

**InGamePurchaseDataSource::CreditCardPayment** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997e50` | CreditCardPaymentFieldNameForIndex | confirmed | Virtual slot [1] of CreditCardPayment, one stack arg (index), no calls. Switch: 0 PaymentSourceId, 1 CardTypeId, 2 CardSpan, 3 CanDelete, 4 AccountText; any ot… |
| `009a7280` | InGamePurchaseDataSource::CreditCardPayment::~CreditCardPay… | confirmed | Destructor body: writes CreditCardPayment::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091… |
| `009a7de0` | InGamePurchaseDataSource::CreditCardPayment::CreditCardPaym… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs CreditCardP… |

**InGamePurchaseDataSource::GiftOrderEntries** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997e00` | GiftOrderEntriesFieldNameForIndex | confirmed | Virtual slot [1] of GiftOrderEntries, one stack arg (index), no calls. Index 0 returns StoreId, 1 StoreBundleId, 2 Quantity; any other index returns "<error>". |
| `0099d670` | GiftOrderEntries::InitThunk | confirmed | Adjusts ECX by -0x254 and jumps directly to 0099ACF0; this is an adjustor thunk, not the target body. |
| `009a71c0` | InGamePurchaseDataSource::GiftOrderEntries::~GiftOrderEntri… | confirmed | Destructor body: writes GiftOrderEntries::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091b… |
| `009a7d20` | InGamePurchaseDataSource::GiftOrderEntries::GiftOrderEntries | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs GiftOrderEn… |

**InGamePurchaseDataSource::GiftOrderHistory** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997da0` | GiftOrderHistoryFieldNameForIndex | confirmed | Virtual slot [1] of GiftOrderHistory, one stack arg (index), no calls. Switch: 0 Id, 1 PurchaserGuid, 2 PurchaserName, 3 GiftMessage, 4 TimeStamp, 5 ItemEntryC… |
| `009a7100` | InGamePurchaseDataSource::GiftOrderHistory::~GiftOrderHisto… | confirmed | Destructor body: writes GiftOrderHistory::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091b… |
| `009a7c60` | InGamePurchaseDataSource::GiftOrderHistory::GiftOrderHistory | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs GiftOrderHi… |

**InGamePurchaseDataSource::InternalDsBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099a9e0` | InternalDsBaseScalarDeletingDtor | confirmed | Thiscall (ECX=this) with one stack arg (byte flags). Stores the InGamePurchaseDataSource::InternalDsBase vtable (0180d2fc) at [this], and if bit 0 of the flags… |
| `00997780` | InGamePurchaseDataSourceInternalDsBaseDtorBody | likely | Fastcall (ECX=this), 7 bytes: stores the InGamePurchaseDataSource::InternalDsBase vtable (0180d2fc) to [this] and returns. Releases nothing. |

**InGamePurchaseDataSource::MerchantSellBundles** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009980f0` | MerchantSellBundlesFieldNameForIndex | confirmed | Virtual slot [1] of MerchantSellBundles, one stack arg (index). Index 0x22 returns QuantityAvailable; any other index calls 00997840 and returns its result. 26… |
| `009a8cd0` | InGamePurchaseDataSource::MerchantSellBundles::MerchantSell… | confirmed | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), installs MerchantSellBundles::vftable at [0], [0xc], [0xd], [0x95], [0x… |
| `009a8dc0` | InGamePurchaseDataSource::MerchantSellBundles::~MerchantSel… | confirmed | MerchantSellBundles destructor: writes MerchantSellBundles::vftable to [0], [0xc], [0xd], [0x95], [0x9c], destroys the MerchantSellBundleEntry hash map via FUN… |

**InGamePurchaseDataSource::StateCodes** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009980b0` | StateCodesFieldNameForIndex | confirmed | Virtual slot [1] of StateCodes, one stack arg (index), no calls. Index 0 returns Id, 1 StateCode, 2 StateName; any other index returns "<error>". |
| `009a7640` | InGamePurchaseDataSource::StateCodes::~StateCodes | confirmed | Destructor body: writes StateCodes::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; p… |
| `009a81a0` | InGamePurchaseDataSource::StateCodes::StateCodes | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs StateCodes:… |
| `0099ade0` | StateCodesSlot1RefreshFrom3d61c | likely | Virtual slot [1] of StateCodes, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3d61c into th… |

**InGamePurchaseDataSource::StationCashPriceList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997eb0` | StationCashPriceListFieldNameForIndex | confirmed | Virtual slot [1] of StationCashPriceList, one stack arg (index), no calls. Switch: 0 Id, 1 TotalStationCashAmount, 2 PromotionalStationCashAmount, 3 Promotiona… |
| `009a7340` | InGamePurchaseDataSource::StationCashPriceList::~StationCas… | confirmed | Destructor body: writes StationCashPriceList::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0… |
| `009a7ea0` | InGamePurchaseDataSource::StationCashPriceList::StationCash… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs StationCash… |
| `0099ad60` | StationCashPriceListSlot1RefreshFrom3b5d8 | likely | Virtual slot [1] of StationCashPriceList, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3b5… |

**InGamePurchaseDataSource::StoreBillboardPanels** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997cc0` | StoreBillboardPanelsFieldNameForIndex | confirmed | Virtual slot [1] of StoreBillboardPanels, one stack arg (index), no calls. Switch: 0 Id, 1 BillboardId, 2 Name, 3 ImageId, 4 IsMembersOnly, 5 ShortcutType, 6 S… |
| `009a6e70` | InGamePurchaseDataSource::StoreBillboardPanels::~StoreBillb… | confirmed | Destructor body: writes StoreBillboardPanels::vftable to this[0], this[0xc], this[0xd], this[0x95], then overwrites this[0x95] with InternalDsBase::vftable and… |
| `009a7a10` | InGamePurchaseDataSource::StoreBillboardPanels::StoreBillbo… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs StoreBillbo… |

**InGamePurchaseDataSource::StoreBillboards** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997c80` | StoreBillboardsFieldNameForIndex | confirmed | Virtual slot [1] of StoreBillboards, one stack arg (index), no calls. Index 0 returns ColumnId, 1 LocationId, 2 PanelCount; any other index returns "<error>". |
| `009a6db0` | InGamePurchaseDataSource::StoreBillboards::~StoreBillboards | confirmed | Destructor body: writes StoreBillboards::vftable to this[0], this[0xc], this[0xd], this[0x95], then overwrites this[0x95] with InternalDsBase::vftable and call… |
| `009a7960` | InGamePurchaseDataSource::StoreBillboards::StoreBillboards | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], then installs StoreB… |

**InGamePurchaseDataSource::StoreBundleActiveFilters** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997d50` | StoreBundleActiveFiltersFieldNameForIndex | confirmed | Virtual slot [1] of StoreBundleActiveFilters, one stack arg (index), no calls. Index 0 returns FilterId, 1 returns OptionList; any other index returns "<error>… |
| `009a6f30` | InGamePurchaseDataSource::StoreBundleActiveFilters::~StoreB… | confirmed | Destructor body: writes StoreBundleActiveFilters::vftable to this[0], [0xc], [0xd], [0x95], then under EH state 1 calls FUN_00402140 (SoeUtil::IStringFixed256 … |
| `009a7ad0` | InGamePurchaseDataSource::StoreBundleActiveFilters::StoreBu… | likely | Constructor: base ctor FUN_00922890(param_2), InternalDsBase::vftable at [0x95], owner word param_3 at [0x96], StoreBundleActiveFilters::vftable at [0], [0xc],… |

**InGamePurchaseDataSource::StoreBundleActiveSort** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997d80` | StoreBundleActiveSortFieldNameForIndex | confirmed | Virtual slot [1] of StoreBundleActiveSort, one stack arg (index), no calls. Index 0 returns SortId; any other index returns "<error>". Bytes-equivalent to the … |
| `009a7010` | InGamePurchaseDataSource::StoreBundleActiveSort::~StoreBund… | confirmed | Destructor body: writes StoreBundleActiveSort::vftable to this[0], [0xc], [0xd], [0x95], then under EH state 1 calls FUN_00402140 (IStringFixed256 dtor), reset… |
| `009a7ba0` | InGamePurchaseDataSource::StoreBundleActiveSort::StoreBundl… | confirmed | Constructor: base ctor FUN_00922890(param_2), InternalDsBase::vftable at [0x95], owner word param_3 at [0x96], StoreBundleActiveSort::vftable at [0], [0xc], [0… |

**InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997ab0` | CategoriesTreeNodeSlot4FieldName | confirmed | Virtual slot [4] of CategoriesTreeNode, one stack arg (undefined4 index), no calls. Returns a static name: 0 CategoryId, 1 StoreBundleCount, 2 TintValue; any o… |
| `0099ffa0` | CategoriesTreeNodeDtor | confirmed | Restores the category-node vtable, runs a member cleanup helper, changes the embedded field to Loggable's vtable, and invokes the GUI tree-node cleanup body. |
| `009a0d90` | CategoriesTreeNodeCtor | confirmed | Calls GUI tree-node base constructor 0099EF20 with this in ECX, installs the category-node vtable, stores supplied identifiers and fields, constructs icon data… |
| `009a0e50` | CategoriesTreeNodeScalarDtor | confirmed | Calls the category-node cleanup body and conditionally frees the object when the deleting flag is set. |

**InGamePurchaseDataSource::StoreBundleGroupContents** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a84c0` | InGamePurchaseDataSource::StoreBundleGroupContents::~StoreB… | confirmed | Destructor: writes StoreBundleGroupContents::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then calls the StoreBundles base destructor FUN_009a83b0 under the s… |
| `009a89d0` | InGamePurchaseDataSource::StoreBundleGroupContents::StoreBu… | confirmed | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), then installs StoreBundleGroupContents::vftable at [0], [0xc], [0xd], [… |

**InGamePurchaseDataSource::StoreBundleGroups** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997af0` | StoreBundleGroupsFieldNameForIndex | confirmed | Virtual slot [1] of StoreBundleGroups, one stack arg (index), no calls. Switch: 0 GroupId, 1 Name, 2 ImageSetId, 3 TintValue; any other index returns "<error>". |
| `009a77f0` | InGamePurchaseDataSource::StoreBundleGroups::StoreBundleGro… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner/context word param_3 at [0x96], installs Sto… |

**InGamePurchaseDataSource::StoreBundleOfTheDay** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997bb0` | StoreBundleOfTheDayFieldNameForIndex | confirmed | Virtual slot [1] of StoreBundleOfTheDay, one stack arg (index). Switch: 0x22 QuantityAvailable, 0x23 TimeRemaining, 0x24 IOTDPrice, 0x25 Message. Any other ind… |
| `009a86e0` | InGamePurchaseDataSource::StoreBundleOfTheDay::~StoreBundle… | confirmed | Destructor: writes StoreBundleOfTheDay::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then rewrites the member at this+0x4b8 (byte 0x12e0) from GameCommerce::S… |
| `009a8c10` | InGamePurchaseDataSource::StoreBundleOfTheDay::StoreBundleO… | confirmed | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), installs StoreBundleOfTheDay::vftable at [0], [0xc], [0xd], [0x95], [0x… |

**InGamePurchaseDataSource::StoreBundleRoot** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0099aa00` | StoreBundleRootScalarDeletingDtor | confirmed | Thiscall (ECX=this) with one stack arg (byte flags) and an SEH frame. Stores the InGamePurchaseDataSource::StoreBundleRoot vtable (0180d308) at [this], calls 0… |
| `00997790` | StoreBundleRootCtor | likely | Thiscall (ECX=this) with one stack arg param_2, returns this (ret 4). Calls 00822990 with param_2 pushed, then stores the InGamePurchaseDataSource::StoreBundle… |
| `009977f0` | StoreBundleRootDtorBody | likely | Fastcall (ECX=this) with an SEH frame. Stores the InGamePurchaseDataSource::StoreBundleRoot vtable (0180d308) at [this], then calls 00822810 once (the base tea… |

**InGamePurchaseDataSource::StoreBundleVipLevelContents** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a8550` | InGamePurchaseDataSource::StoreBundleVipLevelContents::~Sto… | confirmed | Destructor: writes StoreBundleVipLevelContents::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then calls the StoreBundles base destructor FUN_009a83b0 under th… |
| `009a8a90` | InGamePurchaseDataSource::StoreBundleVipLevelContents::Stor… | confirmed | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), then installs StoreBundleVipLevelContents::vftable at [0], [0xc], [0xd]… |

**InGamePurchaseDataSource::StoreBundles** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997840` | StoreBundleFieldNameForIndex | confirmed | Virtual, one stack arg (undefined4 index), ret 4, no calls. Switches on the index and returns a pointer to a static field-name string: 0 ItemId, 1 StoreId, 2 C… |
| `009a83b0` | InGamePurchaseDataSource::StoreBundles::~StoreBundles | confirmed | StoreBundles destructor: writes StoreBundles::vftable to [0], [0xc], [0xd], [0x95], [0x9c]; under EH states destroys the embedded SearchFilter HashListMap (FUN… |
| `009a8880` | InGamePurchaseDataSource::StoreBundles::StoreBundles | confirmed | StoreBundles constructor: base ctor FUN_00922890(param_2), member init FUN_0081b470, InternalDsBase::vftable at [0x9c] and owner word param_3 at [0x9d], then S… |

**InGamePurchaseDataSource::StoreItemOfTheDay** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997b60` | StoreItemOfTheDayFieldNameForIndex | confirmed | Virtual slot [1] of StoreItemOfTheDay, one stack arg (index). Switch: 0x22 QuantityAvailable, 0x23 TimeRemaining, 0x24 IOTDPrice, 0x25 Message. Any other index… |
| `009a85e0` | InGamePurchaseDataSource::StoreItemOfTheDay::~StoreItemOfTh… | confirmed | Destructor: writes StoreItemOfTheDay::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then rewrites the member at this+0x4b8 (byte 0x12e0) from GameCommerce::Sto… |
| `009a8b50` | InGamePurchaseDataSource::StoreItemOfTheDay::StoreItemOfThe… | confirmed | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), installs StoreItemOfTheDay::vftable at [0], [0xc], [0xd], [0x95], [0x9c… |

**InGamePurchaseDataSource::StorePortalCategories** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997c00` | StorePortalCategoriesFieldNameForIndex | confirmed | Virtual slot [1] of StorePortalCategories, one stack arg (index), no calls. Switch: 0 Id, 1 Name, 2 Description, 3 ImageSetId, 4 TintValue, 5 ShortcutType, 6 S… |
| `009a6cf0` | InGamePurchaseDataSource::StorePortalCategories::~StorePort… | confirmed | Destructor body: writes StorePortalCategories::vftable to this[0], this[0xc], this[0xd] and this[0x95] (byte offsets 0, 0x30, 0x34, 0x254), then overwrites thi… |
| `009a78b0` | InGamePurchaseDataSource::StorePortalCategories::StorePorta… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], then installs StoreP… |

**InGamePurchaseDataSource::SubscriptionPriceList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00997f20` | SubscriptionPriceListFieldNameForIndex | confirmed | Virtual slot [1] of SubscriptionPriceList, one stack arg (index), no calls. Switch: 0 Id, 1 Name, 2 Description, 3 PriceLocalCurrency, 4 PriceStationCash, 5 Is… |
| `009a7400` | InGamePurchaseDataSource::SubscriptionPriceList::~Subscript… | confirmed | Destructor body: writes SubscriptionPriceList::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_… |
| `009a7f60` | InGamePurchaseDataSource::SubscriptionPriceList::Subscripti… | confirmed | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs Subscriptio… |
| `0099ad80` | SubscriptionPriceListSlot1RefreshFrom3b980 | likely | Virtual slot [1] of SubscriptionPriceList, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3b… |

**InGamePurchaseSearchEngine** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc4c80` | InGamePurchaseSearchEngine::InGamePurchaseSearchEngine | confirmed | Calls 00c30040 with this as receiver, installs the InGamePurchaseSearchEngine vtable, and stores param_2 at this+0x1080. |
| `00cc4cf0` | InGamePurchaseSearchEngine::~InGamePurchaseSearchEngine | confirmed | Restores the InGamePurchaseSearchEngine vtable and calls 00c2fa70 with this as receiver to destroy the owned member. |
| `00cc4d40` | InGamePurchaseSearchEngine::ScalarDeletingDestructor | confirmed | Restores the class vtable, destroys the object member through 00c2fa70 with this as receiver, frees this only when flag bit 0 is set, otherwise returns this, a… |
| `00cc4da0` | InGamePurchaseSearchEngine::FindBundleItem | confirmed | Looks up the bundle by param_1, walks its linked item entries, searches the item-definition hash table by each entry key, writes the first matching definition … |
| `00cc4e40` | InGamePurchaseSearchEngine::ForwardBundleFieldA0 | confirmed | Looks up a bundle by param_1; if found, obtains the object at manager+0x10 and invokes its vtable slot +8 with bundle field +0xa0 and param_2. |
| `00cc50f0` | InGamePurchaseSearchEngine::SearchBundles | confirmed | Resolves the requested bundle, checks the active store context, queries localized strings and associated item maps, then appends matching identifiers and displ… |

**InGamePurchaseSessionStats::IdEntryStoreBundleGroupId** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2a1d0` | IdEntryStoreBundleGroupId::Constructor | speculative | Calls the IdEntry constructor and stores a supplied dword at object offset 0x20. |

**InGamePurchaseSessionStats::IdEntryStoreBundleId** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2a170` | IdEntryStoreBundleId::Constructor | speculative | Calls the IdEntry constructor and stores a supplied dword at object offset 0x20. |

**InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b40370` | InGamePurchaseSessionStatsCtor | confirmed | Constructor of the session-stats object: writes Loggable::vftable then InGamePurchaseSessionStats::vftable at +0, zeroes +4/+5 and +0xc, sets +8 to -1, initial… |
| `00b40630` | InGamePurchaseSessionStatsDump | confirmed | Virtual slot 0 of the session-stats vtable: writes the session summary through FUN_00720c60 - total shopping time (+0x18-+0x10) with referrer id (+8) and refer… |
| `00b40930` | InGamePurchaseSessionStatsDtor | confirmed | Destructor of the session-stats object: rewrites the class vtable, tears down the trailing member through FUN_00b358c0, runs the transaction-tracker list destr… |
| `00b40b30` | InGamePurchaseSessionStats_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 1): runs the session-stats destructor body 00b40930 and, when bit 0 of the flag byte argument is set, releases this wit… |

**InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b221b0` | InGamePurchaseSessionStats_IdEntry_StoreBundleGroupIdDtor | confirmed | Destructor body: stores the IdEntry vtable first, then the Loggable base vtable, into *this, and returns. |
| `00b261f0` | InGamePurchaseSessionStats_IdEntry_StoreBundleGroupIdDeleti… | confirmed | Scalar deleting destructor: writes the IdEntry<StoreBundleGroupId> vtable then Loggable::vftable and frees this via _free when the flags low bit is set. |
| `00b27bc0` | InGamePurchaseSessionStats::IdEntryStoreBundleGroupIdDefaul… | confirmed | Default constructor: writes Loggable::vftable then the IdEntry<StoreBundleGroupId> vtable and zeroes this+4 and this+8. |
| `00b28fb0` | IdEntryStoreBundleGroupIdInitSetField20 | likely | Inside an SEH frame, calls FUN_00b25ef0 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b29300` | HashListRemoveIdEntryStoreBundleGroupIdNode | likely | Same removal as 00b29240 but with bucket = param_2[8] & 0xf and the payload destroyed by writing the IdEntry<StoreBundleGroupId> vtable then Loggable::vftable … |
| `00b25ef0` | — | unknown | Constructor: writes Loggable::vftable then the IdEntry<StoreBundleGroupId> vtable, copies the typed key from *param_2 into this+4, zeroes this+8, seeds this+0x… |

**InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b20fa0` | IdEntryGroupIdDtor | confirmed | Destructor body storing the IdEntry<StoreBundleGroupId> vtable at [param_1] and then Loggable::vftable; reached from five EH unwind funclets. |
| `00b21430` | IdEntryGroupIdDeletingDtor | confirmed | Virtual slot 1 of the IdEntry<StoreBundleGroupId> vtable: stores the IdEntry vtable, stores Loggable::vftable and frees the object with 00d82463 when bit 0 of … |
| `00b356c0` | StoreBundleGroupIdSessionStat::Log | confirmed | Formats entry timestamp then logs entry id, count and last-occurrence string. |

**InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b22160` | InGamePurchaseSessionStats_IdEntry_StoreBundleIdDtor | confirmed | Destructor body: stores the IdEntry vtable first, then the Loggable base vtable, into *this (own-then-base order identifies a destructor), and returns. |
| `00b26180` | InGamePurchaseSessionStats_IdEntry_StoreBundleIdDeletingDtor | confirmed | Scalar deleting destructor: writes the IdEntry<StoreBundleId> vtable then Loggable::vftable to *this and frees this via _free when the flags low bit is set. |
| `00b27b60` | InGamePurchaseSessionStats::IdEntryStoreBundleIdDefaultCtor | confirmed | Default constructor: writes Loggable::vftable then the IdEntry<StoreBundleId> vtable and zeroes this+4 and this+8; no timestamp is loaded. |
| `00b28f50` | IdEntryStoreBundleIdInitSetField20 | likely | Inside an SEH frame, calls FUN_00b25e60 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b29240` | HashListRemoveIdEntryStoreBundleIdNode | likely | Removes node param_2 from the 32-bucket chain (bucket = param_2[8] & 0x1f, table this+0x14, chain param_2[9]) and from the list (next/prev at +0x18/+0x1c, head… |
| `00b25e60` | — | unknown | Constructor: writes Loggable::vftable then the IdEntry<StoreBundleId> vtable, copies the typed key from *param_2 into this+4, zeroes this+8, seeds this+0x10/0x… |

**InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b20ea0` | IdEntryBundleIdDtor | confirmed | Destructor body that stores the IdEntry<StoreBundleId> vtable at [param_1] and then Loggable::vftable; the two stores are the whole body and it is reached from… |
| `00b213c0` | IdEntryBundleIdDeletingDtor | confirmed | Deleting destructor for IdEntry<StoreBundleId>: stores its vtable and Loggable::vftable, then frees the object through thunk 00d82463 when bit 0 of param_2 is … |
| `00b355c0` | StoreBundleIdSessionStat::Log | confirmed | Formats entry timestamp then logs entry id, count and last-occurrence string. |

**InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b21090` | IdEntryIntDtor | confirmed | Destructor body storing the IdEntry<int> vtable at [param_1] and then Loggable::vftable; reached from nine EH unwind funclets. |
| `00b214a0` | IdEntryIntDeletingDtor | confirmed | Virtual slot 1 of the IdEntry<int> vtable: stores the IdEntry vtable, stores Loggable::vftable and frees the object with 00d82463 when bit 0 of param_2 is set;… |
| `00b21fb0` | InGamePurchaseSessionStats::IdEntryCtor | confirmed | Constructor for IdEntry<int>: writes Loggable::vftable then the IdEntry<int> vftable to *this, copies the int key from *param_2 into this+4, zeroes this+8, and… |
| `00b22b80` | InGamePurchaseSessionStats_IdEntryIntDefaultCtor | confirmed | Constructor variant: writes Loggable::vftable then the IdEntry<int> vtable to *this (base then own order) and zeroes this+4 and this+8; it does not load the ti… |
| `00b26260` | InGamePurchaseSessionStats_IdEntryIntDeletingDtor | confirmed | Scalar deleting destructor: writes the IdEntry<int> vtable then Loggable::vftable and frees this via _free when the flags low bit is set. |
| `00b357c0` | IntIdSessionStat::Log | confirmed | Formats entry timestamp then logs entry id, count and last-occurrence string. |
| `00b22200` | InGamePurchaseSessionStats_IdEntryIntDtor | likely | Destructor body: writes the IdEntry<int> vtable then the Loggable base vtable to *this and returns; body is byte-identical to the copy at 00b22250. |
| `00b22250` | InGamePurchaseSessionStats_IdEntryIntDtorVariant | likely | Second byte-identical copy of the IdEntry<int> destructor body: writes the IdEntry<int> vtable then Loggable::vftable to *this and returns. |
| `00b25bc0` | IdEntryIntDerivedCtor | likely | Inside an SEH frame, calls FUN_00b21fb0 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b25c20` | IdEntryIntDerivedCtorVariant | likely | Inside an SEH frame, calls FUN_00b21fb0 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b262d0` | InGamePurchaseSessionStats_IdEntryIntDeletingDtorVariant | likely | Second copy of the IdEntry<int> scalar deleting destructor: own vtable, then Loggable::vftable, then _free on the flags low bit. |
| `00b27b00` | IdEntryIntInitThenSetField20 | likely | Inside an SEH frame runs the default IdEntry<int> constructor FUN_00b22b80 with this, then stores *param_2 at this+0x20 and returns this. |
| `00b293c0` | HashListRemoveIdEntryIntNode6 | likely | Removes an int-keyed entry: unlinks the node from its 6-bucket chain via FUN_00b22520, splices it out of the list (next/prev at +0x18/+0x1c, head this+8, tail … |
| `00b29460` | HashListRemoveIdEntryIntNode | likely | Removal of an int-keyed entry with its own chain walk: bucket = param_2[8] & 0xf, chain link param_2[9], table this+0x14; then splices the list (next/prev +0x1… |

**LobbyGame** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00912310` | — | unknown | __fastcall (this=ECX). Takes head node [this+4]; if null returns. Computes bucket = node[3] % 100 with buckets at this+0x10; walks that bucket chain to the nod… |

**SoeUtil::HashList<InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a1ad0` | HashListMerchantSellCtor | confirmed | Initializes the typed HashListMap, zeroes map fields and 0x1000 bucket bytes, then installs the HashList vtable. |

**SoeUtil::HashList<InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a1a50` | HashListStoreBundlesCtor | confirmed | Initializes the typed HashListMap, zeroes bucket pointers and size fields, clears 0x1000 bytes of bucket array, then installs the HashList vtable. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureCasTrackingId,0>,GameCommerce::CasApi::RequestTracker,16,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d75ac0` | HashListMapRequestTracker16Dtor | confirmed | Resets the vtable to the HashListMap<...,16,-1> vtable and removes every node from the head at +8 through 00d75010 until the head is null. It does not free the… |
| `00d75b10` | HashListMapRequestTrackerScalarDeletingDtor | confirmed | Stores the HashListMap<...,16,-1> vtable, removes every tracker node through 00d75010 until the head at +8 is null, and calls _free(this) when bit 0 of the fla… |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureCasTrackingId,0>,GameCommerce::CasApi::RequestTracker,16,4>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d75e70` | HashListMapRequestTracker16x4Ctor | confirmed | Constructor for the 16-bucket map with 4 pooled slots: zeroes the counters and head, clears a 0x40-byte area through _memset, stores the HashListMap<...,16,4> … |
| `00d76830` | HashListMapRequestTracker16x4Alloc | confirmed | Thunk: adds 0x54 to ECX (the pool sub-object of the map) and jumps to BlockPoolPop 00d76750 (allocate one block). |
| `00d76840` | HashListMapRequestTracker16x4Free | confirmed | Thunk: adds 0x54 to ECX (the pool sub-object of the map) and jumps to BlockPoolPush 00d767c0 (return one block). |
| `00d76850` | HashListMapRequestTracker16x4DtorBody | confirmed | Sets the 16,4 map vtable, removes all nodes from the head at +8 through 00d75010, tears down the chunk registry tree at +0x54 through 00d74120 and releases its… |
| `00d76920` | HashListMapRequestTracker16x4ScalarDeletingDtor | confirmed | Calls the 16,4 destructor body 00d76850, then calls _free(this) when bit 0 of the flag argument is set. It returns this. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,SoeUtil::StrongType<int,class_GameCommerce::SoeUtilSignatureStoreBundleId,1>,1024,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b20860` | CtorHashListMapBundleId1024 | confirmed | Constructor: stores the 1024-bucket HashListMap vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x1000 bytes at +0x14 (1024 bucket heads) to zero and zeroes +… |
| `00b2cb50` | StoreBundleIdHashMap::Dtor | confirmed | Restores the store-bundle-ID hash-map vtable and repeatedly removes each remaining entry. |
| `00b2ccd0` | StoreBundleIdHashMap::ScalarDeletingDtor | confirmed | Restores the hash-map vtable, removes all entries, and frees the map object if the deleting flag is set. |

**SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,SoeUtil::StrongType<int,class_GameCommerce::SoeUtilSignatureStoreBundleId,1>,128,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b209a0` | CtorHashListMapBundleId128 | confirmed | Constructor: stores the 128-bucket bundle-id HashListMap vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x200 bytes at +0x14 (128 bucket heads) to zero and z… |

**SoeUtil::HashListMap<int,AppStoreBundleDefinition,32,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b20a60` | CtorHashListMapAppStoreBundle32 | confirmed | Constructor: stores SoeUtil::HashListMap<int,AppStoreBundleDefinition,32,-1>::vftable at +0, zeroes +0x10, +8 and +0xc, memsets 0x80 bytes at +0x14 (32 bucket … |

**SoeUtil::HashListMap<int,GameCommerce::CurrencyCodeData,32,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b208c0` | CtorHashListMapCurrencyCode32 | confirmed | Constructor: stores SoeUtil::HashListMap<int,GameCommerce::CurrencyCodeData,32,-1>::vftable at +0, zeroes +0x10, +8 and +0xc, memsets 0x80 bytes at +0x14 (32 b… |

**SoeUtil::HashListMap<int,GameCommerce::MarketingDataRecordSetDefinition,32,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7a730` | MarketingRecordMap32_DtorBody | confirmed | Reads the head at +8, stores the HashListMap<int,MarketingDataRecordSetDefinition,32,-1> vftable at this+0, then loops calling 00d79c40 on the head while it is… |
| `00d7a760` | MarketingRecordMap32_ScalarDeletingDtor | confirmed | Same body as MarketingRecordMap32_DtorBody (vftable store, loop calling 00d79c40 on the head), then calls _free(this) through 00d82463 when bit 0 of the byte a… |
| `00d7a7a0` | MarketingRecordMap32_DtorBody_B | likely | Byte-identical copy of MarketingRecordMap32_DtorBody (39 bytes): stores the record-map vftable at this+0 and loops calling 00d79c40 on the head until it is nul… |

**SoeUtil::HashListMap<int,InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a02d0` | HashListMapMerchantSellDtor | confirmed | Installs the typed HashListMap vtable and repeatedly removes head entries with FUN_0099C9A0 until empty. |
| `009a0400` | HashListMapMerchantSellScalarDtor | confirmed | Removes each merchant-sell entry with FUN_0099C9A0, then conditionally frees the map object for the deleting flag. |
| `009a0980` | HashListMapMerchantSellClear | confirmed | Installs the typed merchant-sell map vtable and repeatedly removes the current head entry with FUN_0099C9A0. |

**SoeUtil::HashListMap<int,InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a02a0` | HashListMapStoreBundlesDtor | confirmed | Installs the typed HashListMap vtable and repeatedly removes the head entry with FUN_0099C720 until the map is empty. |
| `009a03c0` | HashListMapStoreBundlesScalarDtor | confirmed | Removes all entries with FUN_0099C720, then conditionally frees the map object when the scalar-deleting flag is set. |
| `009a0ba0` | HashListMapStoreBundlesClear | confirmed | Installs the typed StoreBundles map vtable and repeatedly removes the current head entry with FUN_0099C720. |

**SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>_>,16,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b20f20` | CtorHashListMapIdEntryGroupId16 | confirmed | Constructor: stores the 16-bucket IdEntry<StoreBundleGroupId> map vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x40 bytes at +0x14 and zeroes +4; returns t… |

**SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>_>,32,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b20e20` | CtorHashListMapIdEntryBundleId32 | confirmed | Constructor: stores the 32-bucket IdEntry<StoreBundleId> map vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x80 bytes at +0x14 and zeroes +4; returns the ob… |

**SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>,16,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b21110` | CtorHashListMapIdEntryInt16 | confirmed | Constructor: stores the 16-bucket IdEntry<int> map vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x40 bytes at +0x14 and zeroes +4; returns the object. |

**SoeUtil::HashMap<int,ClientInGamePurchaseManager::CreditCardTypeHashEntry,24,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b207a0` | CtorHashMapCreditCardType24 | confirmed | Constructor: stores SoeUtil::HashMap<int,ClientInGamePurchaseManager::CreditCardTypeHashEntry,24,-1>::vftable at +0, zeroes the dword at +8, memsets 0x60 bytes… |

**SoeUtil::HashMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::OrderTracker*,8,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b31670` | OrderTrackerHashMap::ProcessEntries | confirmed | Walks linked entries, compares each key pair with global values, updates matching fields, checks related tracker state, removes selected hash-chain nodes, and … |

**SoeUtil::List<AppStoreBundleDefinition_const*,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a0300` | ListAppStoreBundleDtor | confirmed | Installs the typed list vtable and delegates to FUN_0099F8A0 to remove each linked-list node. |
| `009a0440` | ListAppStoreBundleScalarDtor | confirmed | Clears the typed bundle list, conditionally frees the list object, and returns its pointer. |

**SoeUtil::List<AppStoreBundleDefinition_const*,512>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a1b50` | ListAppStoreBundleCtor512 | confirmed | Initializes list pointers/counts to zero, installs the List<AppStoreBundleDefinition_const*,512> vtable, then calls 0099BD60 with ECX pointing to embedded list… |

**SoeUtil::List<InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode*,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a0350` | ListCategoryNodeClear | confirmed | Installs the category-node list vtable and clears linked entries through FUN_0099F980. |
| `009a0470` | ListCategoryNodeScalarDtor | confirmed | Clears the category-node list, conditionally frees the list object, and returns its original pointer. |

**SoeUtil::List<SaleDisplayInfo,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2aa90` | SaleDisplayList::Dtor | confirmed | Restores the SaleDisplayInfo list vtable and drains all remaining linked-list nodes. |
| `00b2aad0` | SaleDisplayList::ScalarDeletingDtor | confirmed | Restores the list vtable, drains its nodes, and conditionally frees the list object according to the deleting flag. |

**SoeUtil::List<bool_(__thiscall_ClientInGamePurchaseManager::*)(void),-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2aaa0` | PurchaseCallbackList::Dtor | confirmed | Restores the callback-list vtable and drains the linked list through its specialized removal helper. |
| `00b2ab00` | PurchaseCallbackList::ScalarDeletingDtor | confirmed | Restores the callback-list vtable, drains nodes, and conditionally frees the list object. |

**Uram::UploaderHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d67550` | UploaderHandler::Dtor | speculative | Stores the UploaderHandler vtable pointer into the object and returns. |
| `00d67560` | UploaderHandler::ScalarDeletingDtor | speculative | Stores the UploaderHandler vtable pointer and frees the object when the deleting flag is set. |

**(no class)** (255)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00910da0` | StorePreviewDispatch | confirmed | Looks up an event object with FUN_008fb2e0 (thiscall on global [01be1090], arg "BaseClient.InGamePurchase.StoreItemPreview"). If found: thiscall FUN_0099ff70 w… |
| `009a8f60` | InGamePurchase_RegisterStoreDataSources | confirmed | Initializes the Station-Cash store client object: allocates/constructs the first 0x30-byte data source (FUN_00997790), then for each remaining data source allo… |
| `00b20130` | LogClientInGamePurchaseSeverity5 | confirmed | Calls 00700f90 twice with the same message and varargs: severity 5 to "#ClientInGamePurchase.log", then severity 5 to "InGamePurchase.log". |
| `00b20160` | LogClientInGamePurchaseSeverity4 | confirmed | Calls 00700f90 twice with the same message and varargs: severity 4 to "#ClientInGamePurchase.log", then severity 5 to "InGamePurchase.log". |
| `00b201c0` | LogClientInGamePurchaseSeverity2 | confirmed | Calls 00700f90 twice with the same message and varargs: severity 2 to "#ClientInGamePurchase.log", then severity 5 to "InGamePurchase.log". |
| `00b23e80` | RequestCardTypes | confirmed | If request id 0xb is already pending, logs the duplicate-request message and returns 0. Otherwise calls FUN_00d571e0 to register or obtain a request id, logs i… |
| `00b27f80` | PreviewFundWalletWithCreditCard | confirmed | Checks marketplace enablement, validates the payment-source id against the 8-bucket table or fallback id, and checks the product index against the count and 0x… |
| `00b280a0` | RequestPaymentSources | confirmed | When all three enable bytes are set, checks whether request id 1 is pending; if not, sets the first list entry to 2, calls FUN_00d570e0 with an id and constant… |
| `00b300c0` | InitializeMarketplaceSupport | confirmed | Creates a fixed string, logs marketplace support initialization, calls service setup with InGamePurchase and the ServiceId UP0017-NPUA30048_00, reports success… |
| `00b33900` | PreviewSubscriptionPurchaseWithCreditCard | confirmed | Checks marketplace readiness and a payment-source ID, validates the product index, obtains a product value, builds a secure string, and calls FUN_00d572d0 with… |
| `00b33a50` | PreviewSubscriptionPurchaseWithStationCash | confirmed | Checks marketplace readiness and billing-address requirements, validates the product index, builds a secure string, and calls FUN_00d572d0 with station-cash pu… |
| `00b342f0` | RequestPurchaseSubscription | confirmed | Validates marketplace state, payment source and product index; refuses if request slot 8 is busy, serializes purchase details, logs sku/source/currency, emits … |
| `00b38ea0` | EquipStoreBundleInternal | confirmed | Checks equip eligibility and resolves bundle; collects owned item ids for bundle contents, applies items through equipment helper, then invokes OnEquipStoreBun… |
| `00b3a450` | ProcessPairKeyedTrackers | confirmed | Initializes an eight-bucket temporary map, derives a key by XORing two record fields, searches matching nodes, unlinks matching tracker nodes when present, and… |
| `00b3b2d0` | RequestAcceptedCreditCards | confirmed | When both manager state flags are set, requests card types if the accepted-card list is empty; otherwise refreshes the accepted-card data source and returns su… |
| `00b41fb0` | ClientInGamePurchaseManager::OnReceiveGiftOrder | confirmed | Constructs a gift-notification record, parses the supplied bounded payload and emits the OnReceiveGiftOrder callback on successful parsing; logs a serializatio… |
| `00c04110` | InGamePurchase::PreviewFundWalletWithCreditCard | confirmed | Validates two Lua values, logs detailed failure when manager or arguments are missing, otherwise calls 00b27f80, then pushes the result. |
| `00c042c0` | InGamePurchase::FundWalletWithCreditCard | confirmed | Validates four Lua parameters, converts two values, logs failures for missing manager or invalid converted parameters, otherwise calls 00b33760, then pushes it… |
| `00cc5470` | MapMarketplaceErrorToLocalizedText | confirmed | Maps status 2, 4, 6, and 7 to marketplace_error_order_failure, 5 to marketplace_error_balance_not_enough, 8 to marketplace_error_invalid_coupon, and other non-… |
| `00d57aa0` | InitCasAndUramPaymentApis | confirmed | Allocates a 9000-byte CAS object (FUN_00d833a6(9000)) and stores its initialised pointer at this+0x10 via FUN_00d76940; initialises it with FUN_00d74730 and on… |
| `00d57e30` | BuildFundWalletUrl | confirmed | If the byte at this+8 is non-zero, formats the fundWallet.action link with FUN_00402fd0 from the host argument and the theme argument, optionally appends a suf… |
| `00d57f80` | BuildMembershipUrl | confirmed | If the byte at this+8 is non-zero, formats the membership.action link with FUN_00402fd0 from the host, game code and theme arguments, optionally appends a suff… |
| `00d5e910` | SixStringsCtorEmpty | confirmed | Constructor body (ECX = object) for six IString members at +0, +0x10, +0x20, +0x30, +0x40 and +0x50: each gets the IString vtable, the shared empty buffer DAT_… |
| `00d5ea20` | SixStringsCtorFromArgs | confirmed | Constructor body (ECX = object; six C-string stack arguments): installs the IString vtable on six members at +0, +0x10, +0x20, +0x30, +0x40 and +0x50 with the … |
| `00d5eb30` | SixStringsCopyCtor | confirmed | Copy constructor body (ECX = new block, stack argument = source six-string block): installs the IString vtable on six members at +0..+0x50 with the shared empt… |
| `00d6b8d0` | FormatUramResultAt5C | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats each entry at the response's +… |
| `00d6ba00` | FormatUramResultAtA4 | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6bb30` | FormatUramResultAtCC | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6bc60` | FormatUramResultAt38 | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6bd90` | FormatUramResultAt28 | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6bec0` | FormatUramResultAt28Variant | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6bff0` | FormatUramResultAt38Variant | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6c120` | FormatUramResultAtDC | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6c250` | FormatUramResultAt98 | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6c380` | FormatUramResultAtAC | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6c4b0` | FormatUramResultAt3C | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6c5e0` | FormatUramResultAt28SecondVariant | confirmed | Clears the output string; returns 4 after logging a nonzero result code, returns 1 when status equals success, otherwise formats entries from response array +0… |
| `00d6d4b0` | ConfigureUramEndpoints | confirmed | When the initialized flag is clear, stores version and strings, formats the commerce and wallet URLs, chooses the version-11 route set only for param_5==0xb an… |
| `00d6d830` | DispatchUramRequestByType | confirmed | Switches on the request type at param_2+8 minus one for 13 operations, runs that operation's verification helper, then submits its URL and payload through a ty… |
| `00d6dd20` | AppendFormattedKeyValue | confirmed | Builds a temporary fixed-1024 string from param_3+4, selects a separator based on whether the destination is empty or ends in '?', then appends the %s%s=%s key… |
| `00d6de50` | SubmitPendingUramRequest | confirmed | If the manager has a pending request, obtains it, dispatches it when param_2 is nonzero, records two current-time values and links it on success; otherwise log… |
| `00d6e740` | AppendQueryPartUsingTempBuffer | confirmed | Creates a temporary fixed-1024 string, calls FUN_00d6dd20 with the supplied destination, key and value, then releases the temporary string storage. |
| `01078210` | InitFromArgsThreeIStringsAndArray | confirmed | Thiscall constructor with six stack arguments, under an EH frame. Stores argument 1 at +0 and argument 2 at +4; sets the IString vtable at +0x8 with data point… |
| `010787c0` | ReadPropertyFields | confirmed | Thiscall with one pointer argument (reader state: error byte at +0, mode byte at +2). Reads the named ints member_discount into this+0xf8, limited_series_id in… |
| `008fb2e0` | FindStoreBundlesByKey | likely | thiscall with one stack argument (the key string, ret 4). Computes the hash of the key with 00705bc0 (accepted label HashCString), indexes a table of 1000 buck… |
| `0099aa60` | StoreDataSourceSlot6Activate | likely | Virtual slot [6] shared by six store data-source vtables (StoreBundles, StoreBundleGroupContents, StoreBundleVipLevelContents, StoreItemOfTheDay, StoreBundleOf… |
| `009a6d80` | AdjustorThunk_add4_009a6d80 | likely | 8-byte adjustor thunk: raw bytes 83 c1 04 e9 a8 21 72 00 decode as add ecx,4; jmp FUN_010c8f30. FUN_010c8f30 returns 0 when *(ecx+4)==0 else *(*(ecx+4)+8); wit… |
| `009a6da0` | AdjustorThunk_sub254_009a6da0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 95 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7940, forwarding a secondary-interface this pointer bac… |
| `009a6e40` | AdjustorThunk_add4_009a6e40 | likely | 8-byte adjustor thunk: raw bytes 83 c1 04 e9 e8 20 72 00 decode as add ecx,4; jmp FUN_010c8f30 (accessor returning 0 or *(*(ecx+4)+8) as in the StorePortalCate… |
| `009a6e60` | AdjustorThunk_sub254_009a6e60 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 85 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a79f0 (scalar deleting destructor of StoreBillboards). |
| `009a6f00` | AdjustorThunk_add4_009a6f00 | likely | 8-byte adjustor thunk: raw bytes 83 c1 04 e9 28 20 72 00 decode as add ecx,4; jmp FUN_010c8f30 (shared accessor for the interface pointer at +4, same shape as … |
| `009a6f20` | AdjustorThunk_sub254_009a6f20 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 85 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7ab0 (scalar deleting destructor of StoreBillboardPanel… |
| `009a6fe0` | GetField_0x340 | likely | 7-byte accessor: mov eax,[ecx+0x340]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x340)). No callers recorded, so it is reached through a vftable slot. |
| `009a7000` | AdjustorThunk_sub254_009a7000 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 75 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7b80 (scalar deleting destructor of StoreBundleActiveFi… |
| `009a70f0` | AdjustorThunk_sub254_009a70f0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7c40 (scalar deleting destructor of StoreBundleActiveSo… |
| `009a7190` | GetField_0x22c_009a7190 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a71b0` | AdjustorThunk_sub254_009a71b0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7d00 (scalar deleting destructor of GiftOrderHistory). |
| `009a7250` | GetField_0x22c_009a7250 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a7270` | AdjustorThunk_sub254_009a7270 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7dc0 (scalar deleting destructor of GiftOrderEntries). |
| `009a7310` | GetField_0x22c_009a7310 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a7330` | AdjustorThunk_sub254_009a7330 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7e80 (scalar deleting destructor of CreditCardPayment). |
| `009a73d0` | GetField_0x22c_009a73d0 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a73f0` | AdjustorThunk_sub254_009a73f0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a7f40 (scalar deleting destructor of StationCashPriceLis… |
| `009a7490` | GetField_0x22c_009a7490 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a74b0` | AdjustorThunk_sub254_009a74b0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a8000 (scalar deleting destructor of SubscriptionPriceLi… |
| `009a7550` | GetField_0x22c_009a7550 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a7570` | AdjustorThunk_sub254_009a7570 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a80c0 (scalar deleting destructor of AcceptedCreditCardT… |
| `009a7610` | GetField_0x22c_009a7610 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a7630` | AdjustorThunk_sub254_009a7630 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a8180 (scalar deleting destructor of CountryCodes). |
| `009a76d0` | GetField_0x22c_009a76d0 | likely | 7-byte accessor: mov eax,[ecx+0x22c]; ret (pseudocode returns *(undefined4 *)(param_1 + 0x22c)). No callers recorded, so it is reached through a vftable slot. |
| `009a76f0` | AdjustorThunk_sub254_009a76f0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 45 0b 00 00 decode as sub ecx,0x254; jmp FUN_009a8240 (scalar deleting destructor of StateCodes). |
| `009a7730` | HashListRemoveKeyThenSync_009a7730 | likely | Calls FUN_009a6ba0(key, param_3) to remove key entries, then walks the bucket chain at this+0x12b4 (bucket = key & 7, node key field +0xd0, next +0xd4). If no … |
| `009a77b0` | ClearEntryList4aaAndNotify_009a77b0 | likely | Drains the node chain whose head link is this[0x4aa] (byte +0x12a8) by repeatedly calling FUN_009a6ac0, sets byte this[0x4a6] (byte +0x1298) to 1, then perform… |
| `009a7890` | DeletingDtor_009a7890 | likely | Scalar deleting destructor: calls FUN_009a6c30 (the destructor of the corresponding data-source class, defined just before this shard), and when bit 0 of param… |
| `009a7940` | DeletingDtor_009a7940 | likely | Scalar deleting destructor: calls FUN_009a6cf0 (StorePortalCategories::~StorePortalCategories), and when bit 0 of param_2 is set frees the object through the _… |
| `009a79f0` | DeletingDtor_009a79f0 | likely | Scalar deleting destructor: calls FUN_009a6db0 (StoreBillboards::~StoreBillboards), and when bit 0 of param_2 is set frees the object through the _free thunk; … |
| `009a7ab0` | DeletingDtor_009a7ab0 | likely | Scalar deleting destructor: calls FUN_009a6e70 (StoreBillboardPanels::~StoreBillboardPanels), and when bit 0 of param_2 is set frees the object through the _fr… |
| `009a7b80` | DeletingDtor_009a7b80 | likely | Scalar deleting destructor: calls FUN_009a6f30 (StoreBundleActiveFilters::~StoreBundleActiveFilters), and when bit 0 of param_2 is set frees the object through… |
| `009a7c40` | DeletingDtor_009a7c40 | likely | Scalar deleting destructor: calls FUN_009a7010 (StoreBundleActiveSort::~StoreBundleActiveSort), and when bit 0 of param_2 is set frees the object through the _… |
| `009a7d00` | DeletingDtor_009a7d00 | likely | Scalar deleting destructor: calls FUN_009a7100 (GiftOrderHistory::~GiftOrderHistory), and when bit 0 of param_2 is set frees the object through the _free thunk… |
| `009a7dc0` | DeletingDtor_009a7dc0 | likely | Scalar deleting destructor: calls FUN_009a71c0 (GiftOrderEntries::~GiftOrderEntries), and when bit 0 of param_2 is set frees the object through the _free thunk… |
| `009a7e80` | DeletingDtor_009a7e80 | likely | Scalar deleting destructor: calls FUN_009a7280 (CreditCardPayment::~CreditCardPayment), and when bit 0 of param_2 is set frees the object through the _free thu… |
| `009a7f40` | DeletingDtor_009a7f40 | likely | Scalar deleting destructor: calls FUN_009a7340 (StationCashPriceList::~StationCashPriceList), and when bit 0 of param_2 is set frees the object through the _fr… |
| `009a8000` | DeletingDtor_009a8000 | likely | Scalar deleting destructor: calls FUN_009a7400 (SubscriptionPriceList::~SubscriptionPriceList), and when bit 0 of param_2 is set frees the object through the _… |
| `009a80c0` | DeletingDtor_009a80c0 | likely | Scalar deleting destructor: calls FUN_009a74c0 (AcceptedCreditCardTypes::~AcceptedCreditCardTypes), and when bit 0 of param_2 is set frees the object through t… |
| `009a8180` | DeletingDtor_009a8180 | likely | Scalar deleting destructor: calls FUN_009a7580 (CountryCodes::~CountryCodes), and when bit 0 of param_2 is set frees the object through the _free thunk; return… |
| `009a8240` | DeletingDtor_009a8240 | likely | Scalar deleting destructor: calls FUN_009a7640 (StateCodes::~StateCodes), and when bit 0 of param_2 is set frees the object through the _free thunk; returns th… |
| `009a82d0` | Thunk_loadEcxPtr4_009a82d0 | likely | 8-byte thunk: raw bytes 8b 49 04 e9 58 f4 ff ff decode as mov ecx,[ecx+4]; jmp FUN_009a7730, re-targeting the call onto the store object stored at this+4 and t… |
| `009a82e0` | ResetEntriesAndNotify_009a82e0 | likely | Reads object P = *(this+4); drains the node chain whose head is P[0x4aa] (byte P+0x12a8) by repeatedly calling FUN_009a6ac0; sets byte P+0x1298 = 1; then perfo… |
| `009a84a0` | AdjustorThunk_sub270_009a84a0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 70 02 00 00 e9 05 05 00 00 decode as sub ecx,0x270; jmp FUN_009a89b0 (scalar deleting destructor of StoreBundleGroupCon… |
| `009a84b0` | AdjustorThunk_sub254_009a84b0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 f5 04 00 00 decode as sub ecx,0x254; jmp FUN_009a89b0 (scalar deleting destructor of StoreBundleGroupCon… |
| `009a8530` | AdjustorThunk_sub270_009a8530 | likely | 11-byte adjustor thunk: raw bytes 81 e9 70 02 00 00 e9 35 05 00 00 decode as sub ecx,0x270; jmp FUN_009a8a70 (scalar deleting destructor of StoreBundleGroupCon… |
| `009a8540` | AdjustorThunk_sub254_009a8540 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 25 05 00 00 decode as sub ecx,0x254; jmp FUN_009a8a70 (scalar deleting destructor of StoreBundleGroupCon… |
| `009a85c0` | AdjustorThunk_sub270_009a85c0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 70 02 00 00 e9 65 05 00 00 decode as sub ecx,0x270; jmp FUN_009a8b30 (scalar deleting destructor of StoreBundleVipLevel… |
| `009a85d0` | AdjustorThunk_sub254_009a85d0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 55 05 00 00 decode as sub ecx,0x254; jmp FUN_009a8b30 (scalar deleting destructor of StoreBundleVipLevel… |
| `009a86c0` | AdjustorThunk_sub270_009a86c0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 70 02 00 00 e9 25 05 00 00 decode as sub ecx,0x270; jmp FUN_009a8bf0 (scalar deleting destructor of StoreItemOfTheDay). |
| `009a87c0` | AdjustorThunk_sub270_009a87c0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 70 02 00 00 e9 e5 04 00 00 decode as sub ecx,0x270; jmp FUN_009a8cb0 (scalar deleting destructor of StoreBundleOfTheDay… |
| `009a89b0` | DeletingDtor_009a89b0 | likely | Scalar deleting destructor: calls FUN_009a83b0 (StoreBundles::~StoreBundles, the shared base destructor of the derived content classes), and when bit 0 of para… |
| `009a8a70` | DeletingDtor_009a8a70 | likely | Scalar deleting destructor: calls FUN_009a84c0 (StoreBundleGroupContents::~StoreBundleGroupContents), and when bit 0 of param_2 is set frees the object through… |
| `009a8b30` | DeletingDtor_009a8b30 | likely | Scalar deleting destructor: calls FUN_009a8550 (StoreBundleVipLevelContents::~StoreBundleVipLevelContents), and when bit 0 of param_2 is set frees the object t… |
| `009a8bf0` | DeletingDtor_009a8bf0 | likely | Scalar deleting destructor: calls FUN_009a85e0 (StoreItemOfTheDay::~StoreItemOfTheDay), and when bit 0 of param_2 is set frees the object through the _free thu… |
| `009a8cb0` | DeletingDtor_009a8cb0 | likely | Scalar deleting destructor: calls FUN_009a86e0 (StoreBundleOfTheDay::~StoreBundleOfTheDay), and when bit 0 of param_2 is set frees the object through the _free… |
| `009a8da0` | AdjustorThunk_sub270_009a8da0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 70 02 00 00 e9 a5 00 00 00 decode as sub ecx,0x270; jmp FUN_009a8e50 (scalar deleting destructor of MerchantSellBundles… |
| `009a8db0` | AdjustorThunk_sub254_009a8db0 | likely | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 95 00 00 00 decode as sub ecx,0x254; jmp FUN_009a8e50 (scalar deleting destructor of MerchantSellBundles… |
| `009a8e50` | DeletingDtor_009a8e50 | likely | Scalar deleting destructor: calls FUN_009a8dc0 (MerchantSellBundles::~MerchantSellBundles), and when bit 0 of param_2 is set frees the object through the _free… |
| `009a8e70` | InGamePurchaseStore_FindOrCreateBundle | likely | Find-or-create of a store data-source entry: looks up param_2 via FUN_008fb2e0; if an entry exists it calls FUN_0099ff70 and returns it. Otherwise allocates 0x… |
| `00b1f880` | MatchKeyAndOptionalTag | likely | Compares two records: returns 1 when the dword at +4 matches and either the second record's +8 field is -1 or both +8 fields are equal, otherwise 0. No calls o… |
| `00b20ef0` | StampEntryTime_00b20ef0 | likely | Calls FUN_00702670 into a local 8-byte buffer (the cached time pair) and stores both dwords into the object's +0x10 and +0x14 fields; returns nothing. |
| `00b20ff0` | StampEntryTime_00b20ff0 | likely | Same as 00b20ef0: stores the current time pair from FUN_00702670 into the object's +0x10 and +0x14 dwords. |
| `00b21e90` | ClearHashMap8Buckets | likely | Empties an 8-bucket hash container: walks the bucket array at +0xc, and for each non-empty chain detaches the head, frees it through virtual slot 2 of *param_1… |
| `00b21f20` | ClearHashMapTransactionTracker8Buckets | likely | Identical body to 00b21e90: walks 8 buckets at +0xc, detaches each chain head, frees nodes through virtual slot 2 and decrements the count at +8 until the cont… |
| `00b22110` | EhFramedCall_00b22110 | likely | Builds an MSVC SEH registration frame (push -1, handler 015d2678, fs:[0] chain, cookie xor esp), saves the incoming ECX into the frame slot, calls FUN_00d58300… |
| `00b222a0` | EhFramedCall_00b222a0 | likely | Builds an MSVC SEH frame (push -1, handler 015d2768, fs:[0] chain, cookie xor esp), saves ECX into the frame slot, calls FUN_01078650 with that ECX (its body w… |
| `00b22800` | SehInitThenSetField1c | likely | Inside an SEH frame, calls FUN_00d5c2c0 with this (that callee writes the GameCommerce::StoreBundleCategoryGroupDefinition and SoeUtil::Set vtables, i.e. const… |
| `00b22860` | SehInitThenSetField128 | likely | Inside an SEH frame, calls FUN_01078450 with this (its body references AppStoreBundleDefinition::vftable), then stores *param_2 into this+0x128 and returns thi… |
| `00b260d0` | EhFramedCall_00b260d0 | likely | MSVC EH helper body: sets up an SEH frame (handler 015d32b8, try level -1) and calls FUN_00d5cab0 with ECX unchanged (that callee writes GameCommerce::StoreBun… |
| `00b27290` | InitThenSetField78 | likely | Inside an SEH frame, calls FUN_00d58be0 with this (its body references GameCommerce::StoreBundleGroupDefinition::vftable and the group HashListMap) and then st… |
| `00b27740` | SehInitThenSetField128Variant | likely | Second copy of the initialiser shape at 00b22860: inside an SEH frame calls FUN_01078450 with this (body referencing AppStoreBundleDefinition::vftable) then st… |
| `00b279d0` | InitThenZeroFields30to48 | likely | Inside an SEH frame calls FUN_00d5cb70 with this (its body writes GameCommerce::StoreBundleCategoryDefinition::vftable), seeds this+0x30 from the global DAT_01… |
| `00b27a50` | InitEntryThenSetField14 | likely | Inside an SEH frame calls FUN_00d58400 with this (its body references GameCommerce::StoreBundleGroupDefinition::Entry::vftable) then stores *param_2 at this+0x… |
| `00b28150` | HashStringLookup32Buckets | likely | Looks up a string key: computes FUN_00705b00(param_2) (string hash), walks the 32-bucket table at this+0x3b8f8 (hash & 0x1f) with chain links at node+0x90 comp… |
| `00b28360` | NextIdSkippingCurrent | likely | Generates the next request id: lazily seeds the global counters (DAT_01be95e4 bit 0, DAT_01be95e0 = 0), increments DAT_01be95e0 until it differs from DAT_01be9… |
| `00b28940` | IterateList120Call_00c40360 | likely | Walks from the node pointer at this+8 through each node?s +0x120 link. For each node, restores the incoming manager in ECX and calls FUN_00c40360 with stack ar… |
| `00b289d0` | HashListRemoveAppStoreBundleNode | likely | Removes a bundle record: unlinks the node from its hash chain via FUN_00b25a00 (2048-bucket table on field +0x128), splices it out of the doubly linked list (n… |
| `00b28d20` | HashListRemoveStoreBundleGroupNode | likely | Removes a store bundle group record: bucket = (*(uint*)(param_2+0x78) & 0xf) with the table at this+0x14 and chain link node+0x7c; splices the node out of the … |
| `00b28dd0` | HashListRemoveStoreBundleCategoryGroupNode | likely | Removal counterpart with bucket = (*(uint*)(param_2+0x1c) & 0x3f), table at this+0x14, chain node+0x20, list links +0x14/+0x18 (head this+8, tail this+0xc): sp… |
| `00b29090` | EhFramedCall_00b29090 | likely | MSVC EH helper body: sets up an SEH frame (handler 015d39d0, try level -1) and calls FUN_00d5cab0 with ECX unchanged (writes GameCommerce::StoreBundleCategoryD… |
| `00b290e0` | HashListRemoveBaseDefinitionNode | likely | Removes a node from an 8-bucket hash chain (bucket = *(uint*)(param_2+0x14) & 7, table this+0x14, chain node+0x18) and from the list with links +0xc/+0x10 (hea… |
| `00b29540` | HashListRemoveAppStoreBundleDefinitionNode | likely | Removes a bundle definition node: bucket = (*(uint*)(param_2+0x128) & 0x1f) with the chain link at node+300, table this+0x14; splices the list whose links are … |
| `00b30200` | FundWalletFromSteam | likely | Rejects disabled marketplace, invalid product indexes, and pending requests; otherwise reads product data, formats a Steam ID, logs SKU and currency, and retur… |
| `00b335f0` | UpdateMarketplaceCallbacks | likely | Initializes callback state once, checks marketplace-related state and elapsed time, optionally removes the list head, logs OnInitializeSearchEngine, invokes a … |
| `00b33760` | FundWalletWithCreditCard | likely | Rejects disabled marketplace, unknown payment source, invalid product index, and pending request; otherwise builds secure strings, calls the funding helper wit… |
| `00b33bc0` | RequestAddCreditCard | likely | Checks manager readiness and card-type bounds, refuses while request slot 2 is pending, builds/submits the add-card request, logs its id, and clears the pendin… |
| `00b33e60` | RequestUpdateCreditCard | likely | Checks readiness, finds payment source by id, refuses while update slot 3 is pending, builds/submits update request, and logs its id. |
| `00b340f0` | RequestDeleteCreditCard | likely | Looks up payment source, rejects missing or non-deletable entries, checks pending slot 4, then builds/submits deletion and logs the request id. |
| `00b3b320` | NotifyStoreBundlesMissingFromIndex | likely | On the nonzero flag path, clears per-entry state, walks the store records, checks each bundle key in a tree, notifies for entries missing from that tree, and s… |
| `00b3c1b0` | RemoveCurrentSaleTypeEntries | likely | Reads the active sale type, walks the sale list, removes matching entries from the hash map and doubly linked list, decrements the list count and destroys each… |
| `00b3c480` | ResetStoreSessionCaches | likely | Refreshes timestamp fields, clears multiple hashed and linked collections, then stores the supplied state byte and two supplied dwords in the object header. |
| `00b3c610` | ResetTransactionTrackerCollections | likely | Stores the current timestamp pair, processes the tracker map at +0x2b0, resets four tracker collections through the pair-keyed helper, and clears the state byt… |
| `00c34100` | GetFieldAt50 | likely | Loads and returns the dword at param_1+0x50; x86 bytes show a callee-cleaned four-byte argument. |
| `00d5e670` | StringMembersDtorCleanup0to50 | likely | Releases six IString members of the object at ECX, in order from the last to the first: for each one it installs the IString vtable, and when its count is posi… |
| `00d5e780` | SecureStringsDtorCleanup | likely | Unwind cleanup of a request object (ECX): releases the IString members at +0xf0 and +0xd8 (vtable installed, buffer released through the string vtable slot 2 w… |
| `00d5e8c0` | FormatMaskedLastTwoChars | likely | Writes the mask '****' into the string object at ECX through 00402ce0 (IStringAssign), then, when the C string at this+8 is longer than one character, appends … |
| `00d5ec40` | SixStringsAssignFrom | likely | Copy-assigns six strings: for each offset 0, 0x10, 0x20, 0x30, 0x40 and 0x50 it assigns the source string at param_2 + offset to the member at ECX + offset thr… |
| `00d5ed80` | AddCreditCardRecordCtor | likely | Thiscall ctor, ret 0x2c (11 args). Constructs StringSecure<24> at +0x00 from arg1 (00b30ad0) and StringSecure<8> at +0x30 from arg2 (00b30b40); assigns arg3 an… |
| `00d60070` | AddSubscriber | likely | Thiscall (ECX=updater) with arg1 = subscriber node (key C string at node+0x14) and arg2 = 1-based position. If the key is not already present (00d5f4d0), compu… |
| `00d60300` | GameCommerce::MarketingDataUpdater::EnableAutomaticUpdates | likely | Thiscall (ECX=this) with arg1 = enabled byte and arg2 = refresh hour dword. Logs 'EnableAutomaticUpdates - enabled=%d, dataRefreshHour=%02d:00GMT.' with both a… |
| `00d61370` | NamedRecordFind16 | likely | Thiscall (ECX=table) with arg1 = C string key. Computes the one-at-a-time style hash of the key inline (h = (c + h) * 0x401; h ^= h >> 6, then the final mix h … |
| `00d637a0` | ReadShortcutIntegerPair | likely | Looks up a record, requires type 1 and a nonnull row-data pointer, then reads adjacent 32-bit words at index*8 and returns them as a 64-bit value; otherwise re… |
| `00d63820` | ReadImageRecordPointer | likely | Looks up a record, requires type 3 and a nonnull row-data pointer, then reads the pointer at 4+index*0x34; otherwise returns DAT_0175b400. It does not check th… |
| `00d65680` | SetIntegerRecordValue | likely | Looks up the keyed record, requires type 1 and a nonnull row-data pointer; uses the existing slot when index is below length, otherwise grows if capacity is be… |
| `00d656f0` | SetDoubleRecordValue | likely | Looks up the keyed record, requires type 2 and a nonnull row-data pointer; uses the existing slot when index is below length, otherwise grows if capacity is be… |
| `00d66d90` | LoadMarketingRecordSetFromFile | likely | Formats a record-set text path, loads its contents, parses rows and typed fields, updates source record values, logs read status and row count, and compares de… |
| `00d681f0` | IncrementCounterSkippingValue | likely | Increments the integer at this+0x24 until it differs from the global sentinel, then writes the resulting value to the output pointer. |
| `00d69300` | DrainListHeadAndReleaseEntries | likely | Repeatedly removes the current list head, optionally invokes its virtual slot +4 with value 3, updates the matching key-bucket reference count, and invokes vir… |
| `00d693d0` | DrainSecondaryListHeadAndReleaseEntries | likely | Starts at the secondary list head, saves each next entry, optionally invokes virtual slot +4 with value 3, unlinks through 00d69210, decrements the matching ke… |
| `00d6e6b0` | AppendRequestAndTypeCount | likely | Appends the node to the manager list, increments its keyed type count, calls the request object's virtual method at +0x34, and on false calls FUN_00d691b0 and … |
| `00d74c70` | CasUrlAppendQueryParam | likely | Percent-encodes the string at argument +4 through 00fc9160 into a local buffer, then appends '<sep><name>=<encoded>' to the URL string at its first argument vi… |
| `00d74da0` | HashListInsertRequestTracker16 | likely | Allocates a node through the list vtable slot 1 (+4) and, when that succeeds, constructs it with 00d743a0 and stores the dword at *argument2 into node+0x830 (t… |
| `00d74f20` | CasUrlAppendQueryParamFromString1024 | likely | Constructs a temporary SoeUtil StringFixed<1024> from the third argument through 00aaf4c0, appends it to the URL in the first argument as query parameter named… |
| `00d75010` | HashListRemoveRequestTracker16 | likely | Unlinks the node at argument2 from the 16-bucket list at ECX: walks the bucket at (node+0x830 & 0xf)+5 through node+0x834 until it finds the node, splices it o… |
| `00d75800` | HashListExpireRequestTrackers | likely | Walks the tracker list from the head at ECX+8 while each node's elapsed time (now minus node+8/+0xc, via 007027f0) is below 0xea61 ms, stopping at the first yo… |
| `00d75890` | HashListClearRequestTrackers | likely | Repeatedly removes the head node of the tracker list at ECX (head at +8) through 00d75010 until the head is null. It does not call the node callbacks. |
| `00d772d0` | AssignOrderStringFields | likely | Copies the order string fields from the source object at argument1 into ECX: it assigns the strings at source +4, +0x38, +0x70, +0x8c and +0xcc through 006f9e6… |
| `00d79380` | MarketingElementOwner_InitWithKey | likely | Assigns three C-string arguments into StringFixed<32> fields (through 0070ac50 and 00402ce0), installs StringFixed<32> vftables at +0, +0x34, +0x68 and +0x9c, … |
| `00d79b50` | MarketingRecordSet_ScalarDeletingDtor | likely | Calls 00d79110 (the non-deleting body that releases the owner's containers and 16-entry pointer array), then calls _free(this) only when bit 0 of the flag argu… |
| `00d79b70` | MarketingRecordMap32_InsertNew | likely | Allocates a record through the map's vtable slot +4 (slot 1); on success calls 00d79380 with the three string arguments and stores the int key *param_2 at reco… |
| `00d79c40` | MarketingRecordMap32_UnlinkAndDestroy | likely | Unlinks record param_2 from bucket (record key at +0x188 & 0x1f) and from the list (+0x180 next, +0x184 prev, head at map+8, tail at map+0xc), decrements the c… |
| `00d79ce0` | SchemaNode_ParseRecordSetEntry | likely | Reads the attributes dbname/dbName, outputfile/outputFile and noOutputfile through 0073a200 and copies their text into StringFixed<64> locals; hashes the dbnam… |
| `00d7a680` | MarketingRecordMap32_RemoveAll | likely | Thiscall (ECX = map). While the head at map+8 is non-null, pushes it and calls 00d79c40 to unlink and destroy it. Returns nothing. |
| `00d7a6a0` | OwnerClearRecordsAndStrings | likely | Thiscall (ECX = owner). Destroys every entry of the list at owner+0x150 by calling 00d79c40 for each head. Then releases the string buffer at owner+4 (refcount… |
| `00d7a7d0` | MarketingRecordSetOwner_DtorBody | likely | Called with the owner in ECX; calls 00d7a6a0 to clear the owner's containers, stores the record-map vftable at owner+0x148 (param_1[0x52]) and destroys the lis… |
| `00d7abd0` | MarketingRecordSetOwner_Construct | likely | Fastcall (ECX = owner). Installs the StringFixed<256> vftable at +0 and the StringFixed<32> vftable at +0x114, points the string buffers at the empty string DA… |
| `01077f70` | InitThreeIStringsAndArray | likely | Fastcall with this in ECX: stores DAT_01cb78c0 and DAT_01cb78c4 in the first two dwords; sets three SoeUtil::IString<char> members at +0x08, +0x18 and +0x28 (v… |
| `01078020` | DestructThreeIStringsAndArray | likely | Fastcall with this in ECX, under an EH frame: stores the array vtable at +0x38, sets the array count at +0x40 to 0, frees the buffer at +0x3c through 00401c00(… |
| `010783e0` | CopyAssignDwordsIStringsArray | likely | Thiscall with one stack argument (source object), ret 4. If source differs from this: copies the dwords at +0 and +4; copy-assigns the three IString members at… |
| `010785c0` | CopyAssignFieldsF8 | likely | Thiscall with one stack argument (source pointer; ret 4), returns this. Calls 0110ab90(source) on this, then copies the dwords at +0xf8, +0xfc, +0x104, +0x10c,… |
| `0099d6f0` | FindStoreEntryById | speculative | Selects a 16-bucket slot from the supplied integer key, follows collision links, and returns the matching entry pointer or null. |
| `0099da70` | CopyBundleDefinitionFields | speculative | Copies offsets 4, 8, 0xC, 0x10, and 0x20 through 0x34 from the source object, then calls FUN_0099C2B0(1). |
| `0099f000` | ContainsStoreBundleId | speculative | Finds a store object by a 4-bit bucket and full key, then searches its nested 8-bucket list for a second identifier and returns whether it exists. |
| `0099f150` | EvaluateBundleRequirements | speculative | Locates a store entry by id, checks candidate bundle ancestry and several per-entry flags, then compares nested requirement records through a callback. |
| `0099f3b0` | HasBundleAncestor | speculative | Looks up a store graph by id, starts at its leftmost entry, and returns true when any entry is an ancestor or equal to the global selected key. |
| `009a87d0` | AdjustorThunk_sub254_009a87d0 | speculative | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 d5 04 00 00 decode as sub ecx,0x254; jmp FUN_009a8cb0 (scalar deleting destructor of StoreBundleOfTheDay… |
| `00b210e0` | StampEntryTime_00b210e0 | speculative | Same as the other two copies: writes the FUN_00702670 time pair into the object's +0x10/+0x14. |
| `00b25900` | FormatBuffer2048Then_00b219b0 | speculative | Builds text in a 2048-byte stack buffer via FUN_006fba40(param_1, buf, 0x800, param_3, 1) (that callee references the literals space/digit, so it is a formatte… |
| `00b2c790` | DrainStoreBundleGroupHashList | speculative | Repeatedly removes the hash-list node at container offset +8 until the list is empty. |
| `00b2c7b0` | DrainStoreBundleCategoryGroupHashList | speculative | Repeatedly removes the hash-list node at container offset +8 until the list is empty. |
| `00b2cdb0` | DrainBaseDefinitionHashList | speculative | Repeatedly removes the node at the hash-list head field until empty. |
| `00b2cf20` | DrainStoreBundleIdMap | speculative | Repeatedly removes hash-list entries from the map until its head pointer is null. |
| `00b2cf40` | DrainStoreBundleGroupIdMap | speculative | Repeatedly removes hash-list entries from the map until its head pointer is null. |
| `00b2d020` | DrainAppStoreBundleDefinitionMap | speculative | Repeatedly removes AppStoreBundleDefinition nodes at the hash-list head until the map is empty. |
| `00b2d710` | InsertAppStoreBundleDefinition | speculative | Appends a bundle definition to the doubly linked list, inserts it into a 2048-bucket hash chain keyed by field +0x128, and increments count. |
| `00b2d760` | RemoveStoreBundleGroupDefinition | speculative | Finds the group in a 128-bucket chain, unlinks it from hash and doubly linked list, decrements count, destroys the node, and invokes the container's virtual fr… |
| `00b2e780` | InsertStoreBundleGroupDefinition | speculative | Allocates and initializes a store-bundle-group definition, appends it to a linked list, hashes its key into a 16-bucket table, and increments count. |
| `00b2e820` | InsertAppStoreBundleDefinition | speculative | Allocates and initializes an app-store bundle definition, appends it to a linked list, hashes its key into a 32-bucket table, and increments count. |
| `00b2e9b0` | InsertStoreBundleCategoryGroupDefinition | speculative | Allocates and initializes a category-group definition, appends it to a linked list, hashes the key into a 64-bucket table, and increments count. |
| `00b344f0` | LinkStoreRefreshNode | speculative | Allocates/constructs a node, stores old tail as predecessor, links at list tail or sets head, increments count, and updates tail. |
| `00b36bb0` | ReadStoreBundleEntry | speculative | Reads three dwords, nested value, byte boolean and final dword from bounded stream into entry; truncation zeros fields and sets stream error flag. |
| `00b37670` | RequestSubscriptionUsingSelectedSource | speculative | Forwards to subscription request routine, supplying selected payment-source id from manager offset 0x3b3f8. |
| `00b376d0` | PreparePaymentRequestState | speculative | Calls request object virtual operation, gets selected datasource value, formats manager strings, assigns supplied value, allocates/stores next request id, retu… |
| `00b37bb0` | InitializeInGamePurchaseManagerCaches | speculative | When manager offset 0x2d0 is zero, constructs store-bundle-id map, initializes item cache, copies 32-byte value and one flag into manager, invokes supplied ope… |
| `00b39af0` | AppendStoreRecordCacheNode | speculative | Allocates node through virtual slot 2, links it after current tail using offsets +0x60/+0x64, increments list count and returns. |
| `00c03ba0` | CheckItemRecordValidity | speculative | Passes the dword at param_1+8 and the dword at param_1+0x3c to 006fc5b0, returning whether its result is nonzero. |
| `00c044e0` | FormatPurchaseLogLine | speculative | Formats the value at param_2 through the format string at 01770978 into the output string at this+0x14. |
| `00d583e0` | PredicateNotSentinelAt4NonNegAt8 | speculative | Bytes a1 c8 2c bf 01 3b 41 04 74 0c 83 79 08 00 7c 06 b8 01 00 00 00 c3 33 c0 c3: returns 1 when the global DAT_01bf2cc8 differs from the dword at this+4 and t… |
| `00d58bc0` | PredicateNonSentinelAt4Nonnull10Pos4c | speculative | Bytes a1 cc 2c bf 01 3b 41 04 74 12 83 79 10 00 74 0c 83 79 4c 00 7e 06 b8 01 00 00 00 c3 33 c0 c3: returns 1 when DAT_01bf2ccc differs from the dword at +4, t… |
| `00d58c80` | CopyFieldsAndEntries | speculative | Copies the dwords at +4 through +0x10 and the byte at +0x38 from the source object (param_2) into this, runs FUN_00d67330 on the source's sub-object at +0x14, … |
| `00d5ed00` | StringPlus4ResetWritableAssign | speculative | Resets the string member at this+4 with 00b2ac60 (ClearStringBuffer, accepted speculative label), then assigns the C-string argument to that member through 004… |
| `00d5ed40` | StringPlus4ResetEmptyAssign | speculative | Resets the string member at this+4 with 00b2ad40 (ClearStringBuffer, accepted speculative label; restores the empty buffer state), then assigns the C-string ar… |
| `00d5eeb0` | DefaultRecordCtor | speculative | Fastcall ctor (ECX=this). Constructs StringSecure<24> at +0x00 and StringSecure<8> at +0x30 from the static empty string DAT_0175b400; installs IString vtable … |
| `00d5efc0` | RecordSetFieldsFromArgs | speculative | Thiscall setter, ret 0x2c (11 args). Clears the secure string at +0x00 (00b2ac60) and assigns arg1 into the string at +0x04; clears +0x30 (00b2ad40) and assign… |
| `00d5f070` | RecordCopyFieldsFrom | speculative | Thiscall copy from one arg1 record pointer, ret 4. Clears this+0x00 (00b2ac60) and assigns source+0x08 into this+0x04; clears this+0x30 (00b2ad40) and assigns … |
| `00d5f4d0` | StringKeyBucketFind16 | speculative | Thiscall (ECX=this) with arg1 = C string key. Hashes the key with 00705bc0, selects bucket (hash and 0xf) from the table of 16 heads at this+0x14, walks the ch… |
| `00d5fb60` | ResetLoadedData | speculative | Fastcall (ECX=this). Walks the list at this+0x08 following the next link at node+0x10 and calls each node's vtable slot [0] with (this, owned object at this+0x… |
| `00d61950` | NamedRecordFind32At15c | speculative | Thiscall with arg1 = C string key. Same inline hash as 00d61370; selects bucket (hash and 0x1f) from the 32 heads at +0x15c; walks the chain comparing the stor… |
| `00d61f50` | NamedRecordFind32At1c | speculative | Thiscall with arg1 = C string key. Same inline hash as 00d61370; selects bucket (hash and 0x1f) from the 32 heads at +0x1c; walks the chain comparing the store… |
| `00d63750` | ReadXorWordPair | speculative | Validates a selected row type, reads two adjacent 32-bit words at the requested index, and returns their XOR; otherwise returns zero. |
| `00d63860` | AllRecordRowsMatch | speculative | Walks records in the selected set, calls each element's virtual slot 2, and returns false if successive results differ; empty sets return true. |
| `00d65c20` | CallD62d50ReturnArg | speculative | Calls 00d62d50 with the incoming argument, then returns that same argument unchanged. |
| `00d65c70` | CreateInt64MarketingElement | speculative | Uses the incoming ECX pool object in 00d65900 to pop one node; if nonnull, calls 00d64dc0 with the node as ECX and returns that result, otherwise returns null. |
| `00d65d00` | CallD62d50ReturnArg | speculative | Calls 00d62d50 with the incoming argument and returns that argument unchanged. |
| `00d65d50` | CreateDoubleMarketingElement | speculative | Uses the incoming ECX pool object in 00d65900 to pop one node; if nonnull, calls 00d64f90 with the node as ECX and returns that result, otherwise returns null. |
| `00d65de0` | CallD62db0ReturnArg | speculative | Calls 00d62db0 with the incoming argument and returns that argument unchanged. |
| `00d65e30` | CreateStringMarketingElement | speculative | Uses the incoming ECX pool object in 00d65a20 to pop one node; if nonnull, calls 00d65160 with the node as ECX and returns that result, otherwise returns null. |
| `00d66860` | SetStringRecordValue | speculative | Finds a type-3 record for the key, builds a temporary StringFixed<32> from the supplied string, stores it at the requested row, and releases temporary state. |
| `00d669b0` | ParseDoubleText | speculative | Reads a bounded text token into a 2048-byte stack buffer, converts it with atof, stores the double result, and checks the stack cookie. |
| `00d66d00` | ParseStringText | speculative | Reads a bounded text token into a 2048-byte stack buffer, assigns it to the supplied string object, and checks the stack cookie. |
| `00d77440` | ResetThenAssignThreeStrings | speculative | Calls the object's virtual slot 0 (through the first dword of ECX), then assigns three C-string arguments into string members through 00402ce0 (IStringAssign). |
| `00d776d0` | StoreDwordAt4 | speculative | Stores the dword argument2 at this+4 (ECX is the object) and returns. |
| `00d776e0` | ForwardToSinkIfSet | speculative | If the dword at this+4 is non-null, calls its vtable slot 0 with the object pointer, argument2 and argument3 pushed; otherwise it does nothing. |
| `00d77ee0` | RecordSetChainValidate | speculative | Walks the chain of record-set entries from the head at argument1+0x150 while the count at +0x158 is positive. For each entry it stops with 1 when the next link… |
| `00d79a60` | MarketingElementOwner_DefaultInit | speculative | Installs StringFixed<32> vftables at +0, +0x34, +0x68 and +0x9c with each buffer pointing at the shared empty string DAT_01bf3220, zeroes +0xd0, +0xdc, +0xe0, … |
| `009a86d0` | — | unknown | 11-byte adjustor thunk: raw bytes 81 e9 54 02 00 00 e9 15 05 00 00 decode as sub ecx,0x254; jmp FUN_009a8bf0 (scalar deleting destructor of StoreItemOfTheDay). |
| `00b1ffe0` | — | unknown | Calls FUN_009a4dc0 to tear down a member, then frees param_1 through the _free thunk when bit 0 of param_2 is set; otherwise returns param_1. |
| `00b21510` | — | unknown | SEH-framed wrapper calls FUN_01078650 and restores ExceptionList; callee is the AppStoreBundleDefinition destructor body, not a constructor. |
| `00b21ae0` | — | unknown | SEH-framed wrapper calls FUN_00d58cf0 and restores ExceptionList. The callee writes StoreBundleGroupDefinition and BaseDefinition vtables while tearing down th… |
| `00b21b30` | — | unknown | SEH-framed wrapper calls FUN_00d5c330 and restores ExceptionList. The callee writes StoreBundleCategoryGroupDefinition and BaseDefinition vtables while clearin… |
| `00b23be0` | — | unknown | If this+8 is non-null, destroys the container at that address (calls FUN_009a4dc0, whose body references the SoeUtil::List<DataSourceRef<InGamePurchaseDataSour… |
| `00b2c340` | — | unknown | Walks a linked object chain, selects entries whose field at +0x2806c matches a global value, grows an integer array, and appends each selected object's field +… |
| `00b30550` | — | unknown | Checks that the supplied message is nonempty, creates a fixed-size string, prefixes/formats it using the HandlerInGamePurchase handler label, and invokes the l… |
| `00b34e60` | — | unknown | Allocates a node via virtual slot 1, links it after tail, inserts into bucket selected by key modulo 24, increments count, and returns payload. |
| `00b358c0` | — | unknown | Unlinks node from doubly linked list, repairs head/tail and neighbour links, clears link fields, decrements count, cleans payload and frees node. |
| `00b376a0` | — | unknown | Forwards to subscription request routine using manager wallet source id/product and fixed empty currency strings. |
| `00b3b3b0` | — | unknown | Searches the bundle-id tree; if the bundle is present, calls the internal equip routine with the store id, bundle id, and supplied third value, otherwise logs … |
| `00c023f0` | — | unknown | When exactly one Lua argument exists and is truthy, obtains the manager, then pushes zero; returns one in all cases. |
| `00d638b0` | — | unknown | Starts from the source list head, follows each node's +0x180 link, and invokes the supplied callback on every node; returns the last callback result. |
| `00d65760` | — | unknown | Checks a supplied key through a predicate helper and, when it passes, forwards the row index and two-word integer value to the integer setter. |
| `00d65790` | — | unknown | Checks a supplied key through a predicate helper and, when it passes, forwards the row index and 64-bit value to the double setter. |
| `00d66920` | — | unknown | Tests the supplied key with a predicate helper and, if accepted, forwards key, row index, and string value to the string-record setter. |
| `00d69450` | — | unknown | Calls the availability gate; when nonzero, constructs and returns a UramPaymentSourceRequest, otherwise returns zero. |
| `00d6e830` | — | unknown | Appends theme, version, orderOrigin and locale; conditionally appends platformOob/gameOob; then appends deadEnd or service fields and optionally a final string. |
| `00d758b0` | — | unknown | Reads the createsessionresponse section of a parsed response document through 01017500, then looks up status (01016fb0 or 01017650, chosen by the byte at argum… |
| `00d75a50` | — | unknown | Walks the tracker list from the head at ECX+8. For each node it calls 00f5fc30 with the tracking id at node+4, then calls the node's callback object vtable slo… |
| `00d78d70` | — | unknown | Reads a named field definition from a parsed document node: it looks up the 'type' text in the table at 0x17eda5c and the name text at 0x17ee664 through 0073a2… |
| `00d7a8e0` | — | unknown | Thiscall with a schema file path argument. Reads the whole file through 00702440 into a StringFixed<8192> buffer (on failure calls 00402ec0 and exits), clears … |

### `net.coinstore`

**BaseCoinStorePacket** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6e9d0` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable into the object. No free and no callees; 73-byte base teardown body. |
| `00c6ea20` | BaseCoinStorePacket::ScalarDeletingDtor | confirmed | Reinstalls BaseCoinStorePacket then BasePacket vtables, runs the base body, and when bit 0 of the second argument is set calls _free(this) through thunk 00d824… |
| `00c6ed50` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable into the object. No free and no callees; 76-byte base teardown body. |
| `00c6ee10` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable. Identical body to 00c6e9d0 (76 bytes), no free. |
| `00c6eff0` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c6f0a0` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c6f0f0` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c6f140` | BaseCoinStorePacket::BaseDtor | confirmed | Stores BaseCoinStorePacket::vftable then BasePacket::vftable; no free and no callees (76 bytes). |
| `00c74430` | BaseCoinStorePacket::DtorBody | confirmed | Destructor body (second copy, same as 00c74360): calls FUN_00c00d50 on the object, then resets the vtable to BaseCoinStorePacket and finally to BasePacket. |
| `00c6e960` | — 🌐 | unknown | Installs BasePacket::vftable, writes family opcode 0xa5 (165) to word [1], installs BaseCoinStorePacket::vftable, then stores the sub-opcode argument in word [… |
| `00c74360` | — | unknown | Destructor body: calls FUN_00c00d50 on the object, then resets the vtable to BaseCoinStorePacket and finally to BasePacket (the base-class vtable is the last s… |

**ClientInGamePurchaseManager** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b45660` | HandleStationCashSkuListResponse 🌐 | confirmed | Handles InGamePurchaseStationCashSkuListResponsePacket (dispatched for sub-opcode 0xf): resets the client's SKU array at [this+0x3b5d4] (count [this+0x3b5d8]),… |
| `00b45ba0` | HandleSubscriptionSkuListResponse 🌐 | confirmed | Handles InGamePurchaseSubscriptionSkuListResponsePacket (sub-opcode 0x17): clears the membership SKU array at [this+0x3b97c] (count [this+0x3b980]), parses the… |
| `00b46b90` | ClientInGamePurchaseManager::RequestStartupDataInternal 🌐 | confirmed | Start-up gate for the store data: if [this+0x3e82a] is 0 it logs and sends the account-info request (00b45010); if the currency map [this+0x3b8f4] is empty it … |
| `00b47690` | HandleItemOfTheDay 🌐 | confirmed | Handles the InGamePurchase item-of-the-day message (dispatcher sub-opcode 0x20): constructs the IOTD packet object from [this+0x35940] with FUN_00b24160, parse… |
| `00b477e0` | HandleBundleOfTheDay 🌐 | confirmed | Handles the InGamePurchase bundle-of-the-day message (dispatcher sub-opcode 0x29): constructs the BOTD packet from [this+0x35978] with FUN_00b24230, parses it … |
| `00b47f20` | HandleExclusivePartnerStoreBundles 🌐 | confirmed | Handles the exclusive-partner store bundle message (dispatcher sub-opcode 8): constructs a temporary packet holder with FUN_00b24300, builds a read stream over… |
| `00b48040` | HandleStoreBundlesCategoriesResponse 🌐 | confirmed | Handles the store bundle categories response (dispatcher sub-opcode 7): resets the category tree at [this+0x343a0] with FUN_00b22fd0, builds a read stream over… |
| `00b48a10` | HandleInGamePurchasePacket 🌐 | confirmed | Main dispatch for opcode 66 (InGamePurchase): builds a read stream over (param_2,param_3), reads the 16-bit sub-opcode with FUN_00b2dd60 and switches on it, fo… |
| `00b460c0` | HandleCurrencyCodesResponse 🌐 | likely | Handles PacketInGamePurchaseCurrencyCodesResponse (sub-opcode 0x11): first frees the node list at [this+0x3b8ec] through FUN_00b3a930 and clears the structure … |
| `00b46200` | HandleCountryCodesResponse 🌐 | likely | Clears the country list, parses the response payload, sorts the parsed entries by their name strings, compares entries against a built-in country-name table, s… |
| `00b44fa0` | ClientInGamePurchaseManager::SendStateCodesRequest 🌐 | speculative | Builds a PacketInGamePurchaseStateCodesRequest on the stack (constructor FUN_00b233c0, whose object vtable is 0181e208) and, when the gateway pointer [*(this+4… |
| `00b46dc0` | SendWorldPacketFromMember 🌐 | speculative | Serialises the fixed payload held at [this+0x1c] with FUN_00911c30(this+0x1c) into a temporary packet buffer (FUN_00b36a10/FUN_00b3fc60), then wraps the buffer… |
| `00b485e0` | SerializeAndSendStoreData 🌐 | speculative | Allocates a large temporary buffer (FUN_00b36a10), serialises the store object at [param_1+0x18] into it with 00b453f0, wraps the buffer contents (pointer at [… |
| `00b45010` | — 🌐 | unknown | Constructs a PacketInGamePurchaseAccountInfoRequest on the stack (constructor FUN_00b2fe40, object vtable 0181e690), passing the client's locale string [this+0… |
| `00b45560` | — 🌐 | unknown | Deserialises a PacketInGamePurchaseStoreBundleGroups payload from the (data,length) pair: starts a read stream, reads two 16-bit header values (FUN_00b2dd60), … |
| `00b45f60` | — 🌐 | unknown | Handles PacketInGamePurchaseAccountInfoResponse (sub-opcode 0x1a): parses the payload with FUN_00b2e080; when the first int is 1 it copies the two returned str… |
| `00b46a50` | — 🌐 | unknown | Handles PacketInGamePurchaseStateCodesResponse (sub-opcode 0x13): clears the state list at [this+0x3d61c], parses the payload with FUN_00b41800 through a tempo… |
| `00b473f0` | — 🌐 | unknown | Serialises the list object at [this+0x18] into a temporary buffer with FUN_00b41a30(buffer,this+0x18,cookie), then wraps the buffer in a ClientGateway::PacketT… |
| `00b47a50` | — 🌐 | unknown | Only when [this+0x3e828], [this+0x3b45d] and [this+0x3e829] are all non-zero: asks the embedded store data manager at [this+0x3b460] whether it is ready (virtu… |
| `00b48370` | — 🌐 | unknown | Entry point (reached through the thunk at 00cc4be0) for one store refresh: only if [this+0x3e829] is set and the embedded manager at [this+0x3b460] reports rea… |
| `00b48700` | — 🌐 | unknown | Handles the store-bundle-base family (dispatcher sub-opcode 5) from a (data,length) pair: parses the base packet (two 16-bit fields with FUN_00b2dfd0 on a temp… |

**CoinStoreBuyBackRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c701d0` | CoinStoreBuyBackRequestPacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 0xc (12), installs CoinStoreBuyBackRequestPacket::vftable, copies a global (DAT_01beea20) into word [3], zeroes word… |
| `00c70240` | CoinStoreBuyBackRequestPacket::ScalarDeletingDtor | confirmed | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**CoinStoreBuyBackResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c702b0` | CoinStoreBuyBackResponsePacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 0xd (13), installs CoinStoreBuyBackResponsePacket::vftable, copies a global (DAT_01beea20) into word [3], zeroes wor… |
| `00c70320` | CoinStoreBuyBackResponsePacket::ScalarDeletingDtor | confirmed | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**CoinStoreBuyFromClientRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c70010` | CoinStoreBuyFromClientRequestPacket::ScalarDeletingDtor | confirmed | Stores BaseCoinStorePacket and BasePacket vtables (the derived class's own vtable is not stored here), then calls _free(this) through thunk 00d82463 when bit 0… |
| `00c6ff90` | — 🌐 | unknown | Calls the base constructor with sub-opcode 5, installs CoinStoreBuyFromClientRequestPacket::vftable, copies the two globals DAT_01beea08/DAT_01beea0c into +0x1… |

**CoinStoreClearTransactionHistoryPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c70160` | CoinStoreClearTransactionHistoryPacket::ScalarDeletingDtor | confirmed | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
| `00c6f040` | — 🌐 | unknown | Calls the base constructor with sub-opcode 0xb (11), installs CoinStoreClearTransactionHistoryPacket::vftable, returns this. |

**CoinStoreGiftTransactionCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c751b0` | CoinStoreGiftTransactionCompletePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the shared base destructor body FUN_00c74430 on this, then calls _free(this) when bit 0 of the flag argument is set; returns … |
| `00c743c0` | — 🌐 | unknown | Constructor: calls the base constructor FUN_00c6e960 with 0x12, stores the CoinStoreGiftTransactionCompletePacket vtable, sets the field at +0xc to 2, then cal… |

**CoinStoreItemDefinitionsRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ea90` | CoinStoreItemDefinitionsRequestPacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 2, installs CoinStoreItemDefinitionsRequestPacket::vftable, zeroes the word at +0xc, returns this. |
| `00c6eaf0` | CoinStoreItemDefinitionsRequestPacket::Dtor | confirmed | Installs CoinStoreItemDefinitionsRequestPacket, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6eb50` | CoinStoreItemDefinitionsRequestPacket::ScalarDeletingDtor | confirmed | Reinstalls the request, base and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**CoinStoreItemDefinitionsResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ebc0` | CoinStoreItemDefinitionsResponsePacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 3, installs CoinStoreItemDefinitionsResponsePacket::vftable, zeroes the word at +0xc and the byte at +0x10, returns … |
| `00c6ec20` | CoinStoreItemDefinitionsResponsePacket::Dtor | confirmed | Installs CoinStoreItemDefinitionsResponsePacket, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6ec80` | CoinStoreItemDefinitionsResponsePacket::ScalarDeletingDtor | confirmed | Reinstalls the response, base and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**CoinStoreItemDynamicListUpdateRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ee60` | CoinStoreItemDynamicListUpdateRequestPacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 8, installs CoinStoreItemDynamicListUpdateRequestPacket::vftable, returns this. |
| `00c6eec0` | CoinStoreItemDynamicListUpdateRequestPacket::Dtor | confirmed | Installs the request, base and BasePacket vtables in turn; no free and no callees. |
| `00c6ef20` | CoinStoreItemDynamicListUpdateRequestPacket::ScalarDeleting… | confirmed | Reinstalls the request, base and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**CoinStoreItemDynamicListUpdateResponsePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6f500` | CoinStoreItemDynamicListUpdateResponsePacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 9, installs the CoinStoreItemDynamicListUpdateResponsePacket<HashList<ItemDefinitionMetaData,256,-1>> vtable, zeroes… |
| `00c6f560` | CoinStoreItemDynamicListUpdateResponsePacket::Dtor | confirmed | Installs the update-response, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6f630` | CoinStoreItemDynamicListUpdateResponsePacket::ScalarDeletin… | confirmed | Reinstalls the update-response and BaseCoinStorePacket vtables, then BasePacket, then calls _free(this) through thunk 00d82463 when bit 0 of the second argumen… |

**CoinStoreItemListPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6f440` | CoinStoreItemListPacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 1, installs the CoinStoreItemListPacket<HashList<ItemDefinitionMetaData,4096,1024>,...> vtable, zeroes the words at … |
| `00c6f4a0` | CoinStoreItemListPacket::Dtor | confirmed | Installs the item-list, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6f5c0` | CoinStoreItemListPacket::ScalarDeletingDtor | confirmed | Reinstalls the item-list, BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. R… |

**CoinStoreMerchantListPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6ef90` | CoinStoreMerchantListPacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 10, installs CoinStoreMerchantListPacket::vftable, zeroes the word at +0xc, returns this. |
| `00c700f0` | CoinStoreMerchantListPacket::ScalarDeletingDtor | confirmed | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**CoinStoreOpenPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c74490` | CoinStoreOpenPacket::Construct 🌐 | confirmed | Constructor: calls the base constructor FUN_00c6e960 with 7, stores the CoinStoreOpenPacket vtable, calls FUN_00c72420 and FUN_0079e0d0 for members, and stores… |
| `00c751d0` | CoinStoreOpenPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body FUN_00c74520 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c74520` | — | unknown | Destructor body for the store-open packet: calls FUN_0079e170 on the object, then FUN_00c72e50 on its member, then resets the vtable to BaseCoinStorePacket and… |
| `00c74f70` | — | unknown | Parses the store-open body from the bounded reader at param_2: calls FUN_008dab40 into the sub-structure at this+0x334 and FUN_00c72060 into this+0xc, then rea… |

**CoinStoreReceiveGiftItemPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c751f0` | CoinStoreReceiveGiftItemPacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body FUN_00c74620 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c74590` | — 🌐 | unknown | Constructor: calls the base constructor FUN_00c6e960 with 0x11, stores the CoinStoreReceiveGiftItemPacket vtable, sets the SoeUtil::IString<char> member at +0x… |
| `00c74620` | — | unknown | Destructor body for the gift item packet: calls FUN_00c00d50 on the object, resets the SoeUtil::IString<char> member vtable at +0x18, releases the string buffe… |

**CoinStoreSellToClientAndGiftRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c75190` | CoinStoreSellToClientAndGiftRequestPacket::ScalarDeletingDt… | confirmed | Scalar deleting destructor: calls the shared base destructor body FUN_00c74360 on this, then calls _free(this) when bit 0 of the flag argument is set; returns … |
| `00c75130` | — 🌐 | unknown | Constructor: calls the base constructor FUN_00c6e960 with 0xe, stores the CoinStoreSellToClientAndGiftRequestPacket vtable, and calls the security-cookie check… |

**CoinStoreSellToClientRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6fef0` | CoinStoreSellToClientRequestPacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 4, installs CoinStoreSellToClientRequestPacket::vftable, copies two words from globals DAT_01beea08/DAT_01beea0c int… |
| `00c6ff70` | CoinStoreSellToClientRequestPacket::ScalarDeletingDtor | confirmed | Calls 00c6ecf0 (embedded ItemRecord teardown), then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
| `00c6ecf0` | CoinStoreSellToClientRequestPacket::ItemRecordMemberDtor | likely | Stores ItemRecord::vftable into word [7] (offset +0x1c, an embedded ItemRecord), then resets the BaseCoinStorePacket and BasePacket vtables. No free and no cal… |

**CoinStoreTransactionCompletePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c6eda0` | CoinStoreTransactionCompletePacket::Ctor 🌐 | confirmed | Calls the base constructor with sub-opcode 6, installs CoinStoreTransactionCompletePacket::vftable, sets word [3] to 2 and word [5] to 0 around a stack-cookie … |
| `00c70080` | CoinStoreTransactionCompletePacket::ScalarDeletingDtor | confirmed | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |

**(no class)** (45)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c752d0` | HandleCoinStoreClearTransactionHistory | confirmed | Receive handler for the clear-transaction-history packet: constructs a temporary with FUN_00c6f040, sets up a bounded reader over the body and requires no over… |
| `00c75fc0` | HandleCoinStoreItemListPacket 🌐 | confirmed | Receive handler for the item-list packet: calls FUN_00c71670, constructs a 256-bucket HashList temporary with FUN_00c73c30, FUN_00c6f440 and the decoder FUN_00… |
| `00c76100` | HandleCoinStoreItemDynamicListUpdateResponse 🌐 | confirmed | Receive handler for the dynamic item-list update response: constructs a 256-bucket HashList temporary with FUN_00c73c30 and FUN_00c6f500, decodes the body with… |
| `00c764f0` | HandleCoinStoreReceiveGiftItem | confirmed | Constructs CoinStoreReceiveGiftItemPacket with 00c74590, parses the body with 00c758a0, and when the parse succeeds walks the returned node list (next link +0x… |
| `00c76600` | SendBuyBackRequest 🌐 | confirmed | Initialises a 0x10000000-capacity serializer through FUN_008d4a40, writes the buy-back body with FUN_00c759c0, wraps the buffer in a PacketTunneledClientPacket… |
| `00c76710` | SendItemDynamicListUpdateRequest 🌐 | confirmed | Initialises a 0x10000000-capacity serializer through FUN_008d4a40, writes the dynamic list update body with FUN_00c75580, wraps the buffer in a PacketTunneledC… |
| `00c76820` | SendSellItemRequest 🌐 | confirmed | Initialises a 0x10000000-capacity serializer through FUN_008d4a40, writes the sell body with FUN_00c75a70, wraps the buffer in a PacketTunneledClientPacket via… |
| `00c76a00` | SendBuyItemRequest 🌐 | confirmed | Initialises a 0x10000000-capacity serializer through FUN_008d4a40, writes the purchase body with FUN_00c75ed0, wraps the buffer in a PacketTunneledClientPacket… |
| `00c76b10` | SendItemDefinitionsRequest 🌐 | confirmed | Initialises a 0x10000000-capacity serializer through FUN_008d4a40, writes the id pair with FUN_00c75580 and, when the object at param_1+0xc is non-null, the ke… |
| `00c76dd0` | RequestSellItem 🌐 | confirmed | Calls FUN_00c6ff90 on a temporary record, copies the two merchant GUID words, the item instance GUID word (from the fourth argument) and the quantity word into… |
| `00c77090` | HandleCoinStoreItemDefinitionsResponse 🌐 | confirmed | Receive handler for the item-definitions response: constructs CoinStoreItemDefinitionsResponsePacket with FUN_00c6ebc0 and a temporary Set with FUN_00c73020, d… |
| `00c78890` | HandleCoinStoreOpenPacket | confirmed | Constructs CoinStoreOpenPacket through FUN_00c74490, parses the body with FUN_00c74f70 from a bounded reader over the payload, and requires no overrun and no u… |
| `00b47530` | ReadFixedListPayload | likely | Deserialiser wrapper with no static caller: builds a read stream over (param_2,param_3), reads two 16-bit fields with FUN_00b2dd60, and when the destination's … |
| `00b47b20` | RequestMissingStoreItemsForRecord 🌐 | likely | Looks up the supplied record id at +0x9c; if it is absent from the local id map, builds a request list from entries linked at record+0x50 and sends it through … |
| `00b48120` | RequestMissingStoreItemBatches 🌐 | likely | Walks the store-id tree at +0x2cc; for ids absent from the local map, resolves each store record, appends its linked item entries to a temporary list, sends ba… |
| `00c6f300` | CoinStoreTransactionResultDispatch | likely | Switches on the result code (1-11 and default). Every arm makes the same indirect call: the function at slot 4 (+0x10) of the vtable held at slot 1 (+4) of the… |
| `00c75b30` | SerializeTwoDwordsAfterHeader | likely | Calls FUN_00a1aee0 on the serializer (writes a header), then appends the dwords at param_1+0x10 and param_1+0x14 to the outgoing serializer through FUN_007cc40… |
| `00c75ce0` | DecodeTwoOptionalListSections | likely | Returns 0 unless both param_1 and param_2 are non-zero. Builds a bounded reader over [param_2, param_2+param_3) and, when param_1+0xc is non-zero, calls FUN_00… |
| `00c76390` | ReadGiftTransactionCompleteFields | likely | Sets up a bounded reader over param_2 and param_3 bytes, reads one dword into param_1+0xc (0 with overrun when past the end), then calls FUN_00c750b0 on the re… |
| `00c764b0` | WriteGiftTransactionEntries | likely | Appends the dword at param_2+0xc to the serializer param_1 through FUN_00911420, then for each node in the list at param_2+4 (next link +0x18) calls FUN_00c75b… |
| `00c76c60` | RequestBuyBackItem | likely | Looks up the first argument through FUN_01086f00; only when the returned object's first dword equals 2 does it copy its field at +0x10 and the third argument i… |
| `00c77410` | RequestStoreItemDefinitions 🌐 | likely | When the list at param_2 has a count above zero (dword at +8): constructs a CoinStoreItemDefinitionsRequestPacket temporary through FUN_00c6ea90 with the list,… |
| `00c78fb0` | HandleCoinStoreMerchantList | likely | Constructs CoinStoreMerchantListPacket through FUN_00c6ef90, reads the body from the bounded reader over param_1 and param_2 with FUN_00c71d90 and an optional … |
| `00c79370` | RequestGiftPurchase 🌐 | likely | Rejects a record whose two GUID words equal the session merchant globals DAT_01beea08 and DAT_01beea0c, and returns 0 when the count at record+0x1c is not posi… |
| `00b45340` | ReadStoreDefinitionRecord 🌐 | speculative | Reads a store record from a byte stream: three 4-byte fields into this+4, this+8 and this+0xc, then two length-prefixed UTF-8 strings (FUN_00b2ea50, which assi… |
| `00b453f0` | — | unknown | Writes an object into a caller-owned byte buffer: one flag byte, the 4-byte values at this+8 and this+0xc, two strings at this+0x10 and this+0x18 (FUN_008fd1a0… |
| `00b46ca0` | — | unknown | Parses an item-of-the-day packet body from a (data,length) pair: reads two 16-bit fields with FUN_00b2dd60, and when the destination object's field at +0x18 is… |
| `00b46d30` | — | unknown | Same shape as the item-of-the-day reader but for the bundle-of-the-day packet: reads two 16-bit fields from the (data,length) range with FUN_00b2dd60, then, if… |
| `00b470b0` | — 🌐 | unknown | Builds an empty payload buffer (FUN_00b36a10/FUN_00b3fc60), wraps it in a ClientGateway::PacketTunneledClientWorldPacket (FUN_0089ae70 plus FUN_00926360 called… |
| `00b475b0` | — | unknown | Deserialiser wrapper with no static caller: builds a read stream over (param_2,param_3), reads two 16-bit fields with FUN_00b2dd60, then parses a category tabl… |
| `00c74ef0` | — 🌐 | unknown | Reads two dwords from the bounded reader at param_2 into fields +0xc and +0x10 of param_1 (each read clamps the cursor and sets the overrun flag at reader+0x10… |
| `00c75580` | — | unknown | Writes two 4-byte values into the outgoing serializer at param_2 (write offset dword +0x808, capacity +0x806 minus the buffer header at *param_2+8): the first … |
| `00c75620` | — | unknown | Writes the dword at param_2+8 through FUN_00911420 (count), then walks the tree rooted at param_2+4 in order (links at node+4, node+8 and parent at node+0xc) a… |
| `00c758a0` | — | unknown | Reads an 8-byte value into param_1+0x10/+0x14 from the bounded reader at param_2, then reads a signed count and a raw block of that size through FUN_00894b10 (… |
| `00c759c0` | — | unknown | Calls FUN_00c75580 on the serializer (id pair), then appends the dword at param_1+0xc and the dword at param_1+0x10 to the outgoing serializer through FUN_007c… |
| `00c75a70` | — 🌐 | unknown | Calls FUN_00c75580 on the serializer (id pair), calls FUN_008fb810 on the object at param_1+0x10, then appends the dwords at param_1+0x18 and param_1+0x1c to t… |
| `00c75d80` | — | unknown | Variant of DecodeTwoOptionalListSections: returns 0 unless both pointers are non-zero; builds the same bounded reader, calls FUN_00c74200 only for the list at … |
| `00c75e00` | — | unknown | Builds the same bounded reader as the other decoders; when param_1+0xc is non-zero calls FUN_00b33020 on the reader, then returns 1 when the overrun flag is cl… |
| `00c75ed0` | — 🌐 | unknown | Calls FUN_00c75580 on the serializer (id pair), calls FUN_008fb810 on param_1+0x10, appends the dword at param_1+0x18, calls FUN_00a1aee0 on the serializer (he… |
| `00c76d20` | — 🌐 | unknown | Logs Coin store opened with the referrer id and context (two parameters) through FUN_00704db0, constructs a temporary through FUN_00c6ee60 (base opcode 8 in th… |
| `00c77df0` | — | unknown | Constructs CoinStoreGiftTransactionCompletePacket through FUN_00c743c0 and parses its body with FUN_00c76390. On success logs through FUN_0070bcc0 with the OnG… |
| `00c78220` | — 🌐 | unknown | Constructs CoinStoreTransactionCompletePacket through FUN_00c6eda0 and parses its body with FUN_00c74ef0 (two dwords and an optional payload). On success it se… |
| `00c78a10` | — | unknown | Constructs CoinStoreBuyBackResponsePacket through FUN_00c702b0 and decodes the body with FUN_00c71e40 (two dwords: transaction id and result). On success logs … |
| `00c790a0` | — 🌐 | unknown | Initialises a 0x10000000-capacity serializer through FUN_008d4a40, writes an id pair with FUN_00c75580, two FUN_008fb810 sub-objects at param_1+0x10 and +0x18,… |
| `00c791d0` | — 🌐 | unknown | Constructs a BasePacket temporary with FUN_00c6e960(0), reads the leading 32-bit sub-opcode from the bounded reader over the payload with FUN_00c71d90, and whe… |

### `net.ingamepurchase`

**ECommCommand** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e3430` | ECommCommand::ECommCommand | confirmed | Fastcall default constructor. Calls base ctor 0140c090 on this, stores the ECommCommand vftable at +0, zeroes +4 (AccountID), then calls 012d46a0; if that retu… |
| `014e34e0` | ECommCommand::GetTypeName | confirmed | Returns the address of the constant string "ECommCommand" (mov eax, imm32; ret). |
| `014e3520` | ECommCommand::~ECommCommand | confirmed | Fastcall destructor body (non-deleting). Stores the ECommCommand vftable into *this, calls 0140c160 with this (base destructor, not inspected), returns. |
| `014e3640` | ECommCommand::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to ECommCommand (00d8d382 with NetworkCommand and ECommCommand d… |
| `014e3690` | ECommCommand::Read | confirmed | Thiscall virtual (slot 9). Calls virtual slot 4 (vtable +0x10) with (param_2, &param_2); if nonzero reads one integer into this+4 with 013c24d0; if nonzero cal… |
| `014e36d0` | ECommCommand::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting ECommCommand " (no-op), then 013c26e0 with "Account… |
| `014e37b0` | ECommCommand::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores the ECommCommand vftable into *this, calls 0140c160 with this (base destructor), then if bit 0 of the flag a… |
| `014e3810` | ECommCommand::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "EComm Command " (len 0xe), base text routine 0140c290(param_2), then _sprintf of "AccountID: %d " with this+4 … |
| `014e34f0` | ECommCommand::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8c4 (mov eax,[01cbe8c4]; ret). No this pointer is read. |
| `014e3500` | ECommCommand::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8c4 (mov [01cbe8c4],eax; ret 4). No this pointer is read. |
| `014e3590` | ECommCommand::Slot21Triple | likely | Plain function (no this used). Stores the constant 0x3b9aca04 through its first pointer argument and zero through the second and third. Returns nothing. |
| `014e35d0` | ECommCommand::Clone | likely | Fastcall virtual (slot 17). Allocates 8 bytes with operator new (00d833a6); if non-null constructs the new object with the default constructor 014e3430 (no arg… |
| `014e3730` | ECommCommand::Create | likely | Non-member function. Allocates 8 bytes with operator new (00d833a6); if non-null calls 014e3430 (default ctor) and returns its EAX; if allocation fails returns… |
| `014e38a0` | — | unknown | Virtual (slot 20, one stack argument, ret 4). Loads its argument into ECX without using it, then calls 01241650 with "EComm Command" (len 0xd). Writes a fixed … |

**ECommCommand_OrderHistory** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e29e0` | ECommCommand_OrderHistory::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Calls base CloneInto 014e3640 with the argument (ECX = this), then calls 00d8d382 (dynamic … |
| `014e2a10` | ECommCommand_OrderHistory::Read | confirmed | Thiscall virtual (slot 9 override of the base ECommCommand::Read 014e3690). Calls virtual slot 4 (vtable +0x10) with (param_2, &param_2); if nonzero calls base… |
| `014e2a50` | ECommCommand_OrderHistory::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting ECommCommand_OrderHistory " (no-op, lead-verified);… |
| `014e2fb0` | ECommCommand_OrderHistory::~ECommCommand_OrderHistory | confirmed | Fastcall destructor body (non-deleting). Stores the OrderHistory vftable into *this; for each non-null pointer in the vector at +0xc..+0x14 (begin +0xc, end +0… |
| `014e3100` | ECommCommand_OrderHistory::GetTypeName | confirmed | Returns the address of the constant string "ECommCommand_OrderHistory" (mov eax, imm32; ret). |
| `014e3140` | ECommCommand_OrderHistory::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 014e2fb0 on this, then if bit 0 of the flag argument is set calls _free on this; returns … |
| `014e31d0` | ECommCommand_OrderHistory::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls base DebugPrint 014e3810(param_2) (header and AccountID), then computes the count (this+0x10 minus this+0xc) >> 2 (0 when the… |
| `014e3260` | ECommCommand_OrderHistory::PrintTag | confirmed | Virtual (slot 20). Calls 01241650 with the fixed text "EComm Command" (len 0xd) and returns nothing. |
| `014e32e0` | ECommCommand_OrderHistory::ECommCommand_OrderHistory | confirmed | Fastcall default constructor. Calls base ctor 014e3430 on this, stores the OrderHistory vftable at +0, zeroes the dwords at +0xc, +0x10 and +0x14 (an empty ord… |
| `014e3110` | ECommCommand_OrderHistory::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8c0 (mov eax,[01cbe8c0]; ret). No this pointer is read. |
| `014e3120` | ECommCommand_OrderHistory::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8c0 (mov [01cbe8c0],eax; ret 4). No this pointer is read. |
| `014e3340` | ECommCommand_OrderHistory::Clone | likely | Fastcall virtual (slot 17). Allocates 0x18 bytes with operator new (00d833a6); if non-null constructs the new object with the default constructor 014e32e0 and … |
| `014e33b0` | ECommCommand_OrderHistory::Create | likely | Non-member function. Allocates 0x18 bytes with operator new (00d833a6); if non-null calls 014e32e0 (default ctor) with the block in ECX and returns its EAX; if… |

**GameCommerce::CasApi** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d73d10` | GameCommerce::CasApi::OnUploaderLogDebug | confirmed | Virtual slot [16] of GameCommerce::CasApi::vftable (30 bytes). When the message argument is null, calls the logger at this+0x22e8 (006faad0) with 'OnUploaderLo… |
| `00d73d30` | GameCommerce::CasApi::OnUploaderLogInfo | confirmed | Virtual slot [17] of GameCommerce::CasApi::vftable (30 bytes). When the message argument is null, calls the logger at this+0x22e8 (006faad0) with 'OnUploaderLo… |
| `00d73d50` | GameCommerce::CasApi::OnUploaderLogError | confirmed | Virtual slot [18] of GameCommerce::CasApi::vftable (30 bytes). When the error-message argument is null, calls the logger at this+0x22e8 (006faad0) with 'OnUplo… |
| `00d74030` | — | unknown | Virtual slot [11] of GameCommerce::CasApi::vftable (172 bytes, ECX = CasApi, two stack arguments: the reply object and an error code). When the error code is 0… |

**InGamePurchaseMerchantListPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b1fec0` | InGamePurchaseMerchantListPacketCleanup | confirmed | Writes InGamePurchaseMerchantListPacket::vftable and then BasePacket::vftable at the object start, then returns. |
| `00b23b40` | InGamePurchaseMerchantListPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: writes its own vftable, seeds try level -1, writes BasePacket::vftable and frees this via _free when the flags low bit is set. |
| `00b23ae0` | InGamePurchaseMerchantListPacket::InGamePurchaseMerchantLis… 🌐 | speculative | Constructor: calls FUN_00b22e10 with sub-opcode 0x2a (42), writes InGamePurchaseMerchantListPacket::vftable and zeroes this+0x18. |

**InGamePurchaseUpdateItemRequirementsReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2ff80` | InGamePurchaseUpdateItemRequirementsReply::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x27, installs reply and list vtables, and zeros the list head, tail, and count fields. |
| `00b2fff0` | InGamePurchaseUpdateItemRequirementsReply::Dtor | confirmed | Restores the reply vtable, destroys its owned list, then rewrites the object vtable to BasePacket. |
| `00b335d0` | InGamePurchaseUpdateItemRequirementsReply::ScalarDeletingDt… | confirmed | Calls reply cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the object pointer. |

**InGamePurchaseUpdateItemRequirementsRequest** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c022a0` | InGamePurchaseUpdateItemRequirementsRequest::DtorBody | confirmed | Restores InGamePurchaseUpdateItemRequirementsRequest::vftable and then BasePacket::vftable at the object's first pointer. |
| `00c03920` | InGamePurchaseUpdateItemRequirementsRequest::Ctor 🌐 | confirmed | Calls the base in-game purchase packet constructor with sub-opcode 0x26 and installs InGamePurchaseUpdateItemRequirementsRequest::vftable. |
| `00c03980` | InGamePurchaseUpdateItemRequirementsRequest::ScalarDeleting… | confirmed | Restores the derived request vtable then BasePacket::vftable, and frees this only when flag bit 0 is set. |
| `00c047a0` | SendInGamePurchaseUpdateItemRequirementsRequest 🌐 | confirmed | Constructs an update-item-requirements request and, when the session send field is nonzero, serializes and sends it through the gateway world-packet wrapper. |

**InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23a70` | InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForP… | confirmed | Scalar deleting destructor: writes its own vftable, seeds try level -1, writes BasePacket::vftable, and frees this via _free when the flags low bit is set. |
| `00b23a10` | InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForP… 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0x2d (45), writes its own vftable and zeroes this+0x18 and this+0x1c. |

**PacketBaseInGamePurchase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b22e10` | PacketBaseInGamePurchase::PacketBaseInGamePurchase | confirmed | Writes BasePacket::vftable, stores 0x42 at this+4, writes PacketBaseInGamePurchase::vftable, and stores its caller-supplied sub-opcode at this+8. |
| `00b22e80` | PacketBaseInGamePurchase::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (vtable slot 0): writes BasePacket::vftable to *this and, when the low bit of the flags argument is set, calls _free(this); returns … |

**PacketInGamePurchaseAccountInfoRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fe40` | PacketInGamePurchaseAccountInfoRequest::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x19, writes its request vtable, initializes a payload member from the argument, and returns the packet. |
| `00b2feb0` | PacketInGamePurchaseAccountInfoRequest::ScalarDeletingDtor | confirmed | Calls packet cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the pointer. |

**PacketInGamePurchaseAccountInfoResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fed0` | PacketInGamePurchaseAccountInfoResponse::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x1a, writes the response vtable, stores an argument in field 6, initializes two members, and writes one byte at field 0x54. |
| `00b2ff60` | PacketInGamePurchaseAccountInfoResponse::ScalarDeletingDtor | confirmed | Calls response cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**PacketInGamePurchaseBOTD<AppStoreBundleDefinition>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b242a0` | PacketInGamePurchaseBOTD::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b24230` | PacketInGamePurchaseBOTD::Ctor 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0x29 (41), writes PacketInGamePurchaseBOTD vftable and stores the two 4-byte arguments at this+0x18 and this+0x… |

**PacketInGamePurchaseCountryCodesRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fd10` | PacketInGamePurchaseCountryCodesRequest::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x14, writes its vtable, initializes the payload member from the argument, and returns the packet. |
| `00b2fd80` | PacketInGamePurchaseCountryCodesRequest::ScalarDeletingDtor | confirmed | Calls packet cleanup, conditionally frees the object when flag bit 0 is set, and returns its pointer otherwise. |

**PacketInGamePurchaseCountryCodesResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fda0` | PacketInGamePurchaseCountryCodesResponse::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x15, writes the response vtable, sets field 6 to 2, initializes a member, and stores the argument at field 0x10. |
| `00b2fe20` | PacketInGamePurchaseCountryCodesResponse::ScalarDeletingDtor | confirmed | Calls packet cleanup, frees the allocation when flag bit 0 is set, and otherwise returns the object pointer. |

**PacketInGamePurchaseCurrencyCodesRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fbe0` | PacketInGamePurchaseCurrencyCodesRequest::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x10, writes the request vtable, initializes its payload member from the argument, and returns the packet. |
| `00b2fc50` | PacketInGamePurchaseCurrencyCodesRequest::ScalarDeletingDtor | confirmed | Calls packet cleanup, frees the object when flag bit 0 is set, and otherwise returns the original pointer. |

**PacketInGamePurchaseCurrencyCodesResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fc70` | PacketInGamePurchaseCurrencyCodesResponse::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x11, writes the response vtable, sets field 6 to 2, initializes a member, and stores the supplied value at field 0x10. |
| `00b2fcf0` | PacketInGamePurchaseCurrencyCodesResponse::ScalarDeletingDt… | confirmed | Calls packet cleanup, frees the allocation when flag bit 0 is set, and otherwise returns the object pointer. |

**PacketInGamePurchaseDisplayMannequinStoreBundles** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23780` | PacketInGamePurchaseDisplayMannequinStoreBundles::ScalarDel… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b23710` | PacketInGamePurchaseDisplayMannequinStoreBundles::Construct 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0x1f (31), writes its own vftable, stores the second argument at this+0x18 and the first argument at this+0x1c. |

**PacketInGamePurchaseEnableMarketplace** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23550` | PacketInGamePurchaseEnableMarketplace::PacketInGamePurchase… 🌐 | confirmed | Constructor: calls FUN_00b22e10 with sub-opcode 0x18 (24), writes PacketInGamePurchaseEnableMarketplace::vftable and stores the single-byte argument at this+0x… |
| `00b235c0` | PacketInGamePurchaseEnableMarketplace::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |

**PacketInGamePurchaseExclusivePartnerStoreBundles<SoeUtil::HashList<AppStoreBundleDefinition,32,-1>>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b24370` | PacketInGamePurchaseExclusivePartnerStoreBundles::ScalarDel… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b24300` | PacketInGamePurchaseExclusivePartnerStoreBundles::Ctor 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 8, writes PacketInGamePurchaseExclusivePartnerStoreBundles vftable and stores the 4-byte argument at this+0x18. |

**PacketInGamePurchaseGiftNotification** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b239e0` | PacketInGamePurchaseGiftNotification::ScalarDeletingDestruc… | confirmed | 27-byte scalar deleting destructor: pushes esi, moves ECX (this) to esi, calls FUN_00b1fdf0 (the class destructor body, which resets the BasePacket vtable and … |
| `00b23980` | — 🌐 | unknown | Constructor: calls FUN_00b22e10 with sub-opcode 0, writes PacketInGamePurchaseGiftNotification::vftable and then calls FUN_01077f70 with this to construct the … |

**PacketInGamePurchaseIOTD<AppStoreBundleDefinition>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b241d0` | PacketInGamePurchaseIOTD::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b24160` | PacketInGamePurchaseIOTD::Ctor 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0x20 (32), writes PacketInGamePurchaseIOTD vftable, stores the first 4-byte argument at this+0x18 and the secon… |

**PacketInGamePurchasePlaceOrderPacket<GameCommerce::ClientInGamePurchaseOrder>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b24440` | PacketInGamePurchasePlaceOrderPacket::ScalarDeletingDestruc… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b243d0` | PacketInGamePurchasePlaceOrderPacket::Ctor | likely | Constructor: calls FUN_00b22e10 with sub-opcode 3, writes PacketInGamePurchasePlaceOrderPacket vftable and stores the 4-byte argument (the order object) at thi… |

**PacketInGamePurchasePlaceOrderResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2f8d0` | PacketInGamePurchasePlaceOrderResponse::Ctor 🌐 | confirmed | Initializes packet base with opcode 4, writes the response vtable, stores two arguments in fields 6 and 7, initializes a subobject, and stores two more argumen… |
| `00b2f960` | PacketInGamePurchasePlaceOrderResponse::ScalarDeletingDtor | confirmed | Calls the packet cleanup routine, frees the object when flag bit 0 is set, and otherwise returns the original pointer. |

**PacketInGamePurchasePreviewOrderResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b27ea0` | PacketInGamePurchasePreviewOrderResponse::ScalarDeletingDes… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b27e20` | PacketInGamePurchasePreviewOrderResponse::PacketInGamePurch… 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 2, writes PacketInGamePurchasePreviewOrderResponse::vftable and stores four payload words at this+0x18, +0x1c, … |

**PacketInGamePurchaseServerStatusRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b231c0` | PacketInGamePurchaseServerStatusRequest::ScalarDeletingDest… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b23160` | PacketInGamePurchaseServerStatusRequest::PacketInGamePurcha… 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0xc and writes PacketInGamePurchaseServerStatusRequest::vftable. |

**PacketInGamePurchaseServerStatusResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23290` | PacketInGamePurchaseServerStatusResponse::ScalarDeletingDes… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b23220` | PacketInGamePurchaseServerStatusResponse::PacketInGamePurch… 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0xd, writes PacketInGamePurchaseServerStatusResponse::vftable and stores the single byte argument at this+0x18. |

**PacketInGamePurchaseStateCodesRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b233c0` | PacketInGamePurchaseStateCodesRequest::PacketInGamePurchase… 🌐 | confirmed | Constructor: calls FUN_00b22e10 with sub-opcode 0x12 (18) and writes PacketInGamePurchaseStateCodesRequest::vftable. |
| `00b23420` | PacketInGamePurchaseStateCodesRequest::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |

**PacketInGamePurchaseStateCodesResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b234f0` | PacketInGamePurchaseStateCodesResponse::ScalarDeletingDestr… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b23480` | — 🌐 | unknown | Constructor: calls FUN_00b22e10 with sub-opcode 0x13 (19), writes PacketInGamePurchaseStateCodesResponse::vftable, sets this+0x18 = 2 and stores the argument a… |

**PacketInGamePurchaseStationCashProductsRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fa10` | PacketInGamePurchaseStationCashProductsRequest::Ctor 🌐 | confirmed | Initializes packet base with opcode 0xe, writes its request vtable, initializes a member from the argument, and returns the packet. |
| `00b2fa80` | PacketInGamePurchaseStationCashProductsRequest::ScalarDelet… | confirmed | Calls request packet cleanup, conditionally frees the object when flag bit 0 is set, and returns the pointer otherwise. |

**PacketInGamePurchaseStationCashProductsResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23360` | PacketInGamePurchaseStationCashProductsResponse::ScalarDele… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b232f0` | PacketInGamePurchaseStationCashProductsResponse::Ctor 🌐 | speculative | Calls FUN_00b22e10 with sub-opcode 0xf, writes its RTTI vtable, stores 2 at this+0x18 and the caller argument at this+0x1c. |

**PacketInGamePurchaseStoreBundleBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b22f00` | PacketInGamePurchaseStoreBundleBase::PacketInGamePurchaseSt… 🌐 | confirmed | Constructor: calls the family base constructor FUN_00b22e10 with sub-opcode 5, writes PacketInGamePurchaseStoreBundleBase::vftable, stores the 4-byte argument … |
| `00b22f70` | PacketInGamePurchaseStoreBundleBase::ScalarDeletingDestruct… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable to *this, calls _free(this) when the flags low bit is set, returns this. |

**PacketInGamePurchaseStoreBundleCategories** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b22fd0` | PacketInGamePurchaseStoreBundleCategories::PacketInGamePurc… 🌐 | confirmed | Constructor: calls FUN_00b22e10 with sub-opcode 7, writes PacketInGamePurchaseStoreBundleCategories::vftable, stores the 4-byte argument at this+0x18. |
| `00b23040` | PacketInGamePurchaseStoreBundleCategories::ScalarDeletingDe… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |

**PacketInGamePurchaseStoreBundleCategoryGroups** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b237e0` | PacketInGamePurchaseStoreBundleCategoryGroups::PacketInGame… 🌐 | confirmed | Constructor: calls FUN_00b22e10 with sub-opcode 6, writes PacketInGamePurchaseStoreBundleCategoryGroups::vftable and stores the 4-byte argument at this+0x18. |
| `00b23850` | PacketInGamePurchaseStoreBundleCategoryGroups::ScalarDeleti… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |

**PacketInGamePurchaseStoreBundleContentRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b43230` | PacketInGamePurchaseStoreBundleContentRequest::PacketInGame… 🌐 | confirmed | Constructs the in-game-purchase base envelope with sub-opcode 0x1b, installs PacketInGamePurchaseStoreBundleContentRequest::vftable and initializes the embedde… |
| `00b43b20` | PacketInGamePurchaseStoreBundleContentRequest::ScalarDeleti… | confirmed | The 27-byte x86 wrapper saves this, calls destructor body 00b43290, tests bit 0 of the flag byte, conditionally calls _free(this), restores the saved register … |

**PacketInGamePurchaseStoreBundleGroups** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23640` | PacketInGamePurchaseStoreBundleGroups::PacketInGamePurchase… 🌐 | confirmed | Constructor: calls FUN_00b22e10 with sub-opcode 9, writes PacketInGamePurchaseStoreBundleGroups::vftable and stores the 4-byte argument at this+0x18. |
| `00b236b0` | PacketInGamePurchaseStoreBundleGroups::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |

**PacketInGamePurchaseStoreBundleUpdate<AppStoreBundleDefinition>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b24100` | PacketInGamePurchaseStoreBundleUpdate::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b24090` | PacketInGamePurchaseStoreBundleUpdate::Ctor 🌐 | speculative | Calls FUN_00b22f00 with inner tag 2; that base constructor builds family 66/sub-opcode 5 and stores the tag at +0x18. Then writes its own vtable and stores the… |

**PacketInGamePurchaseStoreBundles<AppStoreBundleDefinition>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b28450` | PacketInGamePurchaseStoreBundles::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b283e0` | — 🌐 | unknown | Calls FUN_00b22f00 with inner tag 1, building family 66/sub-opcode 5; writes the PacketInGamePurchaseStoreBundles vtable, stores the caller pointer at +0x20 an… |

**PacketInGamePurchaseStoreClientStatistics<GameCommerce::ClientInGamePurchaseOrder>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b24030` | PacketInGamePurchaseStoreClientStatistics::ScalarDeletingDe… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b23fc0` | PacketInGamePurchaseStoreClientStatistics::Ctor | likely | Constructor: calls FUN_00b22e10 with sub-opcode 0x1d (29), writes PacketInGamePurchaseStoreClientStatistics vftable and stores the 4-byte argument at this+0x18. |

**PacketInGamePurchaseStoreEnablePaymentSources** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b238b0` | PacketInGamePurchaseStoreEnablePaymentSources::PacketInGame… 🌐 | confirmed | Constructor: calls FUN_00b22e10 with sub-opcode 0x21 (33), writes PacketInGamePurchaseStoreEnablePaymentSources::vftable and stores two byte arguments at this+… |
| `00b23920` | PacketInGamePurchaseStoreEnablePaymentSources::ScalarDeleti… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |

**PacketInGamePurchaseSubscriptionProductsRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2faa0` | PacketInGamePurchaseSubscriptionProductsRequest::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x16, writes the request vtable, and initializes two members from the supplied arguments. |
| `00b2fb20` | PacketInGamePurchaseSubscriptionProductsRequest::ScalarDele… | confirmed | Calls packet cleanup, frees the object if flag bit 0 is set, and otherwise returns its original pointer. |

**PacketInGamePurchaseSubscriptionProductsResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2fb40` | PacketInGamePurchaseSubscriptionProductsResponse::Ctor 🌐 | confirmed | Initializes packet base with opcode 0x17, writes the response vtable, sets field 6 to 2, initializes a member, and stores the argument in field 0xe. |
| `00b2fbc0` | PacketInGamePurchaseSubscriptionProductsResponse::ScalarDel… | confirmed | Calls packet cleanup, conditionally frees the allocation when flag bit 0 is set, and returns the object pointer otherwise. |

**PacketInGamePurchaseWalletInfoRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b23100` | PacketInGamePurchaseWalletInfoRequest::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
| `00b230a0` | PacketInGamePurchaseWalletInfoRequest::PacketInGamePurchase… 🌐 | likely | Constructor: calls FUN_00b22e10 with sub-opcode 10 and writes PacketInGamePurchaseWalletInfoRequest::vftable; no other fields initialised. |

**PacketInGamePurchaseWalletInfoResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b2f980` | PacketInGamePurchaseWalletInfoResponse::Ctor 🌐 | confirmed | Initializes packet base with opcode 0xb, writes its vtable, stores one argument in field 6, initializes the nested data object, and returns the packet. |
| `00b2f9f0` | PacketInGamePurchaseWalletInfoResponse::ScalarDeletingDtor | confirmed | Calls packet cleanup, frees the object only when flag bit 0 is set, and otherwise returns the object pointer. |

**(no class)** (24)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b3ad20` | ReadStoreBundleCategoryGroups 🌐 | confirmed | Removes the current category-group nodes, reads a group count and each group's two scalar values, creates a group record, and parses its nested fields from the… |
| `00b3b450` | HandlePreviewOrderResponse 🌐 | confirmed | Constructs the preview-order response packet object, decodes its payload fields, builds the Lua arguments and invokes OnOrderPreviewResponse when parsing succe… |
| `00b3bde0` | HandleServerStatusResponse 🌐 | confirmed | Constructs the server-status response object, reads one boolean byte from the bounded payload, and invokes OnReceiveServerStatus when the payload is fully cons… |
| `00b3bf30` | HandleMarketplaceEnabledResponse 🌐 | confirmed | Constructs the enable-marketplace response object, reads one boolean from the bounded payload, and updates manager state through NotifyMarketplaceEnabled when … |
| `00b3c000` | HandleStorePaymentSourcesResponse 🌐 | confirmed | Constructs the payment-sources response object, reads two signed 16-bit values and two boolean bytes, then stores the booleans at manager offsets 0x3e830 and 0… |
| `00b3c0d0` | HandleMerchantSellBundleQuantityUpdate 🌐 | confirmed | Constructs the native subtype 0x2d response object, reads two signed 16-bit values and two dwords from the supplied payload, logs when input is null, the bound… |
| `00b3d770` | ReadStoreBundleGroups 🌐 | confirmed | Removes the current group nodes, reads a count from the bounded stream, creates each group from its scalar key, parses the group entry, and links the result in… |
| `00b3d930` | ReadCounted128ByteRecords 🌐 | confirmed | Reads and bounds-checks a record count, resizes the destination for 0x80-byte elements, then advances over each record's four signed-length byte regions. |
| `00b3d9c0` | ReadCounted72ByteRecords 🌐 | confirmed | Reads and bounds-checks a record count, resizes the destination for 0x48-byte elements, and advances over the two bounded length-prefixed regions in each recor… |
| `00b3e120` | HandleWalletInfoResponse 🌐 | confirmed | Decodes the wallet response fields; on result code one it calls four helpers with decoded values, sets the manager wallet-ready flag, and invokes OnWalletInfo … |
| `00b3e340` | HandleDisplayMannequinStoreBundles 🌐 | confirmed | Constructs the native mannequin-store-bundles response object, parses its payload into a bundle-id set, and calls the store-item preview API with preview kind … |
| `00b3e490` | HandleItemRequirementsReply 🌐 | confirmed | Constructs and parses the item-requirements reply and its counted entries, then updates the matching item record's requirement flag for every parsed entry. |
| `00b43c40` | RequestStationCashSkuPrices | confirmed | When both manager flags at +0x3e828 and +0x3b45d are set, requires account-info flag +0x3e82a; otherwise logs an error. With account info it prepares a request… |
| `00b43d10` | RequestSubscriptionSkuPrices 🌐 | confirmed | When manager flags at +0x3e828 and +0x3b45d are set, requires account-info flag +0x3e82a; otherwise logs an error. With account info it prepares a subscription… |
| `00b43bc0` | BuildWalletInfoRequest 🌐 | likely | Constructs PacketInGamePurchaseWalletInfoRequest, checks the owner's connection at +4+0x280, wraps the request through 00b40da0 with flag 1 when connected, and… |
| `00b44e60` | BuildCurrencyCodesRequest 🌐 | likely | Prepares the request from manager identifier +0x37bd4; if connection field +4+0x280 is live, builds the serialized payload through 00b41120 with flag 1, then r… |
| `00b44f00` | BuildCountryCodesRequest 🌐 | likely | Prepares the request from manager field +0x37bd4; if connection field +4+0x280 is live, builds the payload through 00b41240 with flag 1, then releases temporar… |
| `00d74340` | AdjustorThunk0x148_IStringAssign | likely | Adjustor thunk (11 bytes): adds 0x148 to ECX and jumps to 00402ce0 (IStringAssign). It does not contain the assignment body; the target assigns the string argu… |
| `00d74360` | AdjustorThunk0x138_IStringAssign | likely | Adjustor thunk (11 bytes): adds 0x138 to ECX and jumps to 00402ce0 (IStringAssign). It does not contain the assignment body; the target assigns the string argu… |
| `014e3410` | RegisterFactory0x1b8 | likely | Pushes the address of 014e33b0 and key 0x1b8 (decimal 440), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair… |
| `014e3790` | RegisterFactory0x1b7 | likely | Pushes the address of 014e3730 and key 0x1b7 (decimal 439), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair… |
| `00b3b6d0` | — 🌐 | unknown | Constructs and decodes the place-order response; on a matching active transaction it invokes OnOrderResponse and the success-side merchant and item notificatio… |
| `00b40050` | — 🌐 | unknown | Handler for a store-bundles payload: constructs a PacketInGamePurchaseStoreBundleBase record (FUN_00b22f00), parses (param_2, param_3) through the reader helpe… |
| `00b44d70` | — 🌐 | unknown | Constructs the category-groups record, parses the supplied bounded payload into it, and returns on successful complete consumption; on failure it drains the ca… |

### `net.promotions`

**ClientPromotionalManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a05bb0` | ClientPromotionalManager::LogLevel4 | confirmed | Virtual log method: forwards to FUN_00700f90 with severity 4, file name Promotions.log and the caller's varargs. |
| `00a05bd0` | ClientPromotionalManager::LogLevel3 | confirmed | Virtual log method: forwards to FUN_00700f90 with severity 3, file name Promotions.log and the caller's varargs. |
| `00a07500` | ClientPromotionalManager::~ClientPromotionalManager | confirmed | Manager destructor: installs the manager vtable, walks the list of registered objects at +0x310 (value 0x1C4 of the manager) unregistering each id through FUN_… |
| `00a07b80` | ClientPromotionalManager::ClientPromotionalManager | confirmed | Manager constructor: installs the ClientPromotionalManager vtable, stores the owner pointer at +4, runs the base constructor 01062450, sets the byte flag at +0… |
| `00a05b90` | ClientPromotionalManager::LogLevel5 | likely | Virtual log method: forwards to the logging service FUN_00700f90 with severity 5, the fixed file name Promotions.log and the caller's varargs. |
| `00a05bf0` | ClientPromotionalManager::LogLevel2 | likely | Virtual log method: forwards to FUN_00700f90 with severity 2, file name Promotions.log and the caller's varargs. |

**SoeUtil::Hash<PromotionalBundleData,64,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a06f60` | Hash_PromotionalBundleData64_Construct | confirmed | Initializes the 64-bucket storage and count, then installs SoeUtil::Hash<PromotionalBundleData,64,-1>::vftable as the final vtable. |
| `00a06fe0` | Hash_PromotionalBundleData64_ScalarDeletingDtor | confirmed | Scalar deleting destructor for the Hash base: stores the HashMap vtable, drains the map with 00a06450 and frees this when the delete flag is set. |

**SoeUtil::HashList<PromotionalBundleDatasource*,256,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a076e0` | HashList_PromotionalBundleDatasource_ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the datasource-map teardown 00a07490 and frees this when the delete flag is set. |

**SoeUtil::HashListMap<int,PromotionalBundleDatasource*,256,-1>** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a05cd0` | HashListMap_PromotionalBundleDatasource256_Construct | confirmed | Constructor of the SoeUtil hash-list-map instantiation: stores the HashListMap vtable, zeroes head/tail/count at +8/+0xc/+0x10 and the extra word at +4, then m… |
| `00a06330` | HashListMap_PromotionalBundleDatasource256_NewNode | confirmed | Node allocator of the datasource map: requests a 0x14-byte node from the thread/global memory manager, falling back to FUN_006fb950(0x14,4). |
| `00a06370` | HashListMap_PromotionalBundleDatasource256_FreeNode | confirmed | Node deallocator of the datasource map: routes the node pointer to FreeMemory with alignment 4. |
| `00a070b0` | HashListMap_PromotionalBundleDatasource256_Destruct | confirmed | Destructor body of the datasource map: stores the HashListMap vtable and repeatedly removes the head node with 00a06290 until the list is empty. |
| `00a070e0` | HashListMap_PromotionalBundleDatasource256_ScalarDeletingDt… | confirmed | Scalar deleting destructor: drains the datasource map through 00a06290 and frees this when the delete flag is set. |
| `00a07490` | HashListMap_PromotionalBundleDatasource256_Destruct_B | confirmed | Second emission of the datasource-map destructor: stores the HashListMap vtable and drains the map through 00a06290 under an SEH frame. |
| `00a07660` | HashListMap_PromotionalBundleDatasource256_Construct_B | confirmed | Full constructor of the datasource map: stores the HashListMap vtable, clears head/tail/count and the 256 bucket pointers (0x400 bytes at +0x14), then reinstal… |

**SoeUtil::HashMap<int,PromotionalBundleData,64,-1>** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a05c50` | HashMap_PromotionalBundleData64_Construct | confirmed | Constructor of the SoeUtil hash map instantiation: stores the HashMap vtable, zeroes the count at +8 and the extra word at +4, and memsets the 64 bucket pointe… |
| `00a06220` | HashMap_PromotionalBundleData64_NewNode | confirmed | Node allocator of the bundle map: asks the thread or global memory manager for a 0x134-byte block (falling back to the CRT allocator with FUN_006fb950(0x134,4)… |
| `00a06260` | HashMap_PromotionalBundleData64_FreeNode | confirmed | Node deallocator of the bundle map: routes the node pointer to FreeMemory with alignment 4. |
| `00a06b40` | HashMap_PromotionalBundleData64_Destruct | confirmed | Eleven-byte destructor body: stores the HashMap vtable and calls the drain routine 00a06450 to free every node. |
| `00a06c00` | HashMap_PromotionalBundleData64_ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the HashMap vtable, drains the map with 00a06450 and frees this when the delete flag is set. |
| `00a06f10` | HashMap_PromotionalBundleData64_Destruct_B | confirmed | Second emission of the bundle-map destructor: stores the HashMap vtable, keeps the object pointer in an SEH local and calls the drain routine 00a06450. |
| `00a06450` | — | unknown | Drains the bundle map: for each of the 64 buckets, walks the node chain through the link at node+0x130, runs the node-data destructor FUN_01061530 on node+4 an… |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a088c0` | RedeemCode_HandleSelectOptionResult | confirmed | Small handler: when the record's status word at +8 is not 1 it logs 'Failed to select redeem code option, msg: %s.' through the object's first virtual, then al… |
| `00a05b80` | SetFlagByteAt0x1F4 | likely | Thirteen-byte method (ret 4) that stores the byte argument into the field at this+0x1F4. |
| `00a05ed0` | PairNode_ConstructKeyAndData | likely | Constructs a map node payload: copies the first dword of param_2 into this[0], then calls FUN_01061660 on this+4 with the dword stored at param_3 to construct … |
| `00a060b0` | NodePayload_InitWithKey | likely | Calls FUN_01061150 to default-construct the payload object at this, then copies the dword pointed to by param_2 into this+0x20 (the node key field). |
| `00a066b0` | ScalarDelete_a066b0 | likely | Scalar deleting destructor shape: calls FUN_01061220 (which rewrites the Loggable vtable of the sub-object at +0xc) and then frees this when the delete flag is… |
| `00a069d0` | FindNodeByStringKey | likely | Looks up a node in the map whose buckets start at this+0x31c (256 buckets): computes a case-insensitive string hash with FUN_00705b00, walks the bucket chain t… |
| `00a06a50` | UnregisterAndFreeObjectByKey | likely | Finds the map node for a name key with 00a069d0; if present, calls vtable slot 10 (offset 0x28) on the node value to obtain its id, unregisters that id from th… |
| `00a06be0` | HashListMap256_Drain | likely | Loops while the head pointer at this+8 is non-null, calling 00a06290 (this = map, node = head) to unlink and free each node. |
| `00a07700` | HashMap_PromotionalBundleData64_Insert | likely | Inserts into the bundle map: allocates a node through vtable slot 1, constructs its payload with 00a05ed0 from the two argument pointers, links the node at the… |
| `00a07d10` | HashListMap256_Insert | likely | Inserts into the datasource map: allocates a node through vtable slot 1, stores the value pointer from param_3 at node[0] and the precomputed hash from param_2… |
| `00a07ff0` | RegisterNamedDatasource | likely | Registers a datasource object by name: removes any existing entry with that name (00a06a50), hashes the name with the case-insensitive string hash FUN_00705b00… |
| `00a05d80` | — | unknown | SEH-framed function whose only work is calling FUN_01061530 with this+4; that callee installs the Loggable vtable at +0xc of the sub-object and constructs a Ha… |
| `00a06290` | — | unknown | Unlinks a node from a 256-bucket hash-list map: finds the node in the bucket chain selected by the hash stored at node+0xc, fixes the doubly linked head/tail l… |
| `00a06ab0` | — | unknown | Scans the 64-bucket map whose bucket array starts at this+0x208: for every node on every bucket chain (chain link at node+0x130) it calls FUN_01061370 with the… |
| `00a06b50` | — | unknown | Removes one node from the bundle map given the payload pointer: the key dword sits at payload-4 and selects the bucket (mask 0x3f, buckets from this+0xc); unli… |
| `00a08520` | — | unknown | Processes a code-redemption result record: logs 'Failed to redeem code, msg: %s.' through the object's first virtual when the status word at +8 is not 1; build… |

### `ui.coinstore`

**ClientInGamePurchaseManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b37e60` | ClientInGamePurchaseManager::NotifyProductSupportDescription | confirmed | Creates one Lua string parameter from input and invokes OnProductSystemSupportDescription. |
| `00b38d00` | ClientInGamePurchaseManager::HandlePaymentBrowserResult | confirmed | Reports payment-page result; result 0 logs CAS success and asks launcher to open Paypal/SMS URL, while 1/2/3/other log timeout/cancel/server/invalid result. |
| `00b37ce0` | — | unknown | Optionally runs maintenance when argument is set, invokes OnMarketplaceReady with that flag, logs result and calls manager virtual operation at offset 8. |
| `00b47320` | — | unknown | When the accepted-credit-card list [this+0x3ca34] is empty it logs 'OnOpenStoreScreen() Accepted credit card list is empty, sending update request.' and calls … |
| `00b47ca0` | — 🌐 | unknown | Virtual method at ClientInGamePurchaseManager vtable slot 5: on the success flag it releases and rebuilds the currency/locale member through FUN_00b2a7f0/FUN_0… |
| `00b49090` | — | unknown | When the session byte at [param_1+0x3a105] is set it calls FUN_00b3c610, tears down the session object at [param_1+0x3a100] with FUN_00b23fc0 and, if the gatew… |

**ClientInGamePurchaseStoreManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c39c80` | — | unknown | Initialises the in-game-purchase store manager: stores the data path at this+0x18, initialises the marketing-data updater (007e1270), builds the schema path fr… |

**CoinStoreDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf2440` | CoinStoreDataSource::Ctor | likely | Allocates and constructs eight datasource members using the BaseClient.CoinStore class-name strings, including CoinStoreItems, PlayerSell, SearchResults, Activ… |

**CoinStoreDataSource::ActiveFilters** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf0a30` | CoinStoreDataSource::ActiveFilters::FieldName | confirmed | Returns FilterId for index 0, OptionList for index 1, and <error> for other indices. |
| `00cf21a0` | CoinStoreDataSource::ActiveFilters::Dtor | confirmed | Restores the ActiveFilters vtables at the object and embedded-base offsets, destroys its fixed string through 00402140, then destroys its embedded GUI datasour… |
| `00cf2230` | CoinStoreDataSource::ActiveFilters::GetCount | confirmed | Returns the 32-bit value at this+0x338. |
| `00cf22e0` | CoinStoreDataSource::ActiveFilters::Ctor | confirmed | Constructs the GUI datasource base, installs ActiveFilters vtables, initializes a fixed string from the embedded class-name string, and clears the count at +0x… |
| `00cf2370` | CoinStoreDataSource::ActiveFilters::ScalarDeletingDtor | confirmed | Calls the ActiveFilters destructor body, frees the object when the low bit of the flag is set, and returns the object pointer otherwise. |
| `00cf1ee0` | — | unknown | Calls the base datasource field accessor, resolves a row from the selected datasource, obtains its map, then either releases the result for field 0 or formats … |

**CoinStoreDataSource::CoinStoreItems** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf0700` | CoinStoreDataSource::CoinStoreItems::Ctor | confirmed | Calls the base data-source initializer with its two arguments and installs the CoinStoreItems vtable at three object offsets. |
| `00cf0790` | CoinStoreDataSource::CoinStoreItems::Dtor | confirmed | Restores the CoinStoreItems vtables at three object offsets and invokes the shared base-data-source cleanup routine. |
| `00cf07f0` | CoinStoreDataSource::CoinStoreItems::FieldName | confirmed | Returns field labels for coin purchase price, sell price, item GUID list, no-sale, preview eligibility, tint group, and member purchase price; delegates unknow… |
| `00cf1130` | CoinStoreDataSource::CoinStoreItems::ScalarDeletingDtor | confirmed | Calls CoinStoreItems cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
| `00cf1b60` | CoinStoreDataSource::CoinStoreItems::GetFieldValue | confirmed | Switches on the requested field id, returns or writes values for cases 6, 0xd, 0x29, 0x2a, 0x2c, 0x2d, 0x2e and 0x2f, and delegates all other cases to 00c956c0. |

**CoinStoreDataSource::PlayerSell** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf0860` | CoinStoreDataSource::PlayerSell::Ctor | confirmed | Installs PlayerSell vtables at four object offsets and calls a member initialization helper. |
| `00cf0930` | CoinStoreDataSource::PlayerSell::FieldName | confirmed | Returns CanPreviewItem for index 0x1d and delegates every other index to the shared field-name helper. |
| `00cf0fd0` | ThisAdjustToPlayerSellDtor | confirmed | Subtracts 0x254 from ECX and jumps to 00cf1280; this is a this-adjusting thunk with no independent destructor body. |
| `00cf11f0` | CoinStoreDataSource::PlayerSell::Ctor | confirmed | Initializes the shared data-source base and installs PlayerSell vtables at four object offsets. |
| `00cf1280` | CoinStoreDataSource::PlayerSell::ScalarDeletingDtor | confirmed | Calls PlayerSell cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
| `00cf1d90` | CoinStoreDataSource::PlayerSell::GetFieldValue | confirmed | Returns one when row index is outside the active count; for field 4 computes available quantity and writes it, for field 0x1d writes an eligibility-dependent s… |
| `00cf08d0` | CoinStoreDataSource::PlayerSell::CanSellItem | likely | Rejects invalid items and records in state 1; when a sale limit is active, requires positive available count before returning true. |

**CoinStoreDataSource::SearchResults** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf0950` | CoinStoreDataSource::SearchResults::Ctor | confirmed | Constructs the shared CoinStoreItems base and installs SearchResults vtables at three object offsets. |
| `00cf09d0` | CoinStoreDataSource::SearchResults::Dtor | confirmed | Restores SearchResults vtables at three offsets and invokes the shared CoinStoreItems cleanup routine. |
| `00cf12a0` | CoinStoreDataSource::SearchResults::ScalarDeletingDtor | confirmed | Calls SearchResults cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |

**CoinStoreDataSource::TransactionHistory** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cf0a80` | CoinStoreDataSource::TransactionHistory::FieldName | confirmed | Maps field indices 0 through 8 to id, type, merchant GUID, item ID, quantity, tint ID, price, bought-back flag, and quantity remaining. |
| `00cf2250` | CoinStoreDataSource::TransactionHistory::Dtor | confirmed | Writes the TransactionHistory vtable at the base and embedded offsets, then destroys the embedded GUI datasource base through 0091bf60. |
| `00cf22c0` | CoinStoreDataSource::TransactionHistory::GetRowCount | confirmed | Returns the 32-bit value at this+0x22c. |
| `00cf2420` | CoinStoreDataSource::TransactionHistory::ScalarDeletingDtor | confirmed | Calls the TransactionHistory destructor body, frees the object when the low bit of the flag is set, and otherwise returns the object pointer. |
| `00cf1f70` | — | unknown | Checks the backing store and selected row, resolves a transaction object, then switches on field id 0 through 8 to copy or release field values; returns zero w… |
| `00cf2390` | — | unknown | Constructs the GUI datasource base, installs TransactionHistory vtables, stores its selected datasource pointer, sets the row mode to 2, clears the count, and … |

**CoinStoreScreen** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bffe30` | CoinStoreScreen::BuyItem | likely | Requires three truthy Lua arguments and an available manager, extracts three values, calls 00c77a30, then pushes its result. |
| `00c002d0` | CoinStoreScreen::RequestStoreSort | likely | Requires two Lua arguments and an available manager, extracts the first value and second string, calls 00c77ca0, then pushes its result. |
| `00c00730` | CoinStoreScreen::RequestBuyBackItem | likely | Requires one or two Lua arguments and an available manager, extracts item and optional value, calls 00c76c60, and pushes its result. |
| `00c00b50` | CoinStoreScreen::RebuildFilteredList | likely | When one Lua string argument and a store manager exist, copies the string into a temporary object, calls 00c75bd0 to rebuild filtered items, and pushes the res… |
| `00c00a50` | CoinStoreScreen::RequestGiftPurchase | speculative | Validates four Lua arguments, parses one string as a 64-bit id, builds a temporary item list from three values, calls 00c79370, and pushes its result. |

**GameCommerce::ImageDataDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c39b90` | ImageDataDefinition::ReadImageFields | likely | Reads image/graphics fields of a definition from a data source: getter 00d63820(this=source,"image",row) result is assigned (IStringAssign 00402ce0) into the I… |

**GameCommerce::StoreDefinition<AppStoreBundleDefinition,512>** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3abc0` | GameCommerce::StoreDefinition::~StoreDefinition | confirmed | Destructor of the store definition: stores the StoreDefinition vtable, frees every bundle entry of the chain at this+0x3c through 00b289d0 until the head is nu… |
| `00c3ad40` | StoreDefinition::FindBundleById | confirmed | Finds a bundle entry by id: hashes the id with (id & 0x7ff) into the 2048 bucket heads at this+0x48, walks the chain through entry+0x12c and compares entry+0x1… |
| `00c3ad70` | StoreDefinition::RemoveBundleById | confirmed | Removes a bundle entry by id: performs the same 2048-bucket lookup (bucket = id & 0x7ff at this+0x48, chain at entry+0x12c, id at entry+0x128) and, when found,… |
| `00c3b120` | StoreDefinition::ScalarDeletingDtor | confirmed | Deleting destructor of the store definition: runs the destructor 00c3abc0 and, when bit 0 of the flag byte is set, releases the object through FreeMemory(this,… |
| `00c3b2f0` | GameCommerce::StoreDefinition::StoreDefinition | confirmed | Constructor of the store definition: stores the BaseDefinition then StoreDefinition vtables, seeds the store id at +4 with the default sentinel DAT_01bedd14, z… |
| `00c3ac60` | StoreDefinition::AreBundleContentsAvailable | likely | Predicate over a store definition: false when the id at this+4 equals the default sentinel DAT_01bedd14; otherwise true only when the bundle count at this+0x44… |
| `00c3acb0` | StoreDefinition::HasBundles | likely | Small predicate: returns 1 only when the store id at this+4 differs from the default sentinel DAT_01bedd14 and the bundle count at this+0x44 is positive; other… |
| `00c3acd0` | StoreDefinition::InitBundleEntry | likely | Initialises one bundle-map entry: calls 010784e0 with the source bundle (ECX = the entry, copied from an AppStoreBundleDefinition source object) and then store… |
| `00c3b370` | StoreDefinition::InsertBundleFromDefinition | likely | Inserts a bundle into the store's bundle map: allocates one entry through the map's vtable slot 1 (adjustor 00c3a030 -> pool alloc), fills it with 00c3acd0(key… |
| `00c3adb0` | — | unknown | Builds a debug string for the store: creates a temporary IString, appends "(bundle), " for every bundle entry in the map list at this+0x3c by calling each entr… |
| `00c3dc40` | — | unknown | Insert-or-update of a bundle in the store's bundle map: looks the bundle up by the id at bundle+0x9c in the 2048-bucket map at this+0x48; when absent it insert… |

**GameCommerce::StoreShortcutDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c3b530` | ReadShortcutRowFields | likely | Reads one store-shortcut row from a data source: 00d637a0(source,"shortcut_id") -> this+4, "shortcut_type" -> this+8, "shortcut_value" -> this+0xc, "store_id" … |

**InGamePurchaseDataSource::AcceptedCreditCardTypes** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4850` | — | unknown | Bounds-checks the selected credit-card type row and writes its fields for selectors 0 through 8. |

**InGamePurchaseDataSource::CountryCodes** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4990` | — | unknown | Bounds-checks the selected country row and writes one of its stored fields for selectors 0 through 4. |

**InGamePurchaseDataSource::CreditCardPayment** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4450` | CreditCardPayment::GetValue | confirmed | Resolves a selected payment record and handles field selectors 0 through 3, writing stored values or string fields to the requested output. |

**InGamePurchaseDataSource::GiftOrderEntries** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a43a0` | GiftOrderEntries::GetValue | confirmed | Resolves the requested gift-order entry, checks its index against the record count, and writes one of the supported entry fields to the output. |

**InGamePurchaseDataSource::GiftOrderHistory** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a41f0` | GiftOrderHistory::GetValue | confirmed | Resolves the selected order-history record, dispatches selectors 0 through 5, copies stored values, formats the elapsed-time field, and returns whether the rec… |

**InGamePurchaseDataSource::MerchantSellBundles** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a6490` | MerchantSellBundles::GetValue | confirmed | Requires the backing client data, looks up the row key in a 1024-bucket chain for selector 0x22, writes the found value or zero, and delegates other selectors … |

**InGamePurchaseDataSource::StateCodes** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4a80` | — | unknown | Bounds-checks the selected state row and writes its first or second stored field for selectors 0 through 2. |

**InGamePurchaseDataSource::StationCashPriceList** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4540` | — | unknown | Bounds-checks the selected price row, writes row values for selectors 0 through 6, tests whether sale price is positive, and formats selected price fields. |

**InGamePurchaseDataSource::StoreBillboardPanels** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3f30` | StoreBillboardPanels::GetValue | confirmed | Resolves the selected panel row, indexes a panel record, and handles its supported field selectors, including a converted value for selector 7. |

**InGamePurchaseDataSource::StoreBillboards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3e80` | StoreBillboards::GetValue | confirmed | Checks the backing data object, resolves the selected billboard entry, and handles the supported field selectors before returning success or failure. |

**InGamePurchaseDataSource::StoreBundleActiveFilters** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4110` | StoreBundleActiveFilters::GetValue | confirmed | Resolves the requested filter entry and backing bundle, then returns a selected value or writes the ordered filter identifiers into the output. |
| `009a40a0` | — | unknown | Calls the base row-reset virtual, zeros its count field, resolves the store bundle object, visits its filter array, and increments the count for each visited i… |

**InGamePurchaseDataSource::StoreBundleActiveSort** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a41a0` | StoreBundleActiveSort::GetValue | confirmed | Resolves the StoreBundles data source, writes its selected sort field for selector zero, and returns success when the source exists. |

**InGamePurchaseDataSource::StoreBundleCategories** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a37b0` | StoreBundleCategoriesCtor | confirmed | Constructs the internal data-source base, installs the StoreBundleCategories vtable in both object subobjects, stores its supplied member, and initializes its … |
| `009a3840` | StoreBundleCategoriesDeletingDtorAdjustor | confirmed | Subtracts 0x38 from this and jumps to 009a6c10, the StoreBundleCategories scalar deleting destructor. |
| `009a6710` | StoreBundleCategoriesDestructor | confirmed | Restores the category data-source vtables, recycles the owned node, destroys the embedded category-node list and tree root, restores the base data-source vtabl… |
| `009a67c0` | StoreBundleCategories::RefreshTree | confirmed | Recycles the prior tree, checks backing category data and count, creates root and category nodes, indexes category records, recursively builds descendants, and… |
| `009a6c10` | StoreBundleCategoriesScalarDeletingDtor | confirmed | Calls the StoreBundleCategories destructor, tests deletion flag bit 0, frees this when set, and otherwise returns this. |

**InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a38a0` | StoreBundleCategories::GetCategoryValue | confirmed | For field selectors 0, 1, and 2, writes node fields or a converted value to the output; unsupported selectors write -1 and return zero. |

**InGamePurchaseDataSource::StoreBundleGroupContents** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3a50` | StoreBundleGroupContents::Refresh | confirmed | Removes current rows, marks the data source dirty, resolves the backing group record, and appends each contained row key with the current source value. |

**InGamePurchaseDataSource::StoreBundleGroups** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3940` | StoreBundleGroups::GetValue | confirmed | Looks up the selected bundle-group record, switches on field selector 0 through 3, and writes the selected value or conversion to the output. |
| `009a6c30` | StoreBundleGroupsDestructor | confirmed | Restores StoreBundleGroups vtables in its embedded subobjects and calls the GuiUserDataTable destructor for the owned table. |
| `009a6cc0` | StoreBundleGroups::GetField22C | confirmed | Returns the dword stored at this plus 0x22C. |
| `009a6ce0` | StoreBundleGroupsDeletingDtorAdjustor | confirmed | Subtracts 0x254 from this and jumps to 009a7890, the StoreBundleGroups scalar deleting destructor. |

**InGamePurchaseDataSource::StoreBundleOfTheDay** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a63a0` | StoreBundleOfTheDay::GetValue | confirmed | Requires the backing client data, returns three stored fields for selectors 0x22 through 0x24, checks selector 0x25, and delegates other selectors to the share… |

**InGamePurchaseDataSource::StoreBundleVipLevelContents** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3b10` | StoreBundleVipLevelContents::Refresh | confirmed | Removes current rows, marks the data source dirty, and appends rows whose stored VIP-level field equals this data source selected value. |

**InGamePurchaseDataSource::StoreBundles** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3650` | StoreBundlesRefreshDerivedEntries | confirmed | When its refresh flag is set, clears the existing entry map, finds the matching store bundle record, appends each derived record key with the current source va… |
| `009a56d0` | StoreBundles::GetValue | confirmed | Handles selectors 0 through 0x21, returning stored bundle fields, formatted strings, flags, availability predicates, and values derived from associated item re… |
| `009a6650` | StoreBundles::ResolveAndRequestBundleRecord | confirmed | Rebuilds filtered rows when dirty, resolves the requested bundle record through its selected object, requests missing item data when the record needs it, and r… |
| `009a66f0` | StoreBundles::GetCount | confirmed | Rebuilds filtered rows when dirty, then returns the value stored at this plus 0x24C. |

**InGamePurchaseDataSource::StoreItemOfTheDay** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a62b0` | StoreItemOfTheDay::GetValue | confirmed | Requires the backing client data, returns three stored fields for selectors 0x22 through 0x24, checks selector 0x25, and delegates all other selectors to the s… |

**InGamePurchaseDataSource::StorePortalCategories** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a3cd0` | StorePortalCategories::GetValue | confirmed | Looks up the selected portal-category record, dispatches field selectors 0 through 7, writes the selected value or string conversion, and returns whether the r… |

**InGamePurchaseDataSource::SubscriptionPriceList** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a4680` | — | unknown | Bounds-checks the selected subscription row and handles selectors 0 through 10 by returning stored strings, numbers, flags, or formatted price fields. |

**ItemRecord** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bffd20` | ItemRecord::Ctor | confirmed | Installs ItemRecord::vftable and clears fields at offsets 4, 8, 16, and 20. |
| `00c00490` | ItemRecord::DtorBody | confirmed | Restores ItemRecord::vftable at the start of the record and returns. |
| `00c00520` | ItemRecord::DtorBody | confirmed | Restores ItemRecord::vftable at the start of the record and returns. |
| `00c006d0` | ItemRecord::ScalarDeletingDtor | confirmed | Restores ItemRecord::vftable and frees this only when flag bit 0 is set; otherwise returns this. |
| `00c00680` | CoinStoreItemRecord::Ctor | speculative | Calls 00bffd20 on a temporary record and returns the supplied pointer. |

**SoeUtil::List<CoinStoreItemRecord,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c00c60` | CoinStoreItemList::DtorBody | confirmed | Restores the list vtable and repeatedly removes the current head node through 00c00840 until the list is empty. |
| `00c00c90` | CoinStoreItemList::ScalarDeletingDtor | confirmed | Restores the list vtable, removes all nodes through 00c00840, then frees this only when flag bit 0 is set. |
| `00c00cd0` | CoinStoreItemList::Ctor | confirmed | Initializes four endpoints from sentinel globals, installs the list vtable at offset 16, and clears three node/count fields. |
| `00c00d50` | CoinStoreItemList::DtorBody | confirmed | Restores the embedded list vtable at offset 16 and repeatedly removes its head node at offset 20 through 00c00840 until empty. |

**(no class)** (67)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a51b0` | RefreshMannequinStoreItems | confirmed | Resolves BaseClient.InGamePurchase.MannequinStoreItems, clears existing derived entries, rebuilds rows through the tree traversal helper, and notifies the reso… |
| `009a5230` | RefreshStoreItemSearchResults | confirmed | Resolves BaseClient.InGamePurchase.StoreItemSearchResults, clears current rows, traverses the supplied linked keys to rebuild rows, and notifies the data sourc… |
| `009a5270` | RefreshMerchantSellBundles | confirmed | Resolves BaseClient.InGamePurchase.MerchantSellBundles, clears current rows, populates them from the supplied linked list, and notifies the data source. |
| `009a52e0` | BuildStoreItemRows | confirmed | Walks stored item records, resolves each item definition, formats ItemId, Quantity, Name, ImageSetId, TintValue, and IsTintable fields, and releases temporary … |
| `00ad4a60` | ShowMarketplace | confirmed | When manager offset +0x14 is set, builds two Lua parameters, pushes constant 0x53, invokes MarketplaceHandler:Show, and releases the parameter array. |
| `00b35320` | CallStoreItemPreview | confirmed | Invokes StoreItemPreview API with supplied args, creates Lua parameters for preview kind and item value, then dispatches the corresponding pet, bundle, mannequ… |
| `00b37360` | NotifyMarketplaceEnabled | confirmed | Stores marketplace-enabled byte, builds a Lua boolean parameter and invokes OnMarketplaceEnabled. |
| `00b377e0` | NotifySelectedStoreBundle | confirmed | Looks up bundle in 1024-bucket table, resolves SelectedStoreBundle datasource, clears old category nodes, stores bundle value and invokes virtual callback. |
| `00b37850` | SelectStoreBundle | confirmed | Builds temporary id set, updates selected-bundle datasource, looks up bundle and on success invokes OnStoreBundleContents; logs failure and destroys temporary … |
| `00b37a70` | CallDisplayStoreBundleIds | confirmed | Creates one Lua boolean argument and invokes MarketplaceHandler:DisplayStoreBundleIds. |
| `00b39340` | NotifyUpdatedStoreItem | confirmed | Signals bundle-content availability, releases matching cache payload, builds Lua key/value parameters and invokes OnUpdatedStoreItem. |
| `00c76e70` | CompleteCoinStoreRequest | confirmed | Calls FUN_0070ec40 and FUN_009773f0 on a local flag built from param_2, reads the pending state at this+0xd454: state 1 sets the byte at this+0xdc24 when param… |
| `00c77030` | SellItem | confirmed | Thin wrapper: calls FUN_00c76dd0 with the two session merchant GUID globals DAT_01beea08 and DAT_01beea0c followed by its two arguments. |
| `00c774a0` | DisplayStoreItems | confirmed | Calls FUN_00c754a0 with the page list (param_3) to refresh the store list, then fills the two marketplace widget slots: slot 0 gets a 1 (after releasing the pr… |
| `00c77630` | DisplayStoreItemCategory | confirmed | Fills three marketplace widget slots: each slot first releases its previous payload when its tag is 3, 6, 7 or 8 (through the slot vtable entry), then is marke… |
| `00c77810` | DisplayStoreItemGroup | confirmed | Fills three marketplace widget slots with the same pattern as DisplayStoreItemCategory (release the old payload for tags 3, 6, 7 and 8, then mark 1 and store p… |
| `009a2f20` | LookupMerchantSellBundlesDataSource | likely | Hashes the supplied key, searches a 1000-bucket table, and returns the stored object only when its RTTI casts from GuiDataSource to MerchantSellBundles. |
| `009a31d0` | CreateCategoryTreeNode | likely | Pops a node from the free list, returns zero when allocation fails, and otherwise constructs a category-tree node from the supplied identifiers and fields. |
| `009a36f0` | RemoveTreeEntriesByMode | likely | Reads a mode field, selects one of several predicate functions, and passes the chosen predicate with the tree to the tree-removal helper. |
| `009a3bf0` | CopyRecordAndRebuildOneRow | likely | Copies selected fields from the supplied record, drains current rows, reads the current key and value fields, appends one row, and marks the dirty byte. |
| `009a3c60` | CopyDefinitionAndRebuildOneRow | likely | Copies selected fields from the supplied definition, drains current rows, reads the current key and value fields, appends one row, and marks the dirty byte. |
| `009a51f0` | ApplyRecordUpdateAndNotify | likely | Calls the record-copy and row-refresh helper, then invokes the virtual notification method on the object at this plus 0x10. |
| `009a5210` | ApplyDefinitionUpdateAndNotify | likely | Calls the definition-copy and row-refresh helper, then invokes the virtual notification method on the object at this plus 0x14. |
| `009a52b0` | SetKeyedValueAndNotify | likely | Updates the keyed value through the map helper and invokes the virtual notification method on the object at this plus 0x54. |
| `009a5d30` | RebuildFilteredStoreBundleRows | likely | Clears the prior selected list, iterates current store rows, resolves bundle records, applies requirement checks and selected predicates, appends accepted defi… |
| `009a60d0` | ClearCategoryTreeContents | likely | Recursively clears child-tree contents, releases each node through its owner callback, returns nodes to the free list, then clears the category-node list and i… |
| `009a6280` | RecycleCategoryTreeNode | likely | If the node child field is non-null, clears its child contents, releases the node resource, invokes its virtual destructor, and pushes it onto the owner free l… |
| `00b37470` | NotifyPaymentPageResult3 | likely | Reads payment-page result, stores code 3, and if returned token matches configured token dispatches OnPaymentPageLoaded with a Lua parameter. |
| `00c39e20` | RemoveDefinitionFromKeyedTree | likely | Unregisters one definition object: walks the binary tree rooted at this+0x2bc comparing the key at object+0x9c, and on an exact match calls 00c36fe0 on the tre… |
| `00c3af60` | PruneManagerEntriesWithZeroCount | likely | Maintenance sweep over two manager lists: for the list at this+0xdb4 (next at node+0x48) it frees every node whose counter at node+0x34 is < 1 through 00b291a0… |
| `00c3b020` | InsertIndexedNode190 | likely | Appends one definition node to a 64-bucket container: allocates the node through the container's vtable slot 1, initialises it from param_2 with 00c38f80, link… |
| `00c3b0d0` | ClearListAt8 | likely | Drains the singly linked list at this+8: while the head is non-null it calls 00c38be0(this, head), which frees the node and advances the head. |
| `00c3b240` | DestroySetMemberAt4 | likely | SEH-framed stub with no direct callers: calls 009b40e0 with ECX = this+4, i.e. destroys the SoeUtil::Set member that sits at offset 4 of its owning object (the… |
| `00c3b290` | EntryCtor | likely | Constructor-like initialiser of a small keyed entry: copies the dword pointed to by the argument into this+0, then constructs the SoeUtil::Set member at this+4… |
| `00c3b3f0` | InsertIndexedNode4C | likely | Appends one definition node to a 64-bucket container: allocates it via the container's vtable slot 1, initialises it with 00c350e0(param_2), links it at the ta… |
| `00c3b490` | InsertIndexedNode5C | likely | Appends one definition node to a 16-bucket container: allocates it via the container's vtable slot 1, initialises it with 00c35140(param_2), links it at the ta… |
| `00c3b650` | DestroyStoreDefinitionAt0 | likely | SEH-framed stub with no direct callers: calls 00c3abc0 with ECX unchanged (i.e. destroys the StoreDefinition<AppStoreBundleDefinition,512> sub-object at offset… |
| `00c3b6c0` | RunMaintenanceSweeps | likely | Maintenance pass over the manager's definition containers: runs 00c3af60 on the sub-object at this+0x34390 (frees entries whose counter at +0x34 is < 1, return… |
| `00c3b790` | RemoveContainerEntryById | likely | Removes one indexed entry from the manager: walks the chain at this+0xc094 (key at entry+0x2806c, next at entry+0x28070) to find the store with the requested i… |
| `00c3dbb0` | InsertBucketedEntry1024 | likely | Inserts an indexed entry into a 1024-bucket container: allocates the entry through the container's vtable slot 1, constructs it with 00c3b290(param_2) (key cop… |
| `00c3dde0` | EntryCtorWithKeyAt2806C | likely | Constructor wrapper for the large store-entry object: runs the StoreDefinition<AppStoreBundleDefinition,512> constructor 00c3b2f0 on this and then stores the d… |
| `00c77060` | SellItemForMerchant | likely | Calls FUN_00cf16c0 to read an 8-byte merchant GUID into a local buffer, then calls FUN_00c76dd0 with the two stored words and the two arguments. |
| `00c77a30` | BuyItemByItemId | likely | Calls FUN_00c77230 (BuyItem) with the two session merchant GUID globals DAT_01beea08 and DAT_01beea0c, the constant 0xffffffff (no record) and the three argume… |
| `00c77b80` | RequestStoreSearchInit | likely | Calls FUN_00c710e0 (search engine ready check). When it returns true, sets the pending state at this+0xd454 to 1, clears this+0xd458 and sets this+0xd45c to th… |
| `00c77db0` | BuyItemWithRecord | likely | Builds a four-dword argument block from the three arguments, looks up a record through FUN_00cf10d0 (stored in the block), reads the 8-byte merchant GUID throu… |
| `00c78c10` | SetPlayerCoin | likely | Pushes the script event context: calls FUN_0070ec40 and FUN_0070c580 with the CoinStore name, raises the HandlerMerchant event through FUN_00c722f0, then reads… |
| `00cf10c0` | GetStoreFilterState | likely | Returns the dword at offset 600 from the object referenced by this+0x10. |
| `00cf10d0` | GetCoinStoreRecordMap | likely | Returns the dword at offset 0x2f4 from the object referenced by this+0x18. |
| `00cf1150` | ResolveCoinStoreRecordForItem | likely | Finds an item record in a 4096-bucket map; returns the input value unless its availability flag and prerequisite record apply, then substitutes the first confi… |
| `00cf1e70` | RefreshCoinStoreDatasourceRows | likely | Calls the datasource begin callback, clears the row count, resolves the selected datasource by key, walks its linked rows at +0x52c0/+0xc8, invokes a callback … |
| `00cf2120` | RefreshCoinStoreFilteredRows | likely | Calls 00cf1ac0 to rebuild the filtered linked list, then invokes the virtual callback at the object's vtable offset 0x10. |
| `00cf2140` | RefreshCoinStoreDatasource | likely | Looks up the datasource key held through *(this+0x10)+600, calls 00c96790, and repopulates rows through 00cf1e70. |
| `00cf2170` | RefreshCoinStoreDatasourceAndNotify | likely | Looks up the datasource key held through *(this+0x10)+600, calls 00c93340, then repopulates rows through 00cf1e70. |
| `00b394e0` | NotifyAcceptedCreditCardsUpdated | speculative | Refreshes accepted-card datasource, then invokes OnUpdatedAcceptedCreditCards with supplied flag and manager card-type count. |
| `00cf1050` | FindBrokerRecordByKey | speculative | Searches the bucket at key modulo 15, follows node links at offset 0x14, and returns whether a node's key at offset 0x10 matches. |
| `00cf1100` | GetCoinStoreHistoryList | speculative | Returns the address of the structure at this+0x18 plus offset 0x2e8. |
| `009a3520` | — | unknown | Searches a nested 32-bucket table for the requested key, resolves an object through the global data-source lookup, updates its key and dirty byte when changed,… |
| `009a4dc0` | — | unknown | Clears the typed StoreBundles reference list, frees its inline-eight array storage, then releases each non-null interface field from indices 0 through 0x23 and… |
| `009a5eb0` | — | unknown | Finds category records by key, resolves their label and icon fields, creates typed category nodes, updates node indexes and heap order, recursively builds desc… |
| `00b37570` | — | unknown | Reads alternate payment-page result, stores code 4, and if token matches configured token dispatches OnPaymentPageLoaded with a Lua parameter. |
| `00b37770` | — | unknown | Looks up bundle id in map; when found notifies TooltipData, otherwise logs failed store/bundle lookup. |
| `00b39660` | — | unknown | Finds entry with lowest priority field at +0x18, stores it active, and invokes CallHandler:ShowActiveSale with six values; if none, clears active sale and pass… |
| `00c39e70` | — | unknown | Sweeps the definition list whose head is this+0x359a8 (next link at node+0x4c, node data at +0x38): for every node whose predicate 00c387f0(this,node+0x38) ret… |
| `00c39ed0` | — | unknown | Sweeps the definition list at this+0x36094 (next link at node+0x190); for each node it iterates the embedded array at node+0x10 (0x14 entries of stride 0x5c) a… |
| `00c3b720` | — | unknown | Callback for available bundle content: when 00c388b0(DAT_01b6ea10) resolves an entry and 00c36f80() confirms it, it calls 00b246a0() and, if that returns non-n… |
| `00c77a60` | — | unknown | Calls FUN_00c73020 to build a temporary Set, gets the filter object from FUN_00cf10c0 and builds the filter list through FUN_00c74840. Unless the pending state… |
| `00c77ca0` | — | unknown | Calls FUN_00c73020 to build a temporary Set, builds the sort criteria list through FUN_00c74840 from param_3, then unless the pending state at this+0xd454 is n… |

## Source shards

[[S0236|S0236]] (1), [[S0240 009105e0-009161d0|S0240]] (2), [[S0266 00995720-0099c9a0|S0266]] (35), [[S0267 0099ca10-009a1c60|S0267]] (29), [[S0268 009a1ca0-009a6ce0|S0268]] (51), [[S0269 009a6cf0-009aceb0|S0269]] (102), [[S0286 00a05a50-00a0c390|S0286]] (39), [[S0323 00ad3270-00ad83b0|S0323]] (1), [[S0337 00b1b5a0-00b21f20|S0337]] (35), [[S0338 00b21fb0-00b29620|S0338]] (104), [[S0339 00b29890-00b2eaf0|S0339]] (21), [[S0340 00b2ec20-00b33a50|S0340]] (35), [[S0341 00b33bc0-00b39af0|S0341]] (36), [[S0342 00b39c10-00b3f310|S0342]] (21), [[S0343 00b3f370-00b44f00|S0343]] (22), [[S0344 00b44fa0-00b4b360|S0344]] (36), [[S0382 00bfcfe0-00c056a0|S0382]] (23), [[S0388 00c2df40-00c34100|S0388]] (1), [[S0389 00c34110-00c39a40|S0389]] (56), [[S0390 00c39b90-00c3de40|S0390]] (30), [[S0391 00c3df10-00c423a0|S0391]] (45), [[S0400 00c6df10-00c73b60|S0400]] (40), [[S0401 00c73bb0-00c7a5b0|S0401]] (64), [[S0415 00cc1a00-00cc6bc0|S0415]] (7), [[S0423 00ceb9b0-00cf13e0|S0423]] (20), [[S0424 00cf1430-00cf68d0|S0424]] (17), [[S0445 00d54b90-00d59cb0|S0445]] (43), [[S0446|S0446]] (65), [[S0447|S0447]] (39), [[S0448 00d632e0-00d69450|S0448]] (86), [[S0449 00d694d0-00d6f0a0|S0449]] (73), [[S0450|S0450]] (6), [[S0451|S0451]] (46), [[S0452|S0452]] (14), [[S0579|S0579]] (12), [[S0581|S0581]] (14), [[S0606|S0606]] (14), [[S0781|S0781]] (29)
