# Class: ActivityManagerBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdeac0` | `ActivityManagerBasePacket_Ctor` | ✅ | ctor | Constructor of the opcode 147 family: stores BasePacket::vftable, writes the family opcode 0x93 (147) into the packet field at +4, installs ActivityManagerBasePacket::vftable and zeroes the sub-opcode field at +8. |
| `00bdeb20` | `ActivityManagerBasePacket_CtorWithSubOpcode` | ✅ | ctor | Constructor variant of the opcode 147 family that takes the sub-opcode as an argument: stores BasePacket::vftable, writes 0x93 (147) into +4, installs ActivityManagerBasePacket::vftable and stores the sub-opcode argumen… |
| `00bdeb90` | `ActivityManagerBasePacket::~ActivityManagerBasePacket` | 🟢 | dtor | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; called at the end of every branch of the opcode-147 handler 00be7360 and referenced by six EH unwind funclets. |
| `00be0030` | `ActivityManagerBasePacket::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of ActivityManagerBasePacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when the low bit of the flag argument is se… |
