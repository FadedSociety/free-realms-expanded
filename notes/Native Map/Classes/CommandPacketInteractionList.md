# Class: CommandPacketInteractionList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9b7e0` | `CommandPacketInteractionList::~CommandPacketInteractionList` | 🟢 | dtor | Calls 00a9b760 on this (that helper destroys the embedded InteractionData array through 00a9af50 and releases the IString<char> member at +0xc), then stores BaseCommandPacket::vftable and BasePacket::vftable into the ob… |
| `00a9c1c0` | `CommandPacketInteractionList::CommandPacketInteractionList` | ✅ | ctor | Calls 008a2820 with sub-opcode 9 and zero extra word, calls 00a9c090 on a member, stores CommandPacketInteractionList::vftable and zeroes the byte at +0x248. |
| `00a9c230` | `CommandPacketInteractionList::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9b7e0 (the member-cleanup destructor chain) on this, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of CommandPacketInteractionList:… |
