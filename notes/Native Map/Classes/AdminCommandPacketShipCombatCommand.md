# Class: AdminCommandPacketShipCombatCommand

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a57a30` | `AdminCommandPacketShipCombatCommand::Ctor` | ✅ | ctor | Calls BaseCommandPacket construction with command identifier 0x491 and the ShipCombat string, installs the concrete vtable, initializes numeric fields, and constructs a string member; the base packet opcode is 0x1a. |
| `00a597f0` | `AdminCommandPacketShipCombatCommand::ScalarDeletingDtor` | ✅ | dtor | Calls the packet cleanup helper, frees the object when flag bit 0 is set, otherwise returns its pointer. |
