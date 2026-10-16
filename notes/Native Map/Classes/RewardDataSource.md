# Class: RewardDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c33020` | `RewardDataSourceGetFieldName` | ✅ | method | Maps field indices 0 through 5 to name, icon ID, experience, coin count, entry count, and icon tint; otherwise returns <error>. |
| `00c33290` | `— ⚠reject` | ⚪ | method | For row zero, returns reward name, description, icon, type and counts through virtual reward-object accessors; formats missing localization IDs with a fallback string. |
| `00c33700` | `RewardDataSourceDtor` | ✅ | dtor | Restores RewardDataSource vtables, releases the shared reward object when the global gate allows, then destroys the embedded GUI datasource base. |
| `00c337b0` | `RewardDataSourceHasItem` | ✅ | method | Returns whether the pointer at this+0x22c is nonzero. |
| `00c338f0` | `RewardDataSourceScalarDeletingDtor` | ✅ | dtor | Runs RewardDataSource teardown and frees the object when flag bit 0 is set. |
| `00c33b30` | `RewardDataSourceCtor` | ✅ | ctor | Registers a datasource, initializes RewardDataSource vtables and fields, constructs the table name with .Entries suffix, allocates a RewardEntryDataSource, stores it with reference management, and returns this. |
