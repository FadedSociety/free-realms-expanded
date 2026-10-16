# Class: PacketSendZoneDetails

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cebc0` | `PacketSendZoneDetails::PacketSendZoneDetails` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores the opcode 0x2b (43) into the packet id slot +4, installs PacketSendZoneDetails::vftable, initializes two embedded SoeUtil::IString members (+8 and +0x20), sets t… |
| `008cec60` | `PacketSendZoneDetails::~PacketSendZoneDetails` | ✅ | dtor | Destructor body: rewrites PacketSendZoneDetails::vftable, releases the two embedded SoeUtil::IString members at +0x20 and +8 (capacity check + locked refcount decrement + virtual slot +8), then resets the base vtable to… |
| `008ced20` | `PacketSendZoneDetails::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the PacketSendZoneDetails destructor body FUN_008cec60 and frees the object with _free when bit 0 of the flag argument is set. |
