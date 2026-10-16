# Class: AbilityPacketClientMoveAndCast

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2cda0` | `AbilityPacketClientMoveAndCast::AbilityPacketClientMoveAndCast` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 6, stores AbilityPacketClientMoveAndCast::vftable, and zeroes +0x10..+0x20 with +0x24 set to 0xffffffff. |
| `00a2deb0` | `AbilityPacketClientMoveAndCast::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketClientMoveAndCast::vftable at 01813… |
