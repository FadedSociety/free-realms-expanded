# Class: BattleMagesPacketUpdateProxiedProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b70100` | `BattleMagesPacketUpdateProxiedProjectile::BattleMagesPacketUpdateProxiedProject…` | ✅ | ctor | Constructor for sub-opcode 13: calls the shared base constructor with 0xd, stores BattleMagesPacketUpdateProxiedProjectile::vftable at +0 and zeroes the field at +0xc. |
| `00b70160` | `BattleMagesPacketUpdateProxiedProjectile::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the update-proxied-projectile packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
