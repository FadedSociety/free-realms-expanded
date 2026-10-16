# System: ui.coinstore

162 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (81)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a3650` | `StoreBundlesRefreshDerivedEntries` | InGamePurchaseDataSource::StoreBundles | Refreshes derived StoreBundles entries after the backing records change. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a37b0` | `StoreBundleCategoriesCtor` | InGamePurchaseDataSource::StoreBundleCa… | Constructs the StoreBundleCategories data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3840` | `StoreBundleCategoriesDeletingDtorAdjustor` | InGamePurchaseDataSource::StoreBundleCa… | Adjustor thunk for the StoreBundleCategories deleting destructor. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a38a0` | `StoreBundleCategories::GetCategoryValue` | InGamePurchaseDataSource::StoreBundleCa… | Returns one selected category-tree node field to the data-source caller. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3940` | `StoreBundleGroups::GetValue` | InGamePurchaseDataSource::StoreBundleGr… | Returns one selected StoreBundleGroups row value. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3a50` | `StoreBundleGroupContents::Refresh` | InGamePurchaseDataSource::StoreBundleGr… | Rebuilds StoreBundleGroupContents rows from the selected group. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3b10` | `StoreBundleVipLevelContents::Refresh` | InGamePurchaseDataSource::StoreBundleVi… | Rebuilds the VIP-level contents data source from the matching backing records. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3cd0` | `StorePortalCategories::GetValue` | InGamePurchaseDataSource::StorePortalCa… | Returns one selected portal-category row value. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3e80` | `StoreBillboards::GetValue` | InGamePurchaseDataSource::StoreBillboar… | Returns the selected StoreBillboards data-source value. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3f30` | `StoreBillboardPanels::GetValue` | InGamePurchaseDataSource::StoreBillboar… | Returns a selected StoreBillboardPanels row value. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4110` | `StoreBundleActiveFilters::GetValue` | InGamePurchaseDataSource::StoreBundleAc… | Provides row fields for the active-filter data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a41a0` | `StoreBundleActiveSort::GetValue` | InGamePurchaseDataSource::StoreBundleAc… | Returns the selected sort value for the active-sort data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a41f0` | `GiftOrderHistory::GetValue` | InGamePurchaseDataSource::GiftOrderHist… | Returns one selected GiftOrderHistory field for the UI data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a43a0` | `GiftOrderEntries::GetValue` | InGamePurchaseDataSource::GiftOrderEntr… | Returns a selected GiftOrderEntries row value. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4450` | `CreditCardPayment::GetValue` | InGamePurchaseDataSource::CreditCardPay… | Returns a selected CreditCardPayment data-source value. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a51b0` | `RefreshMannequinStoreItems` |  | Refreshes the mannequin store item data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a5230` | `RefreshStoreItemSearchResults` |  | Refreshes the store item search-results data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a5270` | `RefreshMerchantSellBundles` |  | Refreshes the merchant-sell bundle data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a52e0` | `BuildStoreItemRows` |  | Builds item rows for the store-item tooltip data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a56d0` | `StoreBundles::GetValue` | InGamePurchaseDataSource::StoreBundles | Shared GetValue implementation for StoreBundles, StoreBundleGroupContents, and StoreBundleVipLevelContents. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a62b0` | `StoreItemOfTheDay::GetValue` | InGamePurchaseDataSource::StoreItemOfTh… | Returns item-of-the-day data-source fields and shared bundle fields. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a63a0` | `StoreBundleOfTheDay::GetValue` | InGamePurchaseDataSource::StoreBundleOf… | Returns bundle-of-the-day data-source fields and shared bundle fields. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6490` | `MerchantSellBundles::GetValue` | InGamePurchaseDataSource::MerchantSellB… | Returns MerchantSellBundles row fields. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6650` | `StoreBundles::ResolveAndRequestBundleRecord` | InGamePurchaseDataSource::StoreBundles | Shared implementation in the StoreBundles, group-content, VIP-content, item-of-the-day, bundle-of-the-day, and merchant-sell vtables. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a66f0` | `StoreBundles::GetCount` | InGamePurchaseDataSource::StoreBundles | Shared count accessor in the StoreBundles, group-content, VIP-content, and merchant-sell vtables. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6710` | `StoreBundleCategoriesDestructor` | InGamePurchaseDataSource::StoreBundleCa… | Destructor body for StoreBundleCategories. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a67c0` | `StoreBundleCategories::RefreshTree` | InGamePurchaseDataSource::StoreBundleCa… | Refreshes the StoreBundleCategories tree from the backing category data. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6c10` | `StoreBundleCategoriesScalarDeletingDtor` | InGamePurchaseDataSource::StoreBundleCa… | Scalar deleting destructor for StoreBundleCategories. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6c30` | `StoreBundleGroupsDestructor` | InGamePurchaseDataSource::StoreBundleGr… | Destructor body shared by StoreBundleGroups vtable variants. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6cc0` | `StoreBundleGroups::GetField22C` | InGamePurchaseDataSource::StoreBundleGr… | Small StoreBundleGroups virtual accessor. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6ce0` | `StoreBundleGroupsDeletingDtorAdjustor` | InGamePurchaseDataSource::StoreBundleGr… | Adjustor thunk for a StoreBundleGroups deleting destructor. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `00ad4a60` | `ShowMarketplace` |  | Shared card manager method that opens MarketplaceHandler. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00b35320` | `CallStoreItemPreview` |  | Bridges store preview requests to UI scripts. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b37360` | `NotifyMarketplaceEnabled` |  | Publishes marketplace state to scripts. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b377e0` | `NotifySelectedStoreBundle` |  | Updates selected bundle script datasource. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b37850` | `SelectStoreBundle` |  | Selects bundle and publishes contents to UI scripts. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b37a70` | `CallDisplayStoreBundleIds` |  | Requests script display of bundle ids. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b37e60` | `ClientInGamePurchaseManager::NotifyProductSupportDescription` | ClientInGamePurchaseManager | Publishes payment-system support description. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b38d00` | `ClientInGamePurchaseManager::HandlePaymentBrowserResult` | ClientInGamePurchaseManager | Handles local payment browser launch outcome. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b39340` | `NotifyUpdatedStoreItem` |  | Publishes changed store item to scripts. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00bffd20` | `ItemRecord::Ctor` | ItemRecord | Initializes an empty item record used by the coin store list. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00490` | `ItemRecord::DtorBody` | ItemRecord | Runs base cleanup for an ItemRecord node. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00520` | `ItemRecord::DtorBody` | ItemRecord | Runs base cleanup for an ItemRecord node. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c006d0` | `ItemRecord::ScalarDeletingDtor` | ItemRecord | Implements the scalar deleting destructor for ItemRecord. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00c60` | `CoinStoreItemList::DtorBody` | SoeUtil::List<CoinStoreItemRecord,-1> | Destroys the linked coin-store item list and its nodes. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00c90` | `CoinStoreItemList::ScalarDeletingDtor → SoeUtil::List<CoinStoreItemRecord,-1>::…` | SoeUtil::List<CoinStoreItemRecord,-1> | Implements scalar deleting destruction for the coin-store item list. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00cd0` | `CoinStoreItemList::Ctor → SoeUtil::List<CoinStoreItemRecord,-1>::Ctor` | SoeUtil::List<CoinStoreItemRecord,-1> | Initializes the coin-store item list embedded in packet/request objects. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00d50` | `CoinStoreItemList::DtorBody → SoeUtil::List<CoinStoreItemRecord,-1>::DtorBody` | SoeUtil::List<CoinStoreItemRecord,-1> | Destroys an embedded coin-store item list. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c3abc0` | `GameCommerce::StoreDefinition::~StoreDefinition` | GameCommerce::StoreDefinition<AppStoreB… | Teardown of one store (storeId/nameId/descriptionId plus its bundle map) of the in-game purchase store manager. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3ad40` | `StoreDefinition::FindBundleById` | GameCommerce::StoreDefinition<AppStoreB… | Lookup used by the store UI/manager to resolve a bundle definition from a storeBundleId. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3ad70` | `StoreDefinition::RemoveBundleById` | GameCommerce::StoreDefinition<AppStoreB… | Bundle removal for the update path of the purchase store (slot 7 of the store definition vtable). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b120` | `StoreDefinition::ScalarDeletingDtor` | GameCommerce::StoreDefinition<AppStoreB… | Vtable slot 2 of the store definition, used when a store is deleted through its own vtable. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b2f0` | `GameCommerce::StoreDefinition::StoreDefinition` | GameCommerce::StoreDefinition<AppStoreB… | Creates one store entry of the in-game purchase store manager before its marketing row is read into it. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c76e70` | `CompleteCoinStoreRequest` |  | Completes an asynchronous store search request (initialise, filter or sort) and notifies the Lua-side HandlerCoinStore with the matching response event. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77030` | `SellItem` |  | Lua-facing sell entry point that builds the sell (buy-from-client) request for the session merchant. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c774a0` | `DisplayStoreItems` |  | Pushes a store item page into the marketplace widget and notifies the Lua handler with the DisplayStoreItems event. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77630` | `DisplayStoreItemCategory` |  | Pushes a store category page into the marketplace widget and notifies the Lua handler. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77810` | `DisplayStoreItemGroup` |  | Pushes a store item-group page into the marketplace widget and notifies the Lua handler. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00cf0700` | `CoinStoreDataSource::CoinStoreItems::Ctor` | CoinStoreDataSource::CoinStoreItems | Constructs the coin-store item-list data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0790` | `CoinStoreDataSource::CoinStoreItems::Dtor` | CoinStoreDataSource::CoinStoreItems | Destroys the coin-store item-list data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf07f0` | `CoinStoreDataSource::CoinStoreItems::FieldName` | CoinStoreDataSource::CoinStoreItems | Shared field-name mapping also referenced by the SearchResults vtable. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0860` | `CoinStoreDataSource::PlayerSell::Ctor` | CoinStoreDataSource::PlayerSell | Constructs the player-sell data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0930` | `CoinStoreDataSource::PlayerSell::FieldName` | CoinStoreDataSource::PlayerSell | Exposes the player-sell preview field label. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0950` | `CoinStoreDataSource::SearchResults::Ctor` | CoinStoreDataSource::SearchResults | Constructs the coin-store search-results data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf09d0` | `CoinStoreDataSource::SearchResults::Dtor` | CoinStoreDataSource::SearchResults | Destroys the search-results data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0a30` | `CoinStoreDataSource::ActiveFilters::FieldName` | CoinStoreDataSource::ActiveFilters | Names the active filter data-source columns. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0a80` | `CoinStoreDataSource::TransactionHistory::FieldName` | CoinStoreDataSource::TransactionHistory | Names the transaction-history data-source columns. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0fd0` | `ThisAdjustToPlayerSellDtor` | CoinStoreDataSource::PlayerSell | Secondary-vtable deleting-destructor adjustor for PlayerSell. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf1130` | `CoinStoreDataSource::CoinStoreItems::ScalarDeletingDtor` | CoinStoreDataSource::CoinStoreItems | Scalar deleting destructor for CoinStoreItems. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf11f0` | `CoinStoreDataSource::PlayerSell::Ctor` | CoinStoreDataSource::PlayerSell | Constructs the player-sell data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf1280` | `CoinStoreDataSource::PlayerSell::ScalarDeletingDtor` | CoinStoreDataSource::PlayerSell | Scalar deleting destructor for PlayerSell. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf12a0` | `CoinStoreDataSource::SearchResults::ScalarDeletingDtor` | CoinStoreDataSource::SearchResults | Scalar deleting destructor for SearchResults. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf1b60` | `CoinStoreDataSource::CoinStoreItems::GetFieldValue` | CoinStoreDataSource::CoinStoreItems | Exposes selected coin-store item properties to the GUI datasource layer. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf1d90` | `CoinStoreDataSource::PlayerSell::GetFieldValue` | CoinStoreDataSource::PlayerSell | Supplies fields for player items offered in the coin-store selling view. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf21a0` | `CoinStoreDataSource::ActiveFilters::Dtor` | CoinStoreDataSource::ActiveFilters | Destroys the active-filter datasource and its owned string/base members. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2230` | `CoinStoreDataSource::ActiveFilters::GetCount` | CoinStoreDataSource::ActiveFilters | Exposes the active-filter row count through the datasource virtual interface. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2250` | `CoinStoreDataSource::TransactionHistory::Dtor` | CoinStoreDataSource::TransactionHistory | Destroys a transaction-history datasource instance and its base member. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf22c0` | `CoinStoreDataSource::TransactionHistory::GetRowCount` | CoinStoreDataSource::TransactionHistory | Exposes transaction-history row count through its datasource virtual interface. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf22e0` | `CoinStoreDataSource::ActiveFilters::Ctor` | CoinStoreDataSource::ActiveFilters | Initializes the active-filter datasource and its GUI-visible name/count fields. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2370` | `CoinStoreDataSource::ActiveFilters::ScalarDeletingDtor` | CoinStoreDataSource::ActiveFilters | Implements the MSVC scalar deleting destructor for ActiveFilters. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2420` | `CoinStoreDataSource::TransactionHistory::ScalarDeletingDtor` | CoinStoreDataSource::TransactionHistory | Implements the MSVC scalar deleting destructor for TransactionHistory. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟢 likely (49)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a2f20` | `LookupMerchantSellBundlesDataSource` |  | Typed lookup for the MerchantSellBundles data source. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a31d0` | `CreateCategoryTreeNode` |  | Creates nodes used by the reconstructed store-category tree. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a36f0` | `RemoveTreeEntriesByMode` |  | Selects which store entries to remove during the caller's refresh pass. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3bf0` | `CopyRecordAndRebuildOneRow` |  | Called by the Item of the Day update wrapper at 009a51f0; this function performs the record copy and row refresh but does not parse a packet. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a3c60` | `CopyDefinitionAndRebuildOneRow` |  | Called by the Bundle of the Day update wrapper at 009a5210; this function performs the definition copy and row refresh but does not parse a packet. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a51f0` | `ApplyRecordUpdateAndNotify` |  | The native Item of the Day handler calls this wrapper; packet parsing is performed by the caller. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a5210` | `ApplyDefinitionUpdateAndNotify` |  | The native Bundle of the Day handler calls this wrapper; packet parsing is performed by the caller. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a52b0` | `SetKeyedValueAndNotify` |  | Used from purchase handlers to update and notify a keyed purchase data source; this function does not parse packets itself. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a5d30` | `RebuildFilteredStoreBundleRows` |  | Rebuilds the filtered StoreBundles row collection; the feature identity comes from its typed list callee and refresh callers. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a60d0` | `ClearCategoryTreeContents` |  | Cleans category tree resources for StoreBundleCategories. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a6280` | `RecycleCategoryTreeNode` |  | Returns one category-tree node to its owner pool. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `00b37470` | `NotifyPaymentPageResult3` |  | Reports matching payment-page result to script. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00bffe30` | `CoinStoreScreen::BuyItem` | CoinStoreScreen | Requests a coin-store purchase using three callback values. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c002d0` | `CoinStoreScreen::RequestStoreSort` | CoinStoreScreen | Requests a store sort using a value and sort criteria string. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00730` | `CoinStoreScreen::RequestBuyBackItem` | CoinStoreScreen | Requests a buy-back operation for the selected coin-store item. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00b50` | `CoinStoreScreen::RebuildFilteredList` | CoinStoreScreen | Updates the visible coin-store list using a filter string. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c39b90` | `ImageDataDefinition::ReadImageFields` | GameCommerce::ImageDataDefinition | Field loader for the image half of a commerce definition (member layout matches GameCommerce::ImageDataDefinition, whose vtable 018417bc sits at offset 0 and whose ctor 00d672e0 is called for this+0x10 of StoreDefinitio… | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c39e20` | `RemoveDefinitionFromKeyedTree` |  | Removal path for a store definition from the manager that owns the tree at +0x2bc; called from the store-bundle removal routine 00c3b790 and from 00c40360. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3ac60` | `StoreDefinition::AreBundleContentsAvailable ⚠downgrade` | GameCommerce::StoreDefinition<AppStoreB… | Availability gate used by the store UI/manager before offering a store's bundles for purchase; the per-bundle slot-0 predicate 0110acf0 checks that the bundle's availability fields at +0xd0..+0xe0 are all >= 0. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3acb0` | `StoreDefinition::HasBundles ⚠downgrade` | GameCommerce::StoreDefinition<AppStoreB… | Cheap 'this store actually has sellable bundles' test exposed as vtable slot 9 of the store definition. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3acd0` | `StoreDefinition::InitBundleEntry` | GameCommerce::StoreDefinition<AppStoreB… | Fill step of the map insertion path 00c3b370/00c3dc40 that adds a bundle definition to a store's bundle map. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3af60` | `PruneManagerEntriesWithZeroCount` |  | Drops expired definition entries of the purchase store manager; called from the store maintenance routine 00c3b6c0. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b020` | `InsertIndexedNode190` |  | Row-insert helper used by the marketing-table loader 00c40410 (fails with 'FAILED to load store billboard panels.' elsewhere in that unit) when filling the container of loaded definition nodes. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b0d0` | `ClearListAt8` |  | Teardown helper for a definition list owned by the store manager object; no separate class anchor exists in this function. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b240` | `DestroySetMemberAt4` |  | Compiler-generated member-cleanup scope for the small entry type whose Set member at +4 holds store bundle ids (the vtables of 009b40e0 name Set<StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,...>). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b290` | `EntryCtor` |  | Builds the (bundle-id -> set member) entry allocated by the container insert routine 00c3dbb0, which returns the pointer to this Set member. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b370` | `StoreDefinition::InsertBundleFromDefinition` | GameCommerce::StoreDefinition<AppStoreB… | Adds a bundle definition to a store; called by the re-index path 00c3e890/00c3fa40 and by the insert-or-update slot 00c3dc40. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b3f0` | `InsertIndexedNode4C` |  | Row-insert helper of the 00c3ff10 loader, which reports 'mrkt_store_portal_category_vw' / 'FAILED to find record, name=%s.' - i.e. the store portal-category definition table. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b490` | `InsertIndexedNode5C` |  | Row-insert helper of the 00c400c0 loader, which reports 'mrkt_store_billboard_panel_vw' / 'FAILED to find record, name=%s.' - i.e. the billboard-panel definition table. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b530` | `ReadShortcutRowFields ⚠downgrade` | GameCommerce::StoreShortcutDefinition | Field loader for a shortcut definition row of the marketing data (shortcut id/type/value plus the owning store id). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b650` | `DestroyStoreDefinitionAt0` |  | Compiler-generated member-cleanup scope that runs the store-definition destructor; the same dtor is also reached through the vtable (slot 0 scalar deleting dtor 00c3ac50 region and slot 2 00c3b120). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b6c0` | `RunMaintenanceSweeps` |  | Periodic cleanup of the purchase store manager's definition lists, called from the removal/unregister paths 00c3b790, 00c402b0 and 00c40360. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b790` | `RemoveContainerEntryById` |  | Body of the manager's 'remove store bundle' operation - its only caller 00b28300 logs "ProductUnavaliable Error trying to remove item as requested, didn't find it store bundle id=%d". | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3dbb0` | `InsertBucketedEntry1024` |  | Creates one keyed (id -> bundle-id set) entry; its caller 00c3eb90 is the manager-side loader that fills the store-bundle-of-the-day style collection built alongside ClientInGamePurchaseStoreManager (vtable 0180d864 Sto… | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3dde0` | `EntryCtorWithKeyAt2806C` |  | Creates one store entry of the manager list built by 00c3f930 (which links these objects through node+0x28064 and node+0x28070). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c77060` | `SellItemForMerchant` |  | Sell entry that takes the merchant from a context object instead of the session globals. No caller in this map. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77a30` | `BuyItemByItemId` |  | Lua-facing purchase entry keyed by item id; the -1 placeholder stands in for the record argument. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77b80` | `RequestStoreSearchInit` |  | Initialises the store search engine: completes at once when the engine is ready, otherwise requests the item definitions first. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77db0` | `BuyItemWithRecord` |  | Lua-facing purchase entry that resolves a record first and then runs the BuyItem path. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c78c10` | `SetPlayerCoin ⚠downgrade` |  | Sends the player coin total to the merchant UI through the HandlerMerchant SetPlayerCoin script event. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00cf08d0` | `CoinStoreDataSource::PlayerSell::CanSellItem ⚠downgrade` | CoinStoreDataSource::PlayerSell | Checks whether a player's item can be offered for sale. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf10c0` | `GetStoreFilterState` |  | The store-filter request path obtains this member through the getter. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf10d0` | `GetCoinStoreRecordMap` |  | The buy-item path reads a record through this accessor. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf1150` | `ResolveCoinStoreRecordForItem` |  | Resolves a coin-store item record with a fallback candidate. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf1e70` | `RefreshCoinStoreDatasourceRows` |  | Repopulates a datasource's visible rows after a selection or data refresh. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2120` | `RefreshCoinStoreFilteredRows` |  | Notifies the enclosing datasource after rebuilding its filtered rows. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2140` | `RefreshCoinStoreDatasource` |  | Refreshes a selected coin-store datasource and its visible rows. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2170` | `RefreshCoinStoreDatasourceAndNotify` |  | Refreshes a selected coin-store datasource and notifies its row provider. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2440` | `CoinStoreDataSource::Ctor ⚠downgrade` | CoinStoreDataSource | Builds the composite coin-store datasource and its named child data sources. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b394e0` | `NotifyAcceptedCreditCardsUpdated ?` |  | Publishes accepted-card changes to scripts. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00c00680` | `CoinStoreItemRecord::Ctor ? ⚠rename` | ItemRecord | Initializes a list item record through ItemRecord's constructor. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c00a50` | `CoinStoreScreen::RequestGiftPurchase ? ⚠rename` | CoinStoreScreen | Requests a gift purchase using a recipient id and item fields. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00cf1050` | `FindBrokerRecordByKey ? ⚠rename` |  | Lookup helper used while resolving coin-store records. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf1100` | `GetCoinStoreHistoryList ? ⚠rename` |  | The transaction-history callback accesses this embedded member through the getter. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## ⚪ unknown (27)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a3520` | `— ⚠reject` |  | Updates a selected store data-source object; the store association follows the typed lookup helper and neighboring caller context. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a40a0` | `— ⚠reject` | InGamePurchaseDataSource::StoreBundleAc… | Refreshes the active-filter count from the selected store bundle. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4540` | `— ⚠reject` | InGamePurchaseDataSource::StationCashPr… | Returns Station Cash price-list row values for the purchase UI. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4680` | `— ⚠reject` | InGamePurchaseDataSource::SubscriptionP… | Returns subscription-price row values for the purchase UI. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4850` | `— ⚠reject` | InGamePurchaseDataSource::AcceptedCredi… | Returns accepted credit-card type fields for the purchase UI. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4990` | `— ⚠reject` | InGamePurchaseDataSource::CountryCodes | Returns country-code row values for the purchase UI. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4a80` | `— ⚠reject` | InGamePurchaseDataSource::StateCodes | Returns state-code row values for the purchase UI. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a4dc0` | `— ⚠reject` |  | Teardown helper used by the ClientInGamePurchaseManager destructor; its embedded reference-list type is visible in the vtable data. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `009a5eb0` | `— ⚠reject` |  | Builds the store category tree from the backing category records. | [[Native Map/Shards/S0268 009a1ca0-009a6ce0|S0268]] |
| `00b37570` | `— ⚠reject` |  | Reports matching alternate payment-page result to script. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b37770` | `— ⚠reject` |  | Refreshes tooltip for selected store bundle. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b37ce0` | `— ⚠reject` | ClientInGamePurchaseManager | Publishes marketplace readiness to scripts. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b39660` | `— ⚠reject` |  | Selects and displays active sale. | [[Native Map/Shards/S0341 00b33bc0-00b39af0|S0341]] |
| `00b47320` | `— ⚠reject` | ClientInGamePurchaseManager | Entry point behind the store/merchant screen opening: it makes sure the billing data and start-up data are present before the UI is told to open. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b47ca0` | `— ⚠reject` | ClientInGamePurchaseManager | Completion handler for a store order or preview request: it converts the server's order result into the client's order dialog outcome, fires the UI event and, on success, sends the order item list back through 00b46f10. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49090` | `— ⚠reject` | ClientInGamePurchaseManager | Store/merchant session teardown: it closes the session object and pushes the serialised store data to the server, and the log line documents the known double-fire of the UI event. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00c39c80` | `— ⚠reject` | ClientInGamePurchaseStoreManager | Startup path of the marketing-data-backed store (coin store) client manager: point the data updater at the marketing definitions schema and enable automatic reload. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c39e70` | `— ⚠reject` |  | Expiry/cleanup pass over one manager definition list (entries whose predicate fails are dropped); invoked from the store maintenance routine 00c3b6c0. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c39ed0` | `— ⚠reject` |  | Second expiry pass of the store maintenance routine 00c3b6c0: drops entries whose nested rows (stride 0x5c) all fail validation. | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3adb0` | `— ⚠reject` | GameCommerce::StoreDefinition<AppStoreB… | Loggy/verbose rendering of one store of the purchase store manager (slot 1 of the store definition vtable). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3b720` | `— ⚠reject` |  | Completion notification of the store's bundle content download: publish the assembled content or log the miss, as reported by its caller 00b39340 ('OnUpdatedStoreItem'). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c3dc40` | `— ⚠reject` | GameCommerce::StoreDefinition<AppStoreB… | Slot 5 of the store definition: adds or refreshes one bundle definition of a store (called from the bundle-table loaders 00c3e890/00c3fa40 and the manager insert 00c3ac50 region). | [[Native Map/Shards/S0390 00c39b90-00c3de40|S0390]] |
| `00c77a60` | `— ⚠reject` |  | Starts a store filter request: builds the filter result list and completes it either at once or after the item definitions arrive. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c77ca0` | `— ⚠reject` |  | Starts a store sort request (pending state 3) with the sort key and criteria; completion follows the same path as the filter request. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00cf1ee0` | `— ⚠reject` | CoinStoreDataSource::ActiveFilters | Provides the active-filter row data exposed to the coin-store GUI. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf1f70` | `— ⚠reject` | CoinStoreDataSource::TransactionHistory | Feeds transaction-history rows and fields to the coin-store GUI. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf2390` | `— ⚠reject` | CoinStoreDataSource::TransactionHistory | Initializes the transaction-history datasource instance and selection state. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b47ca0` | `— ⚠reject` | send | PacketTunneledClientWorldPacket |  |  |
