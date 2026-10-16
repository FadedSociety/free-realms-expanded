# Class: IgnoreAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7eec0` | `IgnoreAddPacket::Construct` | ✅ | ctor | Calls common ignore initializer with sub-opcode 2, installs IgnoreAddPacket vtable and initializes an embedded IString member. |
| `00c7ef20` | `IgnoreAddPacket::DeletingDtor` | ✅ | dtor | Calls cleanup body 00c7ec20 and frees on flag bit 0, otherwise returns the packet pointer. |
