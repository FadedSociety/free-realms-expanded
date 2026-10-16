# Class: CombatPacketEnableBossDisplay

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2a030` | `CombatPacketEnableBossDisplay::CombatPacketEnableBossDisplay` | ✅ | ctor | Calls the BaseCombatPacket sub-opcode constructor 00a297e0 with 9 and then stores CombatPacketEnableBossDisplay::vftable at +0. |
| `00a2a090` | `CombatPacketEnableBossDisplay::ScalarDeletingDtor` | 🟢 | dtor | Resets the vtable entries (BaseCombatPacket::vftable then BasePacket::vftable) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of CombatPacketEnableBossDisplay::vftable at 018138a… |
