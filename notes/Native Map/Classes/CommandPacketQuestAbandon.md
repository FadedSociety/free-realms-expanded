# Class: CommandPacketQuestAbandon

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c09f60` | `— ⚠reject` | ⚪ | ctor | Command packet constructor: calls the command base constructor FUN_008a2820(0x17,0) (command opcode 0x17) and installs CommandPacketQuestAbandon::vftable; the quest id is stored in the word at +0x10 by the caller 00c0a5… |
| `00c0a2e0` | `CommandPacketQuestAbandon_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of CommandPacketQuestAbandon::vftable: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
