# Class: ActorProjectileDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c0610` | `ActorProjectileDefinition::ScalarDeletingDestructor` | 🟢 | dtor | Calls 0x010b6220 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00915230` | `Ctor → ActorProjectileDefinition::Ctor` | ✅ | ctor | thiscall (this=arg1, arg2 on stack; returns this). Calls the base constructor FUN_00914670 first; stores AssetRequestInterface::AssetRequester vftable (017e7d14) at [this+0x90] and arg2 at [this+0x94]; stores ActorProje… |
| `009152e0` | `ScalarDeletingDtor → ActorProjectileDefinitionScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 0180501c (thiscall, one byte argument). Calls FUN_010b6220 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this in EAX. |
| `00915300` | `ThunkThisMinus90ToDeletingDtor` | 🟢 | virtual | Adjustor thunk (bytes 81 e9 90 00 00 00 e9 d5 ff ff ff): subtracts 0x90 from ECX and jumps to FUN_009152e0. It is vftable[0] of the secondary vtable 01805008. |
