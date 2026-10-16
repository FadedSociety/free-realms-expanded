# Class: SoeUtil::List<EncounterDetailsCommon,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9aa00` | `SoeUtil_ListEncounterDetailsCommon_NodeAlloc` | 🟢 | method | Allocates a list node: calls 006fb9c0, asks 006fb860 for the current memory manager and, when one is available, tail-calls its vtable slot [1] (the allocate slot, same shape as AllocateMemory::00a9aa34); otherwise falls… |
| `00a9aa40` | `SoeUtil_ListEncounterDetailsCommon_NodeFree` | ✅ | method | Forwards the node pointer to 00401c00 (FreeMemory) with alignment 8 and returns; the alignment matches the 8-byte-aligned 0x2d8-byte node allocated by 00a9aa00. |
| `00aa3400` | `ListDtor_EncounterDetailsCommon` | ✅ | dtor | Stores SoeUtil::List<EncounterDetailsCommon,-1>::vftable, repeatedly unlinks the current head through FUN_00aa3220 until the head is null, then returns. |
| `00aa34a0` | `ListScalarDeletingDtor_EncounterDetailsCommon` | ✅ | dtor | Stores the typed list vftable, repeatedly unlinks its head through FUN_00aa3220, then frees the list object only when flag bit 0 is set. |
