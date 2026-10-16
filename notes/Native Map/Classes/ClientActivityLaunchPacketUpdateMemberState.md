# Class: ClientActivityLaunchPacketUpdateMemberState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4aa70` | `ClientActivityLaunchPacketUpdateMemberState::ClientActivityLaunchPacketUpdateMe…` | ✅ | ctor | Constructor: calls the family base constructor 009c0c30 with sub-opcode 9 and the two header arguments, installs ClientActivityLaunchPacketUpdateMemberState::vftable and zeroes the state byte at +0x18. |
| `00c4aae0` | `ClientActivityLaunchPacketUpdateMemberState::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4b9e0` | `UpdateMemberState_ReadFields` | 🟢 | method | Field reader for the member-state update: shared header read 009c1e00, then a 32-bit member id into +0x14 and one byte into +0x18 with stream bound checks. |
