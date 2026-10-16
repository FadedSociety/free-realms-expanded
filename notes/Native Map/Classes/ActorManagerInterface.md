# Class: ActorManagerInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0079c390` | `ActorManagerInterfaceDtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores ActorManagerInterface::vftable (017edf4c) at +0 and returns. Seven bytes (c7 01 imm32; c3). |
| `0079c3d0` | `ActorManagerInterfaceConstFloat ?` | 🟡 | virtual | Virtual slot [8] of ActorManagerInterface (bytes d9 05 imm32; c3): loads the float constant at 017ed7d4 onto the x87 stack and returns it. |
| `0079c400` | `ActorManagerInterfaceScalarDtor` | ✅ | dtor | Thiscall (ecx = this, one flag argument). Stores ActorManagerInterface::vftable at +0 and frees this through _free when bit 0 of the flag is set; returns this. |
