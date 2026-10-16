# Class: CombatPacketAttackTargetDamage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a29dc0` | `CombatPacketAttackTargetDamage::CombatPacketAttackTargetDamage` | ✅ | ctor | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 4, stores CombatPacketAttackTargetDamage::vftable and zeroes the byte at +0x24. |
| `00a29e20` | `CombatPacketAttackTargetDamage::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of CombatPacketAttackTargetDamage::vftable at 018138… |
