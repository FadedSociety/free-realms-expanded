# Class: ShipCombatPacketSpawnProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a59660` | `ShipCombatPacketSpawnProjectile::Ctor` | ✅ | ctor | Base-initializes with discriminator 0x1a, installs the packet vtable, clears fields, and initializes a member. |
| `00a596e0` | `ShipCombatPacketSpawnProjectile::ScalarDeletingDtor` | ✅ | dtor | Calls the projectile-packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |
