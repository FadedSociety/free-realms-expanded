# Class: BattleMagesPacketCreateProxiedPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6fd30` | `BattleMagesPacketCreateProxiedPlayer::BattleMagesPacketCreateProxiedPlayer` | ✅ | ctor | Constructor for sub-opcode 4: calls the shared base constructor with 4, stores BattleMagesPacketCreateProxiedPlayer::vftable at +0, initialises +0x10/+0x14 from the invalid-handle constant DAT_01be9940/DAT_01be9944 (the… |
| `00b6fdd0` | `BattleMagesPacketCreateProxiedPlayer::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the create-proxied-player packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
| `00b70ee0` | `BattleMagesPacketCreateProxiedPlayer::ReadBody` | 🟢 | method | Body reader for the sub-opcode 4 packet: first reads the two header words with 00b70e40, then copies an 8-byte handle from the stream into packet+0x10/+0x14, a 32-bit value into packet+0x18 and one byte into packet+0x1c… |
