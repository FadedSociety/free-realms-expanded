# Class: ClientActivityLaunchPacketInviteMemberResult

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4a9a0` | `ClientActivityLaunchPacketInviteMemberResult::~ClientActivityLaunchPacketInvite…` | 🟢 | dtor | Destructor: destroys the embedded member object through 00703350 and rewrites *this to BasePacket::vftable. |
| `00c4adb0` | `InviteMemberResult_ReadFields` | 🟢 | method | Field reader with the same layout as 00c4acd0 (guid at +0x18/+0x1c, string via 00894b10, byte at +0x54, dword at +0x58) but called from the invite-member-result paths. |
| `00c4b250` | `ClientActivityLaunchPacketInviteMemberResult::ClientActivityLaunchPacketInviteM…` | ✅ | ctor | Constructor: family base 009c0c30 with sub-opcode 3, installs its vtable, initialises the member guid to the default pair, runs the member initialiser, and clears the byte at +0x54 and the dword at +0x58. |
| `00c4b2e0` | `ClientActivityLaunchPacketInviteMemberResult::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4a9a0 and frees the object with CRT _free when bit 0 of the flag is set. |
