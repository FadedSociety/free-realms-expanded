# Class: BattleMagesPacketRequestAttack

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6ffb0` | `BattleMagesPacketRequestAttack::BattleMagesPacketRequestAttack ⚠downgrade` | 🟢 | ctor | Constructor for sub-opcode 9: calls the common header constructor, installs the RequestAttack vtable, writes the invalid handle constant to +0x10/+0x14 and initializes +0x18 to -1. The send path later fills the handle p… |
| `00b70020` | `BattleMagesPacketRequestAttack::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the RequestAttack packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
