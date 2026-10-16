# Class: SoeUtil::HashList<SoundGroup*,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081c2f0` | `HashListSoundGroup64_Ctor` | ✅ | ctor | Fastcall constructor (ecx=this): stores the HashListMap<int,SoundGroup*,64,-1> vtable, zeroes [2], [3] and [4], calls _memset on the 64-entry bucket array at [5] (0x100 bytes), zeroes [1], then stores the HashList<Sound… |
| `0081c390` | `HashListSoundGroup64_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081be10 (the 64-bucket SoundGroup HashListMap destructor body) on this, and frees this through the _free thunk when flag bit 0 is set. Returns this. |
