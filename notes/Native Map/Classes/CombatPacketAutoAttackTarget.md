# Class: CombatPacketAutoAttackTarget

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a29c20` | `CombatPacketAutoAttackTarget::CombatPacketAutoAttackTarget` | ✅ | ctor | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 1 and then stores CombatPacketAutoAttackTarget::vftable at +0. |
| `00a29c80` | `CombatPacketAutoAttackTarget::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of CombatPacketAutoAttackTarget::vftable at 01813880… |
