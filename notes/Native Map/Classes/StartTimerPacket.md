# Class: StartTimerPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86bf0` | `— ⚠reject` | ⚪ | ctor | Constructor of StartTimerPacket: runs the BaseUiPacket constructor with sub-opcode 10 (class id 47), stores the StartTimerPacket vtable and saves param_2 into +0xc. |
| `00a86c60` | `StartTimerPacketDtorBody` | 🟢 | dtor | Minimal destructor body that only rewrites the object's vtable to ClientGateway::BasePacket and returns; reached from the task/UI dispatcher 00a91bf0 and one MSVC unwind funclet. |
| `00a88020` | `StartTimerPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of StartTimerPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
