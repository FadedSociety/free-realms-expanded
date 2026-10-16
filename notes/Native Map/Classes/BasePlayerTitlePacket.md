# Class: BasePlayerTitlePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b49140` | `BasePlayerTitlePacket::CtorDefault` | ✅ | ctor | Default constructor: stores BasePacket::vftable (018011ec), writes the 32-bit opcode 0x98 (152) at +4, stores BasePlayerTitlePacket::vftable (01821254), and zeroes the sub-opcode field at +8. |
| `00b491a0` | `— ⚠reject` | ⚪ | ctor | Constructor taking the sub-opcode: stores BasePacket::vftable, writes opcode 0x98 (152) at +4, stores BasePlayerTitlePacket::vftable, and writes the caller's value into the sub-opcode field at +8. |
| `00b49210` | `BasePlayerTitlePacket::Dtor` | ✅ | dtor | Destructor body: rewrites the object's vtable from BasePlayerTitlePacket::vftable (01821254) to BasePacket::vftable (018011ec); no members are released and no memory is freed. |
| `00b49260` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: restores the base vtable chain (BasePlayerTitlePacket::vftable then BasePacket::vftable) and, when the low bit of the flag argument is set, calls _free(this) (which does not return); returns … |
