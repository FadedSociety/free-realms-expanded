# Class: CombatPacketAttackProcessed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2a470` | `CombatPacketAttackProcessed::CombatPacketAttackProcessed` | ✅ | ctor | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 7, stores CombatPacketAttackProcessed::vftable and zeroes the dwords at +0x28 and +0x30. |
| `00a2a4d0` | `CombatPacketAttackProcessed::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of CombatPacketAttackProcessed::vftable at 018138b0 … |
