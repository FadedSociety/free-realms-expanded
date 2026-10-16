# Class: ItemFilterDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce6eb0` | `ItemFilterDataSource::ColumnName ?` | 🟡 | virtual | Maps indices 0..3 to Id, Name, Option Count, and Options; returns <error> for other indices. |
| `00ce6fa0` | `ItemFilterDataSource::GetRowCount ?` | 🟡 | virtual | Reads item-filter manager pointer at +0x224 and returns its count field at manager offset +0xb8. |
| `00ce7030` | `ItemFilterDataSource::FormatOptions ?` | 🟡 | function | Walks option list and appends formatted Name, HasValueRange, Value, ValueRangeStart, and ValueRangeEnd records to output wide string. |
| `00ce7230` | `ItemFilterDataSource::GetColumn ?` | 🟡 | virtual | Gets a row from the manager, looks up its filter key, and returns id/name/options for supported columns; otherwise returns false. |
| `00ce7300` | `ItemFilterDataSource::DtorBody ?` | 🟡 | dtor | Restores source vtables at main object and two embedded bases, then destroys the embedded base holder. |
| `00ce7380` | `ItemFilterDataSource::Ctor ?` | 🟡 | ctor | Constructs BaseClient.ItemFilters holder, stores manager argument, installs three source vtables, and invokes embedded initialization method. |
| `00ce7400` | `ItemFilterDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls source destructor body, conditionally frees object when flag bit 0 is set, and returns its pointer. |
