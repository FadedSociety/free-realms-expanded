# Class: NameChange::NameChangeResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16100` | `NameChangeResponsePacket_Ctor` | ✅ | ctor | Calls FUN_00a0ff60(4), installs NameChange::NameChangeResponsePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member and returns this. |
| `00a16160` | `NameChangeResponsePacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a15030 and frees the packet with _free when bit 0 of the flag argument is set. |
