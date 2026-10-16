# Class: NameChange::CheckNamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16000` | `CheckNamePacket_Ctor` | ✅ | ctor | Calls the base packet constructor FUN_00a0ff60(1), installs NameChange::CheckNamePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member and returns this. |
| `00a16060` | `CheckNamePacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0, bytes 568bf1 e8a8eeffff f644240801 7409 56 e8...): calls the BaseNameChangePacket destructor copy 00a14f10 and frees the object with _free when bit 0 of the flag argument is se… |
| `00a16900` | `— ⚠reject` | ⚪ | ctor | Constructor overload: builds the base with sub-opcode 1, stores the two dwords and the trailing flag handed in as arguments (fields at +0xc through +0x14 and the byte at +0xa0) and initialises the name member through FU… |
| `00a16990` | `CheckNamePacket_CtorWithName` | ✅ | ctor | Calls FUN_00a0ff60(1), stores type and identifier dwords at +0xc through +0x14, installs the packet vftable, calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member, clears the byte at +0xa0, assigns the… |
