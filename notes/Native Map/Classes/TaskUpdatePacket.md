# Class: TaskUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86900` | `TaskUpdatePacketCtor` | ✅ | ctor | Constructor of TaskUpdatePacket: runs the task base constructor with sub-opcode 2, argument 0xffffffff, stores the TaskUpdatePacket vtable and initialises the payload dword at +0x10 to 0xffffffff. |
| `00a87e40` | `TaskUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of TaskUpdatePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
