# Class: BaseQuestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86340` | `BaseQuestPacketCtor` | ✅ | ctor | Base constructor of the quest packet family: stores BasePacket's vtable, writes the family class id 0x31 (49) into +4, stores the BaseQuestPacket vtable and writes the per-subclass sub-opcode param_2 into +8. |
| `00a863b0` | `BaseQuestPacketDtorBody` | 🟢 | dtor | Minimal destructor body used by the quest packet family: rewrites the object's vtable to ClientGateway::BasePacket and returns; 49 EH unwind funclets and one quest handler call it, marking it as the shared cleanup for t… |
| `00a87ba0` | `BaseQuestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of BaseQuestPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
