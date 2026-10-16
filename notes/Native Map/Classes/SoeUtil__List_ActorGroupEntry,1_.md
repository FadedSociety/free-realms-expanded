# Class: SoeUtil::List<ActorGroupEntry,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0079aa00` | `ListActorGroupEntry1Ctor` | ✅ | ctor | Fastcall (ecx = this). Stores SoeUtil::List<ActorGroupEntry,-1>::vftable, zeroes +4, +8, +0xc, +0x10, +0x14 and the byte at +0x20, stores SoeUtil::List<ActorGroupEntry,1>::vftable at +0, points +0x18 at the inline buffe… |
| `0079b400` | `ListActorGroupEntry1AllocThunk` | 🟢 | virtual | Adjustor thunk (bytes 83 c1 10; e9 rel32): adds 0x10 to ecx and jumps to 0079b350 (the second pool pop). Vtable slot [2] of SoeUtil::List<ActorGroupEntry,1>. |
| `0079b410` | `ListActorGroupEntry1FreeThunk` | 🟢 | virtual | Adjustor thunk (bytes 83 c1 10; e9 rel32): adds 0x10 to ecx and jumps to 0079b3a0 (the second pool push). Vtable slot [3] of SoeUtil::List<ActorGroupEntry,1>. |
| `0079b420` | `ListActorGroupEntry1DtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores SoeUtil::List<ActorGroupEntry,1>::vftable at +0, unlinks and frees the head nodes through 0079a8d0 until the head at +4 is NULL, zeroes +0x18 and +0x1c, frees the block tree at +0x10 throug… |
| `0079b4f0` | `ListActorGroupEntry1ScalarDtor` | ✅ | dtor | Thiscall (ecx = this, one flag argument). Calls 0079b420 (the destructor body of List<ActorGroupEntry,1>) on this, frees this through _free when bit 0 of the flag is set, and returns this. |
