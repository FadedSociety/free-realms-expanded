# Class: ClientActivityLaunchPacketKickMemberEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4aba0` | `ClientActivityLaunchPacketKickMemberEvent::~ClientActivityLaunchPacketKickMembe…` | 🟢 | dtor | Destructor of the two-member kick-event packet: destroys both embedded member objects through 00703350 (the second inside an EH scope) and rewrites *this to BasePacket::vftable. |
| `00c4af20` | `KickMemberEvent_ReadFields` | 🟢 | method | Field reader for a kick-event packet: shared header read 009c1e00, then a 64-bit guid at +0x18/+0x1c, a length-prefixed string via 00894b10, a second 64-bit guid and a second length-prefixed string, each with stream bou… |
| `00c4b510` | `ClientActivityLaunchPacketKickMemberEvent::ClientActivityLaunchPacketKickMember…` | ✅ | ctor | Constructor: family base 009c0c30 with sub-opcode 0xd (13), installs its vtable, initialises the first member (default guid pair plus initialiser 00703540) and then a second member at +0x58 with its own default guid pai… |
| `00c4b5b0` | `ClientActivityLaunchPacketKickMemberEvent::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4aba0 and frees the object with CRT _free when bit 0 of the flag is set. |
