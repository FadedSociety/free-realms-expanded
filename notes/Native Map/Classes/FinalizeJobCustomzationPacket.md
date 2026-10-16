# Class: FinalizeJobCustomzationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a842a0` | `FinalizeJobCustomzationPacketCtor` | ✅ | ctor | Constructor of the finalize-job-customization packet: runs the JobCustomizationBasePacket family constructor 008a4e90(2) (which installs BasePacket's vtable and stores the family class id 155 and this sub-opcode 2) and … |
| `00a843f0` | `FinalizeJobCustomzationPacketScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of FinalizeJobCustomzationPacket: rewrites the object's vtable to ClientGateway::BasePacket and frees it with _free when the delete flag bit 0 is set. |
| `00a84c00` | `PackJobCustomizationPayload` | 🟢 | method | Packs the caller packet into the bounded byte writer: calls 00a84a30 for the header, writes the dword at this+0xc, then writes 0x12 dwords from this+0x10 through this+0x54, advancing the cursor by each write length. |
