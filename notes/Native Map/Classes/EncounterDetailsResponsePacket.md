# Class: EncounterDetailsResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa2e90` | `EncounterDetailsResponsePacket::EncounterDetailsResponsePacket` | ✅ | ctor | Calls FUN_008a2fd0(0x72) (encounter base constructor with sub-opcode 114), stores EncounterDetailsResponsePacket::vftable, clears the byte at object+0x2e8 and the dword at +0x2ec, then calls FUN_009b4060. |
| `00aa3050` | `EncounterDetailsResponsePacket::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00aa2f20 on this, tests flag bit 0, frees this when the bit is set, and otherwise returns this. |
