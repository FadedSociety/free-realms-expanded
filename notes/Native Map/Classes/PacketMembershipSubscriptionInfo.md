# Class: PacketMembershipSubscriptionInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089c200` | `PacketMembershipSubscriptionInfoCtor` | ✅ | ctor | Constructor of PacketMembershipSubscriptionInfo: installs BasePacket::vftable, stores opcode 0xbd (189) at +4, installs the derived vtable and clears the two flag bytes at +8 and +9. |
| `008b1a50` | `PacketMembershipSubscriptionInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketMembershipSubscriptionInfo::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
