# Class: BaseBroadcastPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1b30` | `BaseBroadcastPacket::~BaseBroadcastPacket` | ✅ | dtor | Destructor body of the broadcast-packet base: rewrites its own vtable to BaseBroadcastPacket::vftable, runs the SoeUtil::IStringFixed<char,256> destructor body FUN_00402140 on the packet's fixed-string member, steps the… |
| `008d1bb0` | `BaseBroadcastPacket::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the broadcast-packet vtable): runs BaseBroadcastPacket::~BaseBroadcastPacket (008d1b30) and frees the object through the CRT _free thunk when bit 0 of the flag argument is se… |
| `008dd380` | `BaseBroadcastPacket::BaseBroadcastPacket` | ✅ | ctor | Base packet constructor: installs BasePacket::vftable, stores family opcode 0x4d (77) at +4, installs BaseBroadcastPacket::vftable, clears sub-opcode +8, calls 0070bac0 with ECX=this+0xc and constructs StringFixed<256> … |
| `008dd400` | `BaseBroadcastPacket::BaseBroadcastPacket_SubOpcode` | ✅ | ctor | Base packet constructor taking a sub-opcode: installs BasePacket::vftable, stores family opcode 0x4d (77) at +4, installs BaseBroadcastPacket::vftable, stores the parameter at +8, calls 0070bac0 with ECX=this+0xc and co… |
