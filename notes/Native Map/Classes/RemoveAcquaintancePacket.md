# Class: RemoveAcquaintancePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16bb0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the acquaintance base constructor FUN_00a10210 with sub-opcode 2 (base opcode 0x48), stores RemoveAcquaintancePacket::vftable, zeroes the fields at +0x14 and +0x18, installs the IString vtable at +0xc… |
| `00a16c30` | `RemoveAcquaintancePacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): delegates to the destructor body 00a15150, which releases the IString member at +0xc, and frees the packet with _free when bit 0 of the flag argument is set. |
