# Class: AcquaintanceOnlinePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a12160` | `AcquaintanceOnlinePacket_Ctor` | 🟢 | ctor | Constructs the packet by calling FUN_00a10210(3) (acquaintance base with sub-opcode 3), installs AcquaintanceOnlinePacket::vftable, stores param_2 and param_3 into +0x10 and +0x14 and the byte param_4 at +0x18. |
| `00a121d0` | `AcquaintanceOnlinePacket_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: stores BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |
