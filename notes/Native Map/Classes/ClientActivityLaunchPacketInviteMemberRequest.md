# Class: ClientActivityLaunchPacketInviteMemberRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4a8e0` | `ClientActivityLaunchPacketInviteMemberRequest::~ClientActivityLaunchPacketInvit…` | 🟢 | dtor | Destructor: destroys the embedded member object through 00703350 and then rewrites *this to BasePacket::vftable; it does not free the object. |
| `00c4b100` | `ClientActivityLaunchPacketInviteMemberRequest::ClientActivityLaunchPacketInvite…` | ✅ | ctor | Constructor: family base 009c0c30 with sub-opcode 0x14 (20), installs its vtable, initialises the member guid to the default pair DAT_01bee130/DAT_01bee134 and runs the member initialiser 00703540. |
| `00c4b180` | `ClientActivityLaunchPacketInviteMemberRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4a8e0 and frees the object with CRT _free when bit 0 of the flag is set. |
