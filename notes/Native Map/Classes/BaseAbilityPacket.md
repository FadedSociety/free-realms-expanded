# Class: BaseAbilityPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2cb10` | `BaseAbilityPacket::BaseAbilityPacket` | ✅ | ctor | Writes BasePacket::vftable then BaseAbilityPacket::vftable at +0, family id 0x24 at +4 and a zero sub-opcode at +8. |
| `00a2cb70` | `— ⚠reject` | ⚪ | ctor | Calls the family base initializer and stores the caller-supplied sub-opcode at +8. |
| `00a2cbe0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2cc30` | `— ⚠reject` | ⚪ | dtor | Resets both vtable entries and, when bit 0 of the flag argument is set, releases the object with _free. |
| `00a2cca0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2ce20` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2ced0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2cf90` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d070` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d120` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d170` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d220` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d2d0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d3e0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d4a0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d550` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
| `00a2d5a0` | `BaseAbilityPacket::~BaseAbilityPacket` | ✅ | dtor | Resets the vtable from BaseAbilityPacket::vftable to BasePacket::vftable and returns; nothing is freed. |
