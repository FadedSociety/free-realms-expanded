# Class: TaskFailPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86a80` | `TaskFailPacketCtor` | ✅ | ctor | Constructor of TaskFailPacket: runs the task base constructor with sub-opcode 4 and the caller's id argument, then stores the TaskFailPacket vtable. |
| `00a87f00` | `TaskFailPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of TaskFailPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
