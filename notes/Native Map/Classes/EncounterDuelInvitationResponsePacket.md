# Class: EncounterDuelInvitationResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a879f0` | `EncounterDuelInvitationResponsePacketCtor` | ✅ | ctor | Constructor of the duel-invitation response packet: runs the BaseEncounterPacket family constructor 008a2fd0(0x69) (class id 41 plus sub-opcode 0x69), stores the EncounterDuelInvitationResponsePacket vtable, copies two … |
| `00a87a60` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (vtable slot [0]) of EncounterDuelInvitationResponsePacket: chains the two vtable stores of destruction (own class then BasePacket) and frees the object with _free when the delete flag bit 0 i… |
| `00a97c10` | `EncounterDuelInvitationResponsePacket::BuildAndSend` | 🟢 | method | Thiscall with a byte and two dword arguments. Constructs the packet in the stack record at buffer+0 via callee 00a879f0 (which stores EncounterDuelInvitationResponsePacket::vftable and passes sub-opcode 0x69 to 008a2fd0… |
