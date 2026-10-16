# Class: ClientActivityLaunchPacketRemoveMember

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c49790` | `ClientActivityLaunchPacketRemoveMember::ClientActivityLaunchPacketRemoveMember` | ✅ | ctor | Constructor: calls the family base constructor 009c0c30 with sub-opcode 8 and the two header arguments, installs ClientActivityLaunchPacketRemoveMember::vftable and zeroes the dword at +0x18. |
| `00c4a090` | `ClientActivityLaunchPacketRemoveMember::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores BasePacket::vftable at *this and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4aeb0` | `RemoveMember_ReadFields` | 🟢 | method | Field reader for a remove-member packet: shared header read 009c1e00, then two 32-bit fields written at +0x14 and +0x18 with stream bound checks. |
