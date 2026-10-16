# Class: RewardBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b88ad0` | `RewardBasePacket_Ctor` | ✅ | ctor | Constructor of the reward packet base with a fixed zero subtype: installs BasePacket::vftable, stores the base opcode 0x32 at +4, installs RewardBasePacket::vftable, zeroes the subtype field at +8 and initializes the So… |
| `00b88b40` | `RewardBasePacket_CtorWithSubtype` | ✅ | ctor | Reward packet base constructor taking the subtype: installs BasePacket::vftable, stores base opcode 0x32 at +4, installs RewardBasePacket::vftable, stores param_2 as the subtype at +8 and initializes the IString member … |
| `00b88bc0` | `— ⚠reject` | ⚪ | dtor | Reward packet base destructor: reinstalls the SoeUtil::IString<char> vtable at +0xc and, for the string data pointer at +0x10 with refcount word at +0x14, atomically decrements and calls the string vtable slot 2 (offset… |
| `00b88ff0` | `RewardBasePacket::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of RewardBasePacket: runs the base destructor 00b88bc0 and frees the object through the _free thunk when bit 0 of param_2 is set. |
| `00b89190` | `RewardBasePacket_ReadHeader` | ✅ | method | Base body reader of the reward family: over the bounded reader object (cursor this+8, end this+0xc, error byte this+0x10) it reads a little-endian 16-bit value into the base-opcode field at +4 and then a single byte int… |
