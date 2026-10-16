# Class: BaseIgnorePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7d9e0` | `BaseIgnorePacket::Construct` | ✅ | ctor | Installs BasePacket then BaseIgnorePacket vtables, stores family id 0x70 and the supplied ignore sub-opcode at object+8. |
| `00c7e0c0` | `BaseIgnorePacket::DeletingDtor` | ✅ | dtor | Restores BasePacket::vftable, conditionally frees the object when flag bit 0 is set, and otherwise returns the pointer. |
