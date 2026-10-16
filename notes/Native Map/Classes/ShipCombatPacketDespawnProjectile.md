# Class: ShipCombatPacketDespawnProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a54fe0` | `ShipCombatPacketDespawnProjectile::Ctor` | ✅ | ctor | Calls the shared ship-combat packet base constructor with 0x1b, installs the DespawnProjectile vtable, clears two fields, and copies default four-dword state values. |
| `00a55060` | `ShipCombatPacketDespawnProjectile::ScalarDeletingDtor` | ✅ | dtor | Rewrites the object's vtable to BaseShipCombatPacket then BasePacket; if flag bit 0 is set, frees the object, and returns its address otherwise. |
