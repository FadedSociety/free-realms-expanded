# Class: SoeUtil::Array<ItemRecord,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad6b80` | `ItemRecordArrayReserve` | 🟢 | function | Requests a larger buffer, copies existing 16-byte ItemRecord elements with vtable and data fields, destroys old storage and records the new pointer and capacity. |
| `00ad6f10` | `— ⚠reject` | ⚪ | virtual | Returns existing storage when capacity suffices, returns null for zero count, otherwise allocates count*16 bytes and updates capacity. |
| `00ad6fb0` | `ItemRecordArrayFreeBuffer` | ✅ | virtual | Frees the supplied ItemRecord buffer using alignment four. |
| `00ad6fd0` | `ItemRecordArrayMoveStorage` | ✅ | virtual | With move flag, invokes source array operation; otherwise swaps buffer pointers and exchanges size and capacity fields. |
| `00ad72e0` | `— ⚠reject` | ⚪ | function | For an in-range index, destroys the prior ItemRecord and returns its slot; otherwise grows storage, extends size and initializes intervening records. |
| `00ad7750` | `ItemRecordArrayDtor` | ✅ | dtor | Installs array vtable, destroys active elements, frees backing storage and clears the pointer. |
| `00ad78f0` | `ItemRecordArrayScalarDeletingDtor` | ✅ | dtor | Destroys live entries, frees backing buffer and frees the array object when deleting flag is set. |
| `00ad7a00` | `— ⚠reject` | ⚪ | function | Grows storage as needed, changes count and initializes new entries with ItemRecord vtable and three zero fields. |
| `00ad7aa0` | `ItemRecordArrayAppendRange` | 🟢 | function | Extends destination count by the requested number, grows storage if needed and copies each source record's three data fields into destination entries. |
| `00ad7b10` | `ItemRecordArrayGetOrCreate` | 🟢 | function | Gets or extends the requested array index, then ensures the returned slot has ItemRecord vtable and zero fields. |
| `00ad7f10` | `ItemRecordArrayGetOrCreate` | 🟢 | function | Extends the array through requested index when needed and returns buffer + index*16. |
