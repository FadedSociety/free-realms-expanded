# Class: ClientActivityLaunchPacketInviteMemberReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4a940` | `ClientActivityLaunchPacketInviteMemberReply::~ClientActivityLaunchPacketInviteM…` | 🟢 | dtor | Destructor: destroys the embedded member object through 00703350 and rewrites *this to BasePacket::vftable. |
| `00c4acd0` | `InviteMemberReply_ReadFields` | 🟢 | method | Field reader for an invite-member-reply packet: calls the shared header reader 009c1e00, then reads a 64-bit guid into +0x18/+0x1c, a length-prefixed string through 00894b10, a boolean into +0x54 and a 32-bit code into … |
| `00c4b1a0` | `ClientActivityLaunchPacketInviteMemberReply::ClientActivityLaunchPacketInviteMe…` | ✅ | ctor | Constructor: family base 009c0c30 with sub-opcode 2, installs its vtable, initialises the member guid to the default pair, runs the member initialiser, and clears the byte at +0x54 and the dword at +0x58 used by the rep… |
| `00c4b230` | `ClientActivityLaunchPacketInviteMemberReply::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4a940 and frees the object with CRT _free when bit 0 of the flag is set. |
