# Class: BattleMagesPacketDestroyProxiedProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6f670` | `BattleMagesPacketDestroyProxiedProjectile::BattleMagesPacketDestroyProxiedProje…` | ✅ | ctor | Constructor for sub-opcode 12: calls the shared base constructor with 0xc, stores BattleMagesPacketDestroyProxiedProjectile::vftable at +0 and zeroes the field at +0xc. |
| `00b70090` | `BattleMagesPacketDestroyProxiedProjectile::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the destroy-proxied-projectile packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
