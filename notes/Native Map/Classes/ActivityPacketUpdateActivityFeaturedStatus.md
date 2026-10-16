# Class: ActivityPacketUpdateActivityFeaturedStatus

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdf060` | `— ⚠reject` | ⚪ | ctor | Constructs the packet with FUN_008a21f0(5) (the BaseActivityPacket constructor with activity sub-opcode 5) and installs ActivityPacketUpdateActivityFeaturedStatus::vftable. |
| `00bdf0c0` | `ActivityPacketUpdateActivityFeaturedStatus::~ActivityPacketUpdateActivityFeatur…` | 🟢 | dtor | Destructor body: stores BaseActivityPacket::vftable, BaseActivityServicePacket::vftable and finally BasePacket::vftable at object+0, with no member cleanup. |
| `00be0190` | `ActivityPacketUpdateActivityFeaturedStatus::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vftable to BaseActivityPacket, then BaseActivityServicePacket, then BasePacket; frees this through _free (thunk) only when bit 0 of the flag argument is set; returns this. |
