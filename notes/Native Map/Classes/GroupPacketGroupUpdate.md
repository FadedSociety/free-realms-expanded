# Class: GroupPacketGroupUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3910` | `GroupPacketGroupUpdate::GroupPacketGroupUpdate` | ✅ | ctor | Constructor: calls FUN_008a35c0(8) so the packet is group opcode 40 with sub-opcode 8, installs GroupPacketGroupUpdate::vftable and zeroes param_1[3]. |
| `008a3970` | `GroupPacketGroupUpdate::~GroupPacketGroupUpdate` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the group update with it. |
| `008b7280` | `GroupPacketGroupUpdate::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
