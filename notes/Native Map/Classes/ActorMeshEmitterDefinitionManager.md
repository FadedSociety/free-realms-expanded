# Class: ActorMeshEmitterDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00914af0` | `CtorWrap0090ef40 → ActorMeshEmitterDefinitionManager::Ctor` | 🟢 | ctor | thiscall (returns this in EAX). Under an SEH frame calls FUN_0090ef40 with the two stack arguments passed through, after storing the frame; the class vtables ActorMeshEmitterDefinitionManager::vftable (01804f6c, 01804f5… |
| `00914b60` | `ScalarDeletingDtor → ActorMeshEmitterDefinitionManagerScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 01804f6c (thiscall, one byte argument). Calls FUN_010b6110 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this. |
| `00914bc0` | `ThunkThisMinus4ToDeletingDtorB` | 🟢 | virtual | Adjustor thunk (bytes 83 e9 04 e9 98 ff ff ff): subtracts 4 from ECX and jumps to FUN_00914b60. It is vftable[0] of the second ActorMeshEmitterDefinitionManager vtable (01804f54). |
| `00914fb0` | `— ⚠reject` | ⚪ | virtual | Virtual at vftable[5] of 01804f6c (thiscall, one argument). Stores the argument into a local, then calls FUN_00914f30 with ECX=&local and stack arguments (this+0xc, &arg); returns nothing. |
