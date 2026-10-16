# Class: BattleMagesPacketRegisterPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6fc50` | `BattleMagesPacketRegisterPlayer::BattleMagesPacketRegisterPlayer` | ✅ | ctor | Constructor for sub-opcode 3: calls the shared base constructor with 3, stores BattleMagesPacketRegisterPlayer::vftable at +0, and copies the two dwords of the static invalid-handle constant DAT_01be9940/DAT_01be9944 in… |
| `00b6fcc0` | `BattleMagesPacketRegisterPlayer::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the RegisterPlayer packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
