# Class: SoeUtil::List<AmbientRandomEffect,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f7290` | `ListSlot2_AllocNode ?` | 🟡 | virtual | Virtual slot [2] of SoeUtil::List<AmbientRandomEffect,-1>::vftable (no arguments): calls 006fb9c0, then 006fb860 and the slot [1] call on the thread manager, else on DAT_01bdf280, else 006fb950(0x90,0x10). Result left i… |
| `009f72d0` | `ListSlot3_FreeAlign16 ?` | 🟡 | virtual | Virtual slot [3] of SoeUtil::List<AmbientRandomEffect,-1>::vftable (one stack argument, ret 4; bytes 8b4424046a1050e8...): pushes 0x10 and the argument, calls 00401c00 (FreeMemory with alignment 16), pops 8 bytes and re… |
| `009f8300` | `ListAmbientRandomEffect_DtorBody` | ✅ | dtor | Destructor body (fastcall, bytes checked): stores SoeUtil::List<AmbientRandomEffect,-1>::vftable into [this], then while the head at this+4 is non-null calls 009f7c70 with ECX = this and the head as its argument, which … |
| `009f8330` | `ListAmbientRandomEffect_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot [0] of SoeUtil::List<AmbientRandomEffect,-1>::vftable (thiscall, bytes checked by shape): stores the list vtable, drains the head records through 009f7c70 (ECX = this) while the head is … |
