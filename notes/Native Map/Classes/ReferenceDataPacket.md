# Class: ReferenceDataPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a44f0` | `ReferenceDataPacket::ReferenceDataPacket` | ✅ | ctor | Reference-data family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 44 (0x2c) at +4, installs ReferenceDataPacket::vftable and zeroes +8; no calls. |
| `008a4550` | `ReferenceDataPacket::ReferenceDataPacketSub` | ✅ | ctor | Reference-data family constructor taking the sub-opcode: installs BasePacket::vftable, stores opcode 44 (0x2c) at +4, installs ReferenceDataPacket::vftable and stores the sub-opcode argument at +8; no calls. |
| `008a45c0` | `ReferenceDataPacket::~ReferenceDataPacket` | ✅ | dtor | Destructor body: writes ReferenceDataPacket::vftable then BasePacket::vftable into the object vtable slot and returns; 12 callers including the 00928d00 dispatcher and EH funclets. |
| `008a4610` | `ReferenceDataPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: in an SEH frame restores ReferenceDataPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
