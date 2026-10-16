# Class: NameChange::BaseNameChangePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a0ff00` | `BaseNameChangePacket_CtorDefault` | ✅ | ctor | Default constructor: stores BasePacket::vftable, writes the base opcode 0xc0 (192) into the packet's opcode field at +4, then stores NameChange::BaseNameChangePacket::vftable and zeroes the sub-opcode field at +8. |
| `00a0ff60` | `BaseNameChangePacket_Ctor` | ✅ | ctor | Constructor taking the sub-opcode: stores BasePacket::vftable, writes 0xc0 (192) into the opcode field at +4, installs NameChange::BaseNameChangePacket::vftable and stores param_2 into the sub-opcode field at +8. |
| `00a0ffd0` | `BaseNameChangePacket_Dtor` | 🟢 | dtor | Destructor body: writes NameChange::BaseNameChangePacket::vftable then BasePacket::vftable into this (the MSVC base-destructor pattern); no fields are released. |
| `00a10020` | `BaseNameChangePacket_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: writes the derived vtable then BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |
| `00a10100` | `BaseNameChangePacket_Dtor2` | 🟢 | dtor | Second destructor body for the same base class: stores the derived vtable and then BasePacket::vftable; no field release. |
| `00a12040` | `BaseNameChangePacket_ScalarDeletingDtor2` | 🟢 | dtor | Scalar deleting destructor: writes NameChange::BaseNameChangePacket::vftable then BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |
| `00a14f10` | `BaseNameChangePacket_Dtor` | ✅ | dtor | Destructor of the name-change packet base: runs the EH-protected base teardown FUN_0070a790 (which releases the packet's IString member), then rewrites the object vtable to NameChange::BaseNameChangePacket and finally t… |
| `00a14f70` | `BaseNameChangePacket_Dtor_00a14f70` | ✅ | dtor | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a14fd0` | `BaseNameChangePacket_Dtor_00a14fd0` | ✅ | dtor | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a15030` | `BaseNameChangePacket_Dtor_00a15030` | ✅ | dtor | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a15090` | `BaseNameChangePacket_Dtor_00a15090` | ✅ | dtor | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a21840` | `— ⚠reject` | ⚪ | handler | Builds a BaseNameChangePacket on the stack with 00a0ff00, parses the incoming buffer (pointer at param_1, length param_2) with 00a16850 into the packet header fields (the sub-opcode lands in local_d0) and then branches:… |
