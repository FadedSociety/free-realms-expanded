# Class: CommandPacketPlayDialogEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9ad20` | `CommandPacketPlayDialogEffect::CommandPacketPlayDialogEffect` | ✅ | ctor | Calls 008a2820 with sub-opcode 0x1c (28) and zero extra word, stores CommandPacketPlayDialogEffect::vftable and zeroes the words at +0x10, +0x14 and +0x18. |
| `00a9ad90` | `CommandPacketPlayDialogEffect::ScalarDeletingDtor` | ✅ | dtor | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of CommandPack… |
