# Class: BaseGroupPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3560` | `BaseGroupPacket::BaseGroupPacket` | ✅ | ctor | Group family constructor with a fixed zero sub-opcode: installs BasePacket::vftable, stores 0x28 (40) in the packet id slot param_1[1], installs BaseGroupPacket::vftable and zeroes param_1[2]. |
| `008a35c0` | `BaseGroupPacket::BaseGroupPacketSub` | ✅ | ctor | Group family constructor taking the sub-opcode: installs BasePacket::vftable, stores 0x28 (40) in param_1[1], installs BaseGroupPacket::vftable and stores the argument in param_1[2]. |
| `008a3690` | `BaseGroupPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the group and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set; it has no external callers in the export. |
| `00c0afb0` | `BaseGroupPacket_DtorBody → GroupChangingJobPacket_DtorBody` | ✅ | dtor | Base-class destructor body: rewrites the vtable to BaseGroupPacket::vftable then BasePacket::vftable; called from the SetChangingJob handler 00c0b300 after the send and from the unwind funclet 015f4870. |
