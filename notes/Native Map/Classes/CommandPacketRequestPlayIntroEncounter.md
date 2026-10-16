# Class: CommandPacketRequestPlayIntroEncounter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14920` | `— ⚠reject` | ⚪ | ctor | Constructor: calls BaseCommandPacket::BaseCommandPacket (FUN_008a2820) with sub-opcode 0x28 (40) and extra word 0, then stores CommandPacketRequestPlayIntroEncounter::vftable at *this. |
| `00c1a940` | `CommandPacketRequestPlayIntroEncounter::ScalarDeletingDtor` | ✅ | dtor | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |
