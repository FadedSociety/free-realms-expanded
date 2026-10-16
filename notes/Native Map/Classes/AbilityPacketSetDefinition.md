# Class: AbilityPacketSetDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2ccf0` | `AbilityPacketSetDefinition::AbilityPacketSetDefinition` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 5, stores AbilityPacketSetDefinition::vftable, zeroes +0xc and allocates an 8-byte child object (FUN_00d833a6 with size 8, constructed by FUN_01045f60) st… |
| `00a2de10` | `AbilityPacketSetDefinition::~AbilityPacketSetDefinition` | ✅ | dtor | Stores AbilityPacketSetDefinition::vftable, destroys and frees the 8-byte child object at +0x10 (FUN_01046050 then _free), then resets the vtable to BaseAbilityPacket::vftable and BasePacket::vftable. |
| `00a2de90` | `AbilityPacketSetDefinition::ScalarDeletingDtor` | 🟢 | dtor | Runs the destructor 00a2de10 and then releases the object with _free when bit 0 of the flag argument is set. |
