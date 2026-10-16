# Class: SoeUtil::Array<SoeUtil::StrongType<unsigned_int,SoeUtilSignatureEventId,1>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a55950` | `SoeUtil::Array_SignatureEventId_Dtor` | ✅ | dtor | Installs the dynamic event-id array vtable, clears its count, frees its backing storage with alignment four, and nulls the pointer. |
| `00a563b0` | `SoeUtil::Array_SignatureEventId_ScalarDeletingDtor` | ✅ | dtor | Resets the array vtable, clears count, frees its buffer with alignment four, clears the pointer, and conditionally frees the object on flag bit 0. |
