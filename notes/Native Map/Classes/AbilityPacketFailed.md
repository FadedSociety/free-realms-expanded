# Class: AbilityPacketFailed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d0c0` | `AbilityPacketFailed::AbilityPacketFailed` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 1, stores AbilityPacketFailed::vftable and zeroes the dword at +0xc. |
| `00a2e070` | `AbilityPacketFailed::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketFailed::vftable at 01813904 is this… |
