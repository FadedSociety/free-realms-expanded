# Class: AbilityPacketDetonateProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2ee30` | `AbilityPacketDetonateProjectile::AbilityPacketDetonateProjectile` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xe, stores AbilityPacketDetonateProjectile::vftable, copies the default id pair DAT_01be6870/DAT_01be6874 into +0x10/+0x14 and zeroes +0x18, +0x1c and +0… |
| `00a2eeb0` | `AbilityPacketDetonateProjectile::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketDetonateProjectile::vftable at 0181… |
