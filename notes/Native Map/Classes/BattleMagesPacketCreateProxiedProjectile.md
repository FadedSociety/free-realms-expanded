# Class: BattleMagesPacketCreateProxiedProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b705b0` | `BattleMagesPacketCreateProxiedProjectile::BattleMagesPacketCreateProxiedProject…` | ✅ | ctor | Constructor for sub-opcode 11: calls the shared base constructor with 0xb, stores BattleMagesPacketCreateProxiedProjectile::vftable at +0, zeroes +0xc, initialises +0x10/+0x14 from the invalid-handle constant DAT_01be99… |
| `00b70620` | `BattleMagesPacketCreateProxiedProjectile::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the create-proxied-projectile packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
