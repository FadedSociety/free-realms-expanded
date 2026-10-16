# Class: SoeUtil::List<SoeUtil::StrongType<unsigned_int,SoeUtilSignatureEffectDefinitionId,1>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9160` | `SoeUtil::List::Allocate` | ✅ | virtual | Virtual allocation hook of the SoeUtil::List instantiation: routes a fixed (0xc, 4) request to the active memory manager (context from FUN_006fb9c0, manager from FUN_006fb860, virtual slot +4), else to the global manage… |
| `00aa91a0` | `SoeUtil::List::Free` | ✅ | virtual | Virtual release hook of the SoeUtil::List instantiation: forwards (pointer, 4) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00ab2520` | `EffectDefinitionIdListDtorThunk` | ✅ | dtor | Stores the EffectDefinitionId list vtable and tail-jumps to 00aaf700 for list cleanup. |
| `00ab2790` | `EffectDefinitionIdListScalarDeletingDtor` | ✅ | dtor | Installs the list specialization vtable, drains its nodes, frees the object only when flag bit zero is set, and otherwise returns the original pointer. |
