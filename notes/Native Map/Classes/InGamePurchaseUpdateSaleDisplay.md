# Class: InGamePurchaseUpdateSaleDisplay

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2c1a0` | `UpdateSaleDisplay::Constructor` | ✅ | ctor | Initializes the purchase packet base with subtype 0x28, writes the update vtable, and initializes an empty SaleDisplayInfo list. |
| `00b2c210` | `UpdateSaleDisplay::Dtor` | ✅ | dtor | Restores the update and list vtables, drains the sale-display nodes, then restores BasePacket's vtable. |
| `00b2c280` | `UpdateSaleDisplay::ScalarDeletingDtor` | ✅ | virtual | Calls the update packet destructor and conditionally frees the object according to the deleting flag. |
