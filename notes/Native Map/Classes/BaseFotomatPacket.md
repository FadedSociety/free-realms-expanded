# Class: BaseFotomatPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a23f0` | `BaseFotomatPacket::BaseFotomatPacket` | ✅ | ctor | Fotomat family constructor: installs BasePacket::vftable, stores 0x9c (156) in the packet id slot param_1[1], installs BaseFotomatPacket::vftable and stores the sub-opcode argument in param_1[2]. |
| `008a2460` | `BaseFotomatPacket::~BaseFotomatPacket` | ✅ | dtor | Destructor body: restores BaseFotomatPacket::vftable then BasePacket::vftable. |
| `008a24b0` | `BaseFotomatPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the Fotomat and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `0091d920` | `BaseFotomatPacket::~BaseFotomatPacket` | ✅ | dtor | Destructor: runs the embedded-buffer cleanup FUN_0091d7d0 and then restores BaseFotomatPacket::vftable followed by BasePacket::vftable. |
| `00ad31f0` | `BaseFotomatPacket::Dtor` | ✅ | dtor | Restores BaseFotomatPacket vtable then BasePacket vtable and returns. |
