# Class: AbilityLineDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8ee10` | `AbilityLineDataSource::ColumnName` | ✅ | virtual | Maps indices 0 through 10 to ability line ID, activation, name, description, icon, profile, experience, rank and level labels. |
| `00c8eeb0` | `AbilityLineDataSource::OnRowsChanged` | ✅ | virtual | If the source is not already dirty, calls a virtual callback through its adjusted owner pointer, then calls FUN_010c8f30. |
| `00c8eed0` | `AbilityLineDataSource::VirtualDispatchAdjustor` | ✅ | virtual | Loads the vtable function at offset 0x10, clears byte this+0x258, and jumps through the loaded function pointer. |
| `00c8eee0` | `AbilityLineDataSource::ClearDirtyAndJump` | ✅ | virtual | Clears byte this+0x258 then jumps to 0x010caf20. |
| `00c8eef0` | `AbilityLineDataSource::RefreshRows` | ✅ | virtual | Refreshes the UI list and traverses profile ability lines and their entries, emitting each available entry identifier. |
| `00c8efa0` | `AbilityLineDataSource::GetColumnValue` | ✅ | virtual | Searches ability rows by ID and returns row ID, activation state, localized name and description, icon, profile, experience, rank and required level fields. |
| `00c8f1a0` | `AbilityLineDataSource::Dtor` | ✅ | dtor | Restores AbilityLineDataSource vtables and destroys embedded GUI data source state. |
| `00c8f210` | `AbilityLineDataSource::AbilityLineDataSource` | ✅ | ctor | Registers BaseClient.AbilityLines, installs source vtables, stores its dependency and clears the initial dirty flag. |
| `00c8f290` | `AbilityLineDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the source destructor and frees the object when deleting flag bit 0 is set. |
