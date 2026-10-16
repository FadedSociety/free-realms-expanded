# Class: NameChange::CheckNameResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16080` | `CheckNameResponsePacket_Ctor` | ✅ | ctor | Calls FUN_00a0ff60(2), installs NameChange::CheckNameResponsePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member and returns this. |
| `00a160e0` | `CheckNameResponsePacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a14f70 and frees the packet with _free when bit 0 of the flag argument is set. |
