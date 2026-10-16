# Class: CommandPacketRequestStorageCheck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14a80` | `— ⚠reject` | ⚪ | ctor | Constructor: calls BaseCommandPacket::BaseCommandPacket (FUN_008a2820) with sub-opcode 0x2c (44) and extra word 0, then stores CommandPacketRequestStorageCheck::vftable at *this. |
| `00c1aa20` | `CommandPacketRequestStorageCheck::ScalarDeletingDtor` | ✅ | dtor | Restores BaseCommandPacket and BasePacket vptrs, then frees the allocation when the deleting flag is set. |
