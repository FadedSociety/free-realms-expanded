# Class: BaseTaskPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86710` | `BaseTaskPacketCtor` | ✅ | ctor | Base constructor of the task packet family (a BaseUiPacket subclass): runs the BaseUiPacket constructor 00a86640(param_2) with the caller's sub-opcode, stores the BaseTaskPacket vtable and saves param_3 into +0xc. |
| `00a87d80` | `BaseTaskPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of BaseTaskPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
