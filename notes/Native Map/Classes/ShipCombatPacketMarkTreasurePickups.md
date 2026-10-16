# Class: ShipCombatPacketMarkTreasurePickups

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a57880` | `ShipCombatPacketMarkTreasurePickups::Ctor` | ✅ | ctor | Calls the base constructor with opcode 0x1e, installs the MarkTreasurePickups vtable, and initializes a Vector2 array member with empty storage. |
| `00a597d0` | `ShipCombatPacketMarkTreasurePickups::ScalarDeletingDtor` | ✅ | dtor | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |
