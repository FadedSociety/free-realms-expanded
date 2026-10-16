# Class: AbilityPacketPulseLocationTargeting

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d430` | `AbilityPacketPulseLocationTargeting::AbilityPacketPulseLocationTargeting` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xf, stores AbilityPacketPulseLocationTargeting::vftable, clears the byte at +0xc and copies the global DAT_017eb904 into +0x10. |
| `00a2e2e0` | `AbilityPacketPulseLocationTargeting::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketPulseLocationTargeting::vftable at … |
