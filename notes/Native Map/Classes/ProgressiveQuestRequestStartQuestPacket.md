# Class: ProgressiveQuestRequestStartQuestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c144a0` | `ProgressiveQuestRequestStartQuestPacket::~ProgressiveQuestRequestStartQuestPack…` | 🟢 | dtor | Destructor body that stores only BasePacket::vftable at *this and returns (the packet's two payload words at +0xc/+0x10 are scalars). |
| `00c144f0` | `ProgressiveQuestRequestStartQuestPacket::ProgressiveQuestRequestStartQuestPacket` | ✅ | ctor | Constructor taking two payload words: calls the progressive-quest base constructor FUN_00bdc8a0 with sub-opcode 2, stores ProgressiveQuestRequestStartQuestPacket::vftable at *this, writes param_2 at +0xc and param_3 at … |
| `00c1a730` | `ProgressiveQuestRequestStartQuestPacket::ScalarDeletingDtor` | ✅ | dtor | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |
