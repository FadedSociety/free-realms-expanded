# Class: ClientUpdatePacketCollectionAddEntry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089db30` | `ClientUpdatePacketCollectionAddEntry::ClientUpdatePacketCollectionAddEntry` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(10) (family 0x26, sub-opcode 10), installs ClientUpdatePacketCollectionAddEntry::vftable, then constructs an embedded IconData member at param_1[… |
| `0089dba0` | `ClientUpdatePacketCollectionAddEntry::~ClientUpdatePacketCollectionAddEntry` | ✅ | dtor | Destructor body: writes ClientUpdatePacketCollectionAddEntry::vftable, then the embedded icon member's base vtable at param_1[4] (Loggable::vftable), then BaseClientUpdatePacket::vftable and BasePacket::vftable. No memo… |
| `0089dc10` | `ClientUpdatePacketCollectionAddEntry::ScalarDeletingDestructor` | 🟢 | dtor | Thin scalar deleting destructor: calls FUN_0089dba0 (the full destructor, which restores the packet vtable and the embedded icon member vtable) and then calls _free(this) when bit 0 of param_2 is set. |
