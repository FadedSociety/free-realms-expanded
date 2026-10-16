# Class: PacketInGamePurchaseStoreBundleContentResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2c120` | `StoreBundleContentResponse::Constructor` | ✅ | ctor | Initializes the purchase base packet with subtype 0x1c, writes the response vtable, and initializes its inline data array. |
| `00b2c180` | `StoreBundleContentResponse::ScalarDeletingDtor` | ✅ | virtual | Calls the packet destructor and conditionally frees the object according to the deleting flag. |
