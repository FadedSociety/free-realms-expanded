# Class: BaseUiPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86640` | `BaseUiPacketCtor` | ✅ | ctor | Base constructor of the BaseUiPacket (class 47) family: stores BasePacket's vtable, writes the family class id 0x2f (47) into +4, stores the BaseUiPacket vtable and writes the per-subclass sub-opcode param_2 into +8. |
| `00a87d20` | `BaseUiPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of BaseUiPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
