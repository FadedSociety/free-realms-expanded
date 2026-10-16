# Class: CommandPacketInteractRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2c50` | `CommandPacketInteractRequest::~CommandPacketInteractRequest` | ✅ | dtor | Destructor body: restores BaseCommandPacket::vftable then BasePacket::vftable; callers 0092efe0 and 00934f90 construct CommandPacketInteractRequest (via FUN_008b6bc0) and destroy it with this function. |
| `008b6bc0` | `— ⚠reject` | ⚪ | ctor | Calls the command family constructor 008a2820(8, 0), installs CommandPacketInteractRequest::vftable and stores param_2 into +0x10, param_3 into +0x14 and the byte param_4 at +0x18. |
| `008b6c40` | `CommandPacketInteractRequest::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
