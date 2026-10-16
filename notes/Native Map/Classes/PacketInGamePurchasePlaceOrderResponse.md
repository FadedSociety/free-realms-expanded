# Class: PacketInGamePurchasePlaceOrderResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2f8d0` | `PacketInGamePurchasePlaceOrderResponse::Ctor` | ✅ | ctor | Initializes packet base with opcode 4, writes the response vtable, stores two arguments in fields 6 and 7, initializes a subobject, and stores two more arguments. |
| `00b2f960` | `PacketInGamePurchasePlaceOrderResponse::ScalarDeletingDtor` | ✅ | dtor | Calls the packet cleanup routine, frees the object when flag bit 0 is set, and otherwise returns the original pointer. |
