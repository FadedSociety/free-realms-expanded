# Class: CommandPacketMoveAndInteract

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98750` | `CommandPacketMoveAndInteract::~CommandPacketMoveAndInteract` | 🟢 | dtor | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the family-26 inbound dispatcher 00aa2560 at the end of case 0x16 and from EH unwind funclet 015… |
| `00a99cd0` | `CommandPacketMoveAndInteract::CommandPacketMoveAndInteract` | ✅ | ctor | Calls 008a2820 with sub-opcode 0x16 (22) and zero extra word, then stores CommandPacketMoveAndInteract::vftable and returns this. |
| `00a99d30` | `CommandPacketMoveAndInteract::ScalarDeletingDtor` | ✅ | dtor | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of CommandPack… |
