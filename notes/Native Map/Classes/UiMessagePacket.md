# Class: UiMessagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86d70` | `UiMessagePacketCtor` | ✅ | ctor | Constructor of UiMessagePacket: runs the BaseUiPacket constructor with sub-opcode 0xf (class id 47), stores the UiMessagePacket vtable and saves param_2 into +0xc. |
| `00a881a0` | `UiMessagePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of UiMessagePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
