# Class: SoeUtil::Array<ItemRecord,18,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad7040` | `InlineItemRecordArrayFreeHeapBuffer` | ✅ | virtual | Frees the supplied buffer only if it differs from this object's aligned inline buffer. |
| `00ad7780` | `InlineItemRecordArrayDtor` | ✅ | dtor | Destroys live inline-array entries, conditionally frees non-inline storage, then performs base dynamic-array cleanup. |
| `00ad7850` | `InlineItemRecordArrayReserve` | ✅ | virtual | Returns the embedded buffer with capacity 18 for requests at most 18; larger requests delegate to dynamic reserve. |
| `00ad7c80` | `InlineItemRecordArrayCtor` | ✅ | ctor | Installs dynamic ItemRecord array vtable, zeros four fields and then installs the 18-element inline specialization vtable. |
| `00ad7ce0` | `InlineItemRecordArrayScalarDeletingDtor` | ✅ | dtor | Calls inline array destruction then frees the outer allocation when the deleting flag is set. |
| `00ad7e10` | `— ⚠reject` | ⚪ | virtual | Appends entries from source and destination, destroys live elements and clears temporary buffers as part of storage transfer. |
