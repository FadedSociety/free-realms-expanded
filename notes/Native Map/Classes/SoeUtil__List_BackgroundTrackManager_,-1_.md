# Class: SoeUtil::List<BackgroundTrackManager*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f7170` | `ListSlot2_AllocNode ?` | 🟡 | virtual | Virtual slot [2] of SoeUtil::List<BackgroundTrackManager*,-1>::vftable (no arguments): calls 006fb9c0, then 006fb860 (current thread manager); when that is non-null it calls the manager's vtable slot [1]; otherwise, whe… |
| `009f71b0` | `ListSlot3_FreeAlign4 ?` | 🟡 | virtual | Virtual slot [3] of SoeUtil::List<BackgroundTrackManager*,-1>::vftable (one stack argument, ret 4; bytes 8b4424046a0450e8...): pushes 4 and the argument and calls 00401c00 (FreeMemory with alignment 4), pops 8 bytes and… |
| `009f8110` | `ListBackgroundTrackManager_DtorBody` | ✅ | dtor | Destructor body as an 11-byte thunk (c7 01 vtable; e9 rel32, bytes checked): stores SoeUtil::List<BackgroundTrackManager*,-1>::vftable into [this], then jumps to 009f7fc0, which drains and frees the list nodes. |
| `009f8140` | `ListBackgroundTrackManager_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot [0] of SoeUtil::List<BackgroundTrackManager*,-1>::vftable (thiscall, ret 4): stores the list vtable into [this], calls 009f7fc0 with ECX = this to drain and free the nodes, then calls _f… |
