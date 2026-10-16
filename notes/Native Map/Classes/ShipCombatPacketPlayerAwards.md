# Class: ShipCombatPacketPlayerAwards

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a576b0` | `ShipCombatPacketPlayerAwards::Ctor` | ✅ | ctor | Calls the common packet constructor with opcode 0x18, installs PlayerAwards packet vtable, initializes an embedded ShipCombatAwardData array and zeros its storage fields. |
| `00a59640` | `ShipCombatPacketPlayerAwards::ScalarDeletingDtor` | ✅ | dtor | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |
