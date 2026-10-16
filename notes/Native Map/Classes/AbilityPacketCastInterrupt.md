# Class: AbilityPacketCastInterrupt

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2e3c0` | `AbilityPacketCastInterrupt::AbilityPacketCastInterrupt` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0x12 and then stores AbilityPacketCastInterrupt::vftable at +0. |
| `00a2e420` | `AbilityPacketCastInterrupt::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketCastInterrupt::vftable at 0181394c … |
