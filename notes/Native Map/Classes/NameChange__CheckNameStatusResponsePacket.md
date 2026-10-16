# Class: NameChange::CheckNameStatusResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16180` | `CheckNameStatusResponsePacket_Ctor ⚠downgrade` | 🟢 | ctor | Calls FUN_00a0ff60(6), installs NameChange::CheckNameStatusResponsePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member and returns this. |
| `00a161e0` | `CheckNameStatusResponsePacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a15090 and frees the packet with _free when bit 0 of the flag argument is set. |
