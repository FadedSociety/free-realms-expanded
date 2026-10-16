# Class: ActorProjectileDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00914fe0` | `CtorWrap0090f420 → ActorProjectileDefinitionManager::Ctor` | 🟢 | ctor | thiscall (returns this). Calls FUN_0090f420(arg2, arg3) with the two stack arguments passed through, after its frame setup; the class vtables ActorProjectileDefinitionManager::vftable (01804fe8, 01804fd0) are referenced… |
| `00915050` | `ScalarDeletingDtor → ActorProjectileDefinitionManagerScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 01804fe8 (thiscall, one byte argument; bytes 56 8b f1 e8 .. ). Calls FUN_010b6340 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this in EAX. |
| `009150b0` | `ThunkThisMinus4ToDeletingDtorC` | 🟢 | virtual | Adjustor thunk (bytes 83 e9 04 e9 98 ff ff ff): subtracts 4 from ECX and jumps to FUN_00915050. It is vftable[0] of the second ActorProjectileDefinitionManager vtable (01804fd0). |
| `00915460` | `— ⚠reject` | ⚪ | virtual | Virtual at vftable[5] of 01804fe8 (thiscall, one argument). Stores the argument into a local, then calls FUN_009153e0 with ECX=&local and stack arguments (this+0xc, &arg); returns nothing. |
