# Class: MessageDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfa510` | `MessageDataSource::GetColumnName ?` | 🟡 | virtual | Returns Index, MessageType, or Function for indices 0 through 2, or an error string otherwise. |
| `00cfa770` | `MessageDataSource::GetRowCount ?` | 🟡 | virtual | Refreshes rows when the dirty byte is clear, then returns the row count at offset 0x22c. |
| `00cfa790` | `MessageDataSource::MessageData::Init ?` | 🟡 | init | Installs an embedded IString<char> vtable, points its string at the static empty value, and clears length and capacity. |
| `00cfa860` | `MessageDataSource::MessageData::ScalarDeletingDtor ?` | 🟡 | dtor | Destroys the embedded IString and frees the record when the delete flag's low bit is set. |
| `00cfac10` | `MessageDataSource::AddMessageRecord ?` | 🟡 | function | Gets or creates a row by index, initializes its string, then returns the resulting record or null. |
| `00cfac80` | `— ⚠reject` | ⚪ | function | Removes a message row through FUN_00cfab80, writes the saved row index into a neighboring record, destroys a supplied string, clears a flag, and invokes virtual slot 2. |
| `00cfad00` | `MessageDataSource::Cleanup ?` | 🟡 | dtor | Restores MessageDataSource vtables, destroys the embedded message array and GUI userdata table, then frees the array buffer. |
| `00cfade0` | `MessageDataSource::MessageDataSource ?` | 🟡 | ctor | Initializes the datasource base with BaseClient.Message, installs class vtables, stores its argument, clears state, and initializes the message array. |
| `00cfae80` | `MessageDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the message datasource cleanup and frees the object when the delete flag's low bit is set. |
| `00cfaea0` | `MessageDataSource::GetCell ?` | 🟡 | virtual | Ensures the datasource is populated, validates the row, and returns its index, fixed message type label, or stored function string. |
