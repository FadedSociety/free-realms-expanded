# Class: PacketInGamePurchasePlaceOrderClientTicket<GameCommerce::ClientInGamePurchaseOrder>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2a7f0` | `PlaceOrderClientTicket::Constructor ⚠downgrade` | 🟢 | ctor | Initializes the purchase base with subtype 0x24, stores the supplied field at offset 0x18, and initializes a byte-array member. |
| `00b2a860` | `PlaceOrderClientTicket::Dtor` | ✅ | dtor | Releases the owned byte-array allocation and restores the BasePacket vtable. |
| `00b2a920` | `PlaceOrderClientTicket::ScalarDeletingDtor` | ✅ | virtual | Calls the packet destructor, conditionally frees the object, and returns its pointer when not deleting. |
