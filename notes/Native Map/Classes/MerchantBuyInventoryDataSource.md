# Class: MerchantBuyInventoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfd8c0` | `MerchantBuyInventoryDataSource::MerchantBuyInventoryDataSource` | ✅ | ctor | Initializes the shared inventory data-source base with BaseClient.MerchantBuyInventoryData, installs the derived vtables, and stores the supplied owner pointer. |
| `00cfd9a0` | `MerchantBuyInventoryDataSource::IsRowEligible ⚠downgrade` | 🟢 | virtual | Rejects rows failing the base predicate, rows without a resolved usable item record, and certain duplicate linked records; returns one only when the checks pass. |
| `00cfddc0` | `MerchantBuyInventoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Runs the shared BaseFilterItemDataSource destructor and frees this object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |
