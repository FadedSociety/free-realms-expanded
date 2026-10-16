# Class: BaseCombatPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a29780` | `BaseCombatPacket::BaseCombatPacket` | ✅ | ctor | Writes BasePacket::vftable then BaseCombatPacket::vftable at +0, family id 0x20 at +4 and a zero sub-opcode at +8. |
| `00a297e0` | `BaseCombatPacket::BaseCombatPacketSubOpcode` | ✅ | ctor | Calls the family base initializer and stores the caller-supplied sub-opcode at +8. |
| `00a29850` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a298a0` | `BaseCombatPacket::ScalarDeletingDtor` | ✅ | dtor | Resets both vtable entries and, when bit 0 of the flag argument is set, releases the object with _free. |
| `00a29910` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29960` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a299b0` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29a00` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29a50` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29aa0` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a29af0` | `BaseCombatPacket::~BaseCombatPacket` | ✅ | dtor | Resets the vtable from BaseCombatPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
