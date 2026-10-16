# Class: CinematicStartLookAtPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a86e40` | `— ⚠reject` | ⚪ | ctor | Constructor of CinematicStartLookAtPacket: runs the BaseUiPacket constructor with sub-opcode 0x10 (class id 47), stores the CinematicStartLookAtPacket vtable, embeds two Target objects by storing the Target::vftable at … |
| `00a86ec0` | `CinematicStartLookAtPacketDtorBody` | 🟢 | dtor | Destroys the embedded Target at this+0x14, then the Target at this+0x0c, through 0101b370, and restores ClientGateway::BasePacket in the object vtable. |
| `00a88200` | `CinematicStartLookAtPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of CinematicStartLookAtPacket: calls 00a86ec0 (dtor body that destroys its two embedded Target members), then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns thi… |
