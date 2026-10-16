# Class: ClientActivityLaunchPacketInviteDetails<SoeUtil::List<ClientActivityLaunchMember,-1>>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4d410` | `ClientActivityLaunchPacketInviteDetails::~ClientActivityLaunchPacketInviteDetai…` | 🟢 | dtor | Destructor of the invite-details packet: destroys the member subobject through 00703350, installs the member-list vtable at +0x14 and drains that list with 00c4bb70, then rewrites *this to BasePacket::vftable. |
| `00c4d590` | `ClientActivityLaunchPacketInviteDetails::ClientActivityLaunchPacketInviteDetails` | ✅ | ctor | Constructor of the invite-details packet (template instance with an external member list): family base 009c0c30 with sub-opcode 1, installs its vtable, installs a SoeUtil member-list vtable at +0x14 with zeroed head/tai… |
| `00c4d650` | `ClientActivityLaunchPacketInviteDetails::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the class destructor 00c4d410 and frees the object with CRT _free when bit 0 of the flag is set. |
