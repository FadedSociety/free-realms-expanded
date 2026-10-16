# Class: PacketTargetClientNotOnline

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ced40` | `PacketTargetClientNotOnline::~PacketTargetClientNotOnline` | ✅ | dtor | Destructor body: rewrites PacketTargetClientNotOnline::vftable, runs the SoeUtil::IStringFixed<char,32> destructor body FUN_00703350 and resets the base vtable to BasePacket::vftable. |
| `008cedb0` | `PacketTargetClientNotOnline::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs FUN_008ced40 and frees the object with _free when bit 0 of the flag argument is set. |
| `008dc2f0` | `PacketTargetClientNotOnline::PacketTargetClientNotOnline` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0x16 (22) at +4, installs PacketTargetClientNotOnline::vftable, then calls the StringFixed<32> constructor 00703540 with ECX=this+0x10. |
