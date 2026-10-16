# Class: SoeUtil::Array<SoeUtil::StrongType<unsigned_int,SoeUtilSignatureActorId,1>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53550` | `— ⚠reject` | ⚪ | virtual | Virtual allocate hook (slot 1) of Array<SoeUtil::StrongType<unsigned_int,SoeUtilSignatureActorId,1>,0,1>: same clumping policy as the other Array instantiations - request*5/4 when growing past the capacity at +0xc, exis… |
| `00a535f0` | `SoeUtil::Array_SignatureActorId_Deallocate` | ✅ | virtual | Virtual deallocate hook (slot 2) of Array<SoeUtil::StrongType<unsigned_int,SoeUtilSignatureActorId,1>,0,1>: loads the pointer argument and forwards it to FUN_00401c00 with alignment 4 (bytes 8b 44 24 04 6a 04 50 e8 ... … |
| `00a53610` | `SoeUtil::Array_SignatureActorId_Swap ⚠downgrade` | 🟢 | virtual | Virtual swap hook (slot 3) of Array<SoeUtil::StrongType<unsigned_int,SoeUtilSignatureActorId,1>,0,1>: exchanges the buffer pointer at +4 and, when unequal, XOR-exchanges the count at +8 and capacity at +0xc with the arg… |
| `00a559e0` | `SoeUtil::Array_SignatureActorId_Dtor` | ✅ | dtor | Installs the typed actor-id array vtable, clears element count, frees backing storage with alignment four, and nulls the pointer. |
| `00a56470` | `SoeUtil::Array_SignatureActorId_ScalarDeletingDtor` | ✅ | dtor | Resets typed actor-id array vtable, clears count, frees its aligned buffer, then frees the object only when delete flag bit 0 is set. |
