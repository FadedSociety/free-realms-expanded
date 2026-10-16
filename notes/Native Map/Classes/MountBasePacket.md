# Class: MountBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089f9c0` | `MountBasePacket::MountBasePacket` | ✅ | ctor | Family constructor taking the sub-opcode: installs BasePacket::vftable, writes 0xa8 (168) into the packet id slot param_1[1], installs MountBasePacket::vftable and stores the sub-opcode argument in param_1[2]. No payloa… |
| `008b36d0` | `— ⚠reject` | ⚪ | dtor | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. No direct callers, so it is only reachable through a vtable slot. |
| `00b03710` | `MountBasePacket::MountBasePacket` | ✅ | ctor | Family constructor without a sub-opcode: installs BasePacket::vftable, writes the mount family opcode 0xa8 (168) into the packet id slot [1] at +4, installs MountBasePacket::vftable and leaves the sub-opcode slot [2] at… |
