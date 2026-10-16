# Class: RewardEntryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c33080` | `RewardEntryDataSourceGetFieldName` | ✅ | method | Maps indices 0-12 to type, name, description, icon, quantity, tint, item ID, parameters, text color, members-only, GUID, hidden and separated columns; other indices return <error>. |
| `00c331e0` | `RewardEntryDataSourceGetField40` | ✅ | method | Reads the nested reward item at this+0x22c then +0x25c; returns its +0x40 field or zero if absent. |
| `00c33430` | `RewardEntryDataSourceGetFieldValue` | ✅ | method | Resolves the requested reward row, then returns entry type, localized name, description placeholder, icon, quantity, tint, item ID, parameters, text color, member/hidden/separated flags by field index. |
| `00c337c0` | `RewardEntryDataSourceDtor` | ✅ | dtor | Restores RewardEntryDataSource vtables in its main and adjusted subobjects, then destroys the embedded GUI datasource base. |
| `00c33a80` | `RewardEntryDataSourceCtor` | ✅ | ctor | Registers a base datasource, installs RewardEntryDataSource vtables in three subobjects, and stores three supplied pointers in fields +0x254/+0x258/+0x25c. |
| `00c33b10` | `RewardEntryDataSourceScalarDeletingDtor` | ✅ | dtor | Calls RewardEntryDataSource teardown, then frees the object when flag bit 0 is set. |
