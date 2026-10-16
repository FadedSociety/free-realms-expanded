# Class: MerchantSellInventoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfec80` | `MerchantSellInventoryDataSource::RebuildRowOrder` | ✅ | virtual | Builds a temporary list from the current row array, optionally reorders it with the configured callback, copies ordered row pointers back to the output array, clears a state byte, and destroys the temporary list. |
| `00cfeea0` | `MerchantSellInventoryDataSource::MerchantSellInventoryDataSource` | ✅ | ctor | Initializes the shared inventory data-source base with BaseClient.MerchantSellInventoryData, installs derived vtables, initializes row arrays, and stores the mode-dependent comparison callback. |
| `00cfefd0` | `MerchantSellInventoryDataSource::Cleanup` | 🟢 | dtor | Clears and frees the two embedded pointer arrays, then invokes the shared BaseFilterItemDataSource destructor body. |
| `00cff090` | `MerchantSellInventoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the merchant sell datasource cleanup body and frees the object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |
