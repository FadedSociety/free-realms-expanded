# Class: PacketInGamePurchaseAccountInfoRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2fe40` | `PacketInGamePurchaseAccountInfoRequest::Ctor` | ✅ | ctor | Initializes packet base with opcode 0x19, writes its request vtable, initializes a payload member from the argument, and returns the packet. |
| `00b2feb0` | `PacketInGamePurchaseAccountInfoRequest::ScalarDeletingDtor` | ✅ | dtor | Calls packet cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the pointer. |
