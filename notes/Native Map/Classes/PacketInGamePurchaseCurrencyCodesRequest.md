# Class: PacketInGamePurchaseCurrencyCodesRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2fbe0` | `PacketInGamePurchaseCurrencyCodesRequest::Ctor` | ✅ | ctor | Initializes packet base with opcode 0x10, writes the request vtable, initializes its payload member from the argument, and returns the packet. |
| `00b2fc50` | `PacketInGamePurchaseCurrencyCodesRequest::ScalarDeletingDtor` | ✅ | dtor | Calls packet cleanup, frees the object when flag bit 0 is set, and otherwise returns the original pointer. |
