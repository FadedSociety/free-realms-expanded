# Class: QuestReplyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86400` | `QuestReplyPacketCtor ⚠downgrade` | 🟢 | ctor | Constructor of QuestReplyPacket: runs the BaseQuestPacket constructor with sub-opcode 2 (family class id 49), stores the QuestReplyPacket vtable, saves param_2 into +0xc and the byte param_3 into +0x10. |
| `00a87c00` | `QuestReplyPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of QuestReplyPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
