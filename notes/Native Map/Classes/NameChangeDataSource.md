# Class: NameChangeDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098dbd0` | `NameChangeDataSource::GetColumnName` | ✅ | virtual | Maps indices 0 through 6 to Id, Type, IsCheckPending, IsRenamePending, Name, Status, and StatusString; returns <error> for other indices. |
| `0098dc40` | `NameChangeDataSource::Method2` | ✅ | virtual | If byte +0x22c is clear, calls the virtual method at object this-0x30 slot +0x38; then tail-jumps to 010c8f30 with ECX adjusted to this+4. |
| `0098dc70` | `NameChangeDataSource::RebuildFilteredRows` | ✅ | virtual | Resets the row-list helper, walks the records, appends each record whose +8 key matches filter +0x260 or when that filter is -1, then sets byte +0x25c. |
| `0098e240` | `NameChangeDataSource::GetFieldValue` | ✅ | virtual | Selects a row and maps requested fields to its id, type, pending flags, name, status, or status string; writes fallback text for missing rows, missing data, and invalid columns, and returns 1. |
| `0098e790` | `NameChangeDataSource::Destructor` | ✅ | dtor | Installs the NameChangeDataSource vtables at the three object subobjects and destroys the GuiUserDataTable member at +0x34. |
| `0098e810` | `NameChangeDataSource::Constructor` | ✅ | ctor | Registers BaseClient.NameChange, initializes its base state, installs three NameChangeDataSource vtables, stores two supplied fields, and sets its state bytes and index defaults. |
| `0098e8a0` | `NameChangeDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the NameChangeDataSource destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
