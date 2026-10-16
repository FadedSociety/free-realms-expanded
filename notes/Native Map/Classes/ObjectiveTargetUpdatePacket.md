# Class: ObjectiveTargetUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a88080` | `ObjectiveTargetUpdatePacket::Construct` | ✅ | ctor | Constructs an ObjectiveTargetUpdatePacket: calls the BaseUiPacket constructor 00a86640 with sub-opcode 0xe, stores ObjectiveTargetUpdatePacket::vftable at *this, clears the byte at +0xc and the dwords at +0x10, +0x14, +… |
| `00a88120` | `ObjectiveTargetUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of ObjectiveTargetUpdatePacket: stores BasePacket::vftable into *this, then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
