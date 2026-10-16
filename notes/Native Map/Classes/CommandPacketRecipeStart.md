# Class: CommandPacketRecipeStart

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14b30` | `CommandPacketRecipeStart::CommandPacketRecipeStart` | ✅ | ctor | Constructor taking one word: calls BaseCommandPacket::BaseCommandPacket (FUN_008a2820) with sub-opcode 0x18 (24) and extra word 0, stores CommandPacketRecipeStart::vftable at *this and writes param_2 at +0x10. |
| `00c1aa90` | `CommandPacketRecipeStart::ScalarDeletingDtor` | ✅ | dtor | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |
