# Class: GuiRegistry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081faa0` | `GuiRegistry_Ctor` | ✅ | ctor | Fastcall constructor (ecx=this). Stores ClientLuaOwner and then GuiRegistry vtables, zeroes the 1000-entry (4000-byte) and 1024-entry (0x1000-byte) tables and counters, builds the handler maps through 0081c270, 0081c2f0… |
| `0081ff10` | `GuiRegistry_DtorBody` | ✅ | dtor | Destructor body (does not free the object): restores GuiRegistry::vftable, drains the 64-bucket map at +0xdff0 through 0081b940, unlinks the 1024-bucket lists through 0081fd60, 0081fe80 and 0081fde0, destroys the IStrin… |
| `008200a0` | `GuiRegistry_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081ff10 (the GuiRegistry destructor body) on this and frees this through the _free thunk when flag bit 0 is set. Returns this. |
