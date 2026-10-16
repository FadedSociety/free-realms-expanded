# Class: AbilityPacketUpdateAbilityExperience

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2cf20` | `AbilityPacketUpdateAbilityExperience::AbilityPacketUpdateAbilityExperience` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 8, stores AbilityPacketUpdateAbilityExperience::vftable, and zeroes the six dwords from +0xc through +0x20 after setting +0xc to 0xffffffff. |
| `00a2df90` | `AbilityPacketUpdateAbilityExperience::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketUpdateAbilityExperience::vftable at… |
