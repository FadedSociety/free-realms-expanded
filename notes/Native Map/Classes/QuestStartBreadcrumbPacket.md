# Class: QuestStartBreadcrumbPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86580` | `— ⚠reject` | ⚪ | ctor | Constructor of QuestStartBreadcrumbPacket: runs the BaseQuestPacket constructor with sub-opcode 0xf (family class id 49), stores the QuestStartBreadcrumbPacket vtable and saves param_2 into +0xc. |
| `00a87cc0` | `QuestStartBreadcrumbPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of QuestStartBreadcrumbPacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
