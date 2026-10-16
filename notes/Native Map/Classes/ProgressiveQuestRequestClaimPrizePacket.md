# Class: ProgressiveQuestRequestClaimPrizePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14620` | `ProgressiveQuestRequestClaimPrizePacket::~ProgressiveQuestRequestClaimPrizePack…` | 🟢 | dtor | Destructor body that stores only BasePacket::vftable at *this and returns. |
| `00c14670` | `ProgressiveQuestRequestClaimPrizePacket::ProgressiveQuestRequestClaimPrizePacket` | ✅ | ctor | Constructor taking two payload words: calls the progressive-quest base constructor FUN_00bdc8a0 with sub-opcode 4, stores ProgressiveQuestRequestClaimPrizePacket::vftable at *this and writes param_2 at +0xc and param_3 … |
| `00c1a7f0` | `ProgressiveQuestRequestClaimPrizePacket::ScalarDeletingDtor` | ✅ | dtor | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |
