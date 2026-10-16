# Class: AbilityPacketStopAura

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2cfe0` | `AbilityPacketStopAura::AbilityPacketStopAura` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 9, stores AbilityPacketStopAura::vftable, and copies the two dwords pointed to by the second argument into +0xc and +0x10. |
| `00a2e000` | `AbilityPacketStopAura::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketStopAura::vftable at 018138fc is th… |
