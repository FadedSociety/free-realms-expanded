# Class: PacketInGamePurchaseAccountInfoResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2fed0` | `PacketInGamePurchaseAccountInfoResponse::Ctor` | ✅ | ctor | Initializes packet base with opcode 0x1a, writes the response vtable, stores an argument in field 6, initializes two members, and writes one byte at field 0x54. |
| `00b2ff60` | `PacketInGamePurchaseAccountInfoResponse::ScalarDeletingDtor` | ✅ | dtor | Calls response cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns its pointer. |
