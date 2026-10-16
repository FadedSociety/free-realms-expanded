# Class: CombatPacketAttackTargetDodged

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a29f60` | `— ⚠reject` | ⚪ | ctor | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 6 and then stores CombatPacketAttackTargetDodged::vftable at +0. |
| `00a29fc0` | `CombatPacketAttackTargetDodged::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of CombatPacketAttackTargetDodged::vftable at 018138… |
