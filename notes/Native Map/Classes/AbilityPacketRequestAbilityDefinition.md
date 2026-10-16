# Class: AbilityPacketRequestAbilityDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d270` | `AbilityPacketRequestAbilityDefinition::AbilityPacketRequestAbilityDefinition` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xc, stores AbilityPacketRequestAbilityDefinition::vftable and sets the dword at +0xc to 0xffffffff. |
| `00a2e250` | `AbilityPacketRequestAbilityDefinition::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketRequestAbilityDefinition::vftable a… |
