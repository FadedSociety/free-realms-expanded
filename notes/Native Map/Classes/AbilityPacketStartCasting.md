# Class: AbilityPacketStartCasting

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2ed30` | `AbilityPacketStartCasting::AbilityPacketStartCasting` | ✅ | ctor | Calls the BaseAbilityPacket sub-opcode constructor 00a2cb70 with 3, stores AbilityPacketStartCasting::vftable, initialises +0x10..+0x1c with two copies of the default id pair DAT_01be6870/DAT_01be6874, sets +0x20 to 0, … |
| `00a2edc0` | `AbilityPacketStartCasting::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseAbilityPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityPacketStartCasting::vftable at 01813954 i… |
