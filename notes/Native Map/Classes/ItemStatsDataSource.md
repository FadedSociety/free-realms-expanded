# Class: ItemStatsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c31180` | `ItemStatsDataSourceGetCount` | ✅ | method | Returns the dword at this+0x224c. |
| `00c314c0` | `— ⚠reject` | ⚪ | method | Validates row index against count, maps it to a client stat key, locates the matching stat record, rejects missing/hidden rows, fetches its localized name through FUN_00baeae0, and formats the result to param_4. |
| `00c31690` | `— ⚠reject` | ⚪ | method | Initializes the output to <Error>, obtains the client item-stat definitions, looks up param_2, writes <No Stat Definition> if absent, and returns the stored value at +0x22c. |
| `00c31820` | `ItemStatsDataSourceDtor` | ✅ | dtor | Restores ItemStatsDataSource vtables, destroys its integer array and hash map, releases a shared IString buffer with reference counting, then tears down the GUI datasource base. |
| `00c31930` | `— ⚠reject` | ⚪ | ctor | Registers BaseClient.ItemStats, initializes four datasource vtables, stores its configuration pointer, initializes an IString field, constructs a 2048-bucket hash map and zero-inline integer array. |
| `00c31a00` | `ItemStatsDataSourceScalarDeletingDtor` | ✅ | dtor | Calls ItemStatsDataSource teardown, then frees the object when flag bit 0 is set. |
