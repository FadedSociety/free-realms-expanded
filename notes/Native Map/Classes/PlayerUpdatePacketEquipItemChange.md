# Class: PlayerUpdatePacketEquipItemChange

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d05d0` | `PlayerUpdatePacketEquipItemChange::~PlayerUpdatePacketEquipItemChange` | ✅ | dtor | Destructor of the EquipItemChange packet: rewrites PlayerUpdatePacketEquipItemChange::vftable, runs the SoeUtil::IStringFixed<char,128> destructor body FUN_0070a900 (three fixed-string members), then rewrites BasePlayer… |
| `008d0640` | `PlayerUpdatePacketEquipItemChange::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the EquipItemChange destructor body FUN_008d05d0 and frees the object with _free when bit 0 of the flag argument is set. |
| `008dcb40` | `PlayerUpdatePacketEquipItemChange::PlayerUpdatePacketEquipItemChange` | ✅ | ctor | Packet constructor: calls family initializer 0089fbb0 with sub-opcode 6 (family 35), installs PlayerUpdatePacketEquipItemChange::vftable, calls 0101cd40 with ECX=this+0x18 and 0070b770 with ECX=this+0x20. |
