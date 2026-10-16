# Class: TaskAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a867e0` | `TaskAddPacketCtor ⚠downgrade` | 🟢 | ctor | Constructor of TaskAddPacket: runs the task base constructor with sub-opcode 1 and second argument 0xffffffff, stores the TaskAddPacket vtable and zeroes the payload fields at +0x10..+0x20 (dwords [4],[5],[6],[8], byte … |
| `00a86850` | `TaskAddPacketDtorBody` | 🟢 | dtor | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and from one MSVC unwind funclet. |
| `00a87de0` | `TaskAddPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of TaskAddPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
