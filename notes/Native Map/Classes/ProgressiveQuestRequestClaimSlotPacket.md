# Class: ProgressiveQuestRequestClaimSlotPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14560` | `ProgressiveQuestRequestClaimSlotPacket::~ProgressiveQuestRequestClaimSlotPacket` | 🟢 | dtor | Destructor body that stores only BasePacket::vftable at *this and returns. |
| `00c145b0` | `— ⚠reject` | ⚪ | ctor | Constructor taking two payload words: calls the progressive-quest base constructor FUN_00bdc8a0 with sub-opcode 3, stores ProgressiveQuestRequestClaimSlotPacket::vftable at *this and writes param_2 at +0xc and param_3 a… |
| `00c1a790` | `ProgressiveQuestRequestClaimSlotPacket::ScalarDeletingDtor` | ✅ | dtor | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |
