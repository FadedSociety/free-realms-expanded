# Class: IgnoreRemovePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7e120` | `IgnoreRemovePacket::Construct` | ✅ | ctor | Calls the common ignore packet constructor with sub-opcode 3 and installs IgnoreRemovePacket::vftable. |
| `00c7e180` | `IgnoreRemovePacket::DeletingDtor` | ✅ | dtor | Restores BasePacket::vftable, frees the object if flag bit 0 is set, and otherwise returns its pointer. |
