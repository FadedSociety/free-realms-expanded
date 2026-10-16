# Class: SoeUtil::Array<InspectProxy::ProfileEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad6de0` | `— ⚠reject` | ⚪ | virtual | Returns current buffer if capacity suffices, returns null for zero count, otherwise allocates count*8 bytes and stores the new capacity. |
| `00ad6e80` | `ProfileEntryArrayFreeBuffer` | ✅ | virtual | Frees the supplied array buffer using alignment four. |
| `00ad6ea0` | `ProfileEntryArrayMoveStorage` | ✅ | virtual | With move flag, dispatches the element operation; otherwise swaps buffer pointers and exchanges size and capacity fields with the source array. |
| `00ad7720` | `ProfileEntryArrayDtor` | ✅ | dtor | Installs the typed array vtable, clears element count, frees backing storage and zeros its pointer. |
| `00ad78b0` | `ProfileEntryArrayScalarDeletingDtor` | ✅ | dtor | Frees the backing buffer and frees the array object when the deleting flag is set. |
