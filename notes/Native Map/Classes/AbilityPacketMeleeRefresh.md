# Class: AbilityPacketMeleeRefresh

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d1c0` | `AbilityPacketMeleeRefresh::AbilityPacketMeleeRefresh` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 0xb, stores AbilityPacketMeleeRefresh::vftable and zeroes the dword at +0xc. |
| `00a2e1e0` | `AbilityPacketMeleeRefresh::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketMeleeRefresh::vftable at 0181390c i… |
