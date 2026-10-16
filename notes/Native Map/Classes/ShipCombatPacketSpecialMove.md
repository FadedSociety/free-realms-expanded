# Class: ShipCombatPacketSpecialMove

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca2e80` | `ShipCombatPacketSpecialMove::Ctor` | ✅ | ctor | Calls the shared packet base constructor with sub-opcode 0x16, installs the SpecialMove vtable, and clears its payload field. |
| `00ca3d80` | `ShipCombatPacketSpecialMove::ScalarDeletingDtor` | ✅ | dtor | Restores the base packet vtables, frees the allocation when the low deleting flag is set, and otherwise returns the packet pointer. |
