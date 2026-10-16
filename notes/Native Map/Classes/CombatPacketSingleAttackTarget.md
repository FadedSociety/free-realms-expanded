# Class: CombatPacketSingleAttackTarget

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a29cf0` | `CombatPacketSingleAttackTarget::CombatPacketSingleAttackTarget` | ✅ | ctor | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 3 and then stores CombatPacketSingleAttackTarget::vftable at +0. |
| `00a29d50` | `CombatPacketSingleAttackTarget::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of CombatPacketSingleAttackTarget::vftable at 018138… |
