# Class: GroupMapPingPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3a60` | `GroupMapPingPacket::~GroupMapPingPacket` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the map ping with it after constructing the class from FUN_008b74e0 (group sub 16) and displaying the ping. |
| `008b74e0` | `GroupMapPingPacket::GroupMapPingPacket` | ✅ | ctor | Calls the group family initializer for top-level opcode 40 008a35c0(0x10), installs GroupMapPingPacket::vftable and stores param_2..param_5 into +0x10..+0x1c. |
| `008b7560` | `GroupMapPingPacket::ScalarDeletingDestructor` | ✅ | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
