# Class: SoeUtil::List<ProxiedCharacter::PendingCompositeEffect,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0096e7d0` | `PendingCompositeEffectListCtor` | ✅ | ctor | Installs the dynamic and fixed-capacity PendingCompositeEffect list vtables, zeros three list pointers, then calls 0095b6a0 with ECX set to this+0x10 to initialize the inline pool region. |
| `0096f200` | `PendingCompositeEffectPopAdjustorThunk` | ✅ | virtual | Adds 0x10 to ECX and tail-jumps to 0096f120; it is an adjustor thunk, not a standalone list-pop body. |
| `0096f210` | `PendingCompositeEffectPushAdjustorThunk` | ✅ | virtual | Adds 0x10 to ECX and tail-jumps to 0096f190; it is an adjustor thunk, not a standalone list-push body. |
| `0096f220` | `PendingCompositeEffectListDtor` | ✅ | dtor | Installs the fixed-list vtable, repeatedly destroys list nodes, frees both tree sentinels and the tracked node block, then installs the dynamic-list base vtable and cleans its nodes. |
| `0096f310` | `PendingCompositeEffectListScalarDeletingDtor` | ✅ | dtor | Calls the pending-effect list destructor and frees object storage when the deleting flag's low bit is set. |
