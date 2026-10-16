# Class: AbilityPacketAbilityDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d320` | `AbilityPacketAbilityDefinition::AbilityPacketAbilityDefinition` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xd, stores AbilityPacketAbilityDefinition::vftable and constructs the embedded definition object with FUN_01064600 (which stores AbilityDefinition::vftab… |
| `00a2e2c0` | `AbilityPacketAbilityDefinition::ScalarDeletingDtor` | 🟢 | dtor | Runs the destructor 00a2d380 (which destroys the embedded definition member and restores the base vtables) and then releases the object with _free when bit 0 of the flag argument is set. |
