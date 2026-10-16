# Class: ClientUpdatePacketCollectionStart

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d8d0` | `ClientUpdatePacketCollectionStartCtor` | ✅ | ctor | Constructor of ClientUpdatePacketCollectionStart: runs the base constructor FUN_0089d180 with sub-opcode 8, then installs ClientUpdatePacketCollectionStart::vftable. |
| `0089d930` | `ClientUpdatePacketCollectionStartDtor` | ✅ | dtor | Destructor body: installs ClientUpdatePacketCollectionStart::vftable, then BaseClientUpdatePacket::vftable and BasePacket::vftable; no member cleanup. |
| `0089d990` | `ClientUpdatePacketCollectionStart::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor of the family-38 sub-8 collection-start packet: writes ClientUpdatePacketCollectionStart::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable, and calls _free(this) wh… |
