# Class: PacketGeneratePortraitRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad3190` | `PacketGeneratePortraitRequest::Ctor` | ✅ | ctor | Calls the BaseFotomatPacket constructor with sub-opcode 1, installs PacketGeneratePortraitRequest vtable, and clears one member. |
| `00ad35e0` | `PacketGeneratePortraitRequest::ScalarDeletingDtor` | ✅ | dtor | Rewrites the object vptr through BaseFotomatPacket then BasePacket and frees this when the deleting-destructor flag bit is set. |
