# Class: ActorMeshEmitterDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c05b0` | `ActorMeshEmitterDefinition::ScalarDeletingDestructor` | 🟢 | dtor | Calls 0x010b5460 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00914d40` | `— ⚠reject` | ⚪ | ctor | thiscall (this=arg1, arg2 on stack; returns this). Calls the base constructor FUN_00914670 first, then writes the vftable references named in the function (AssetRequestInterface::AssetRequester 017e7d14, ActorMeshEmitte… |
| `00914e30` | `ScalarDeletingDtor → ActorMeshEmitterDefinitionScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 01804fa0 (thiscall, one byte argument). Calls FUN_010b5460 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this. |
| `00914e50` | `ThunkThisMinus90ToDeletingDtor` | 🟢 | virtual | Adjustor thunk (bytes 81 e9 90 00 00 00 e9 d5 ff ff ff): subtracts 0x90 from ECX and jumps to FUN_00914e30. It is vftable[0] of the secondary vtable 01804f8c. |
