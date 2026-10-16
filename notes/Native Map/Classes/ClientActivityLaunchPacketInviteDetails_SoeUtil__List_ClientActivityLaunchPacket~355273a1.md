# Class: ClientActivityLaunchPacketInviteDetails<SoeUtil::List<ClientActivityLaunchPacketMember,-1>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c30d0` | `ClientActivityLaunchPacketInviteDetailsDtor` | 🟢 | dtor | Calls the IStringFixed cleanup on the subobject at +0x38, restores the member-list vtable at +0x14 and removes its nodes, then restores BasePacket::vftable at offset zero. |
| `009c3160` | `ActivityLaunchInviteDetailsCtor` | ✅ | ctor | Initializes the invite-details packet and its member list, sets default fields and two shared constants, then clears a later field. |
| `009c3200` | `ActivityLaunchInviteDetailsScalarDeletingDtor` | ✅ | dtor | Calls packet teardown, conditionally frees the allocation from the deleting flag, and otherwise returns the original pointer. |
