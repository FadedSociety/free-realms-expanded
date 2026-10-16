# Class: QuestEndReplyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a864c0` | `QuestEndReplyPacketCtor` | ✅ | ctor | Constructor of QuestEndReplyPacket: runs the BaseQuestPacket constructor with sub-opcode 0xe (family class id 49), stores the QuestEndReplyPacket vtable and saves param_2 into +0xc. |
| `00a87c60` | `QuestEndReplyPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of QuestEndReplyPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
