# Class: BattleMagesPacketKickProxiedPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6fe40` | `BattleMagesPacketKickProxiedPlayer::BattleMagesPacketKickProxiedPlayer ⚠downgra…` | 🟢 | ctor | Constructor for sub-opcode 5: calls the shared base constructor with 5, stores BattleMagesPacketKickProxiedPlayer::vftable at +0, and initialises +0x10/+0x14 from the invalid-handle constant DAT_01be9940/DAT_01be9944 (t… |
| `00b6fed0` | `BattleMagesPacketKickProxiedPlayer::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the kick-proxied-player packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
