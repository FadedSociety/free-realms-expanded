# Class: SoeUtil::HashList<GuiEventHandler*,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081c270` | `HashListGuiEventHandler512_Ctor` | ✅ | ctor | Fastcall constructor (ecx=this): stores the HashListMap<int,GuiEventHandler*,512,-1> vtable, zeroes [2], [3] and [4], calls _memset on the 512-entry bucket array at [5] (0x800 bytes), zeroes [1], then stores the HashLis… |
| `0081c370` | `HashListGuiEventHandler512_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081bda0 (the 512-bucket GuiEventHandler HashListMap destructor body) on this, and frees this through the _free thunk when flag bit 0 is set. Returns thi… |
