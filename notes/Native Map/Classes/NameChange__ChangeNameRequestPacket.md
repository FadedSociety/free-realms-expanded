# Class: NameChange::ChangeNameRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16a30` | `— ⚠reject` | ⚪ | ctor | Constructor: base sub-opcode 3, then stores the guid pair and the trailing flag from the arguments (fields at +0xc through +0x14) and initialises the name member through FUN_008ea990. |
| `00a16ab0` | `ChangeNameRequestPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a14fd0 and frees the packet with _free when bit 0 of the flag argument is set. |
| `00a16ad0` | `ChangeNameRequestPacket_CtorWithName` | ✅ | ctor | Calls FUN_00a0ff60(3), stores type and identifier dwords at +0xc through +0x14, installs the packet vftable, calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member, assigns the name with FUN_00402ce0 on… |
