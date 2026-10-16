# Class: BrokerRemoveItemFromListsPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00beeba0` | `BrokerRemoveItemFromListsPacket::Ctor ⚠downgrade` | 🟢 | ctor | Constructs broker subtype 11, installs the concrete vtable, and copies two default payload values from globals. |
| `00beec10` | `BrokerRemoveItemFromListsPacket::ScalarDeletingDtor` | ✅ | dtor | Restores the broker base vtables, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
