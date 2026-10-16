# Class: PacketClientMembershipVipInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b1970` | `PacketClientMembershipVipInfo::PacketClientMembershipVipInfo` | ✅ | ctor | Packet constructor: writes BasePacket::vftable, stores packet id 0xba (186) at this+4, installs PacketClientMembershipVipInfo::vftable, seeds this+8 from DAT_01762a34, copies DAT_01be4850/DAT_01be4854 into this+0x10/0x1… |
| `008b19f0` | `PacketClientMembershipVipInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientMembershipVipInfo::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
