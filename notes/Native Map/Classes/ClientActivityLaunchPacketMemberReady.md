# Class: ClientActivityLaunchPacketMemberReady

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c498a0` | `ClientActivityLaunchPacketMemberReady::ClientActivityLaunchPacketMemberReady` | ✅ | ctor | Constructor: calls the family base constructor 009c0c30 with sub-opcode 0x12 (18) and the two header arguments, then installs ClientActivityLaunchPacketMemberReady::vftable; no extra members are initialised. |
| `00c4a0f0` | `ClientActivityLaunchPacketMemberReady::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set; the bytes contain no other call, so no member needs destruction. |
