# Class: BrokerPlayerInventoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cee480` | `BrokerPlayerInventoryDataSource::Ctor` | ✅ | ctor | Initializes the named BaseClient.Broker.PlayerInventoryItems data source, installs its vtables at four object offsets, and stores the broker argument. |
| `00cee560` | `BrokerPlayerInventoryDataSource::CanOfferItem` | ✅ | virtual | Rejects invalid or unavailable item records and two state flags; when a limit is enabled, also requires the computed available count to be positive. |
| `00cee950` | `ThisAdjustToBrokerPlayerInventoryDtor` | ✅ | dtor | Subtracts 0x254 from ECX and jumps to the adjusted BrokerPlayerInventoryDataSource destructor entry; this is an adjustor thunk, not a separate destructor body. |
| `00ceece0` | `BrokerPlayerInventoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the member cleanup routine, then frees the object if the deleting flag bit is set; otherwise returns the object pointer. |
