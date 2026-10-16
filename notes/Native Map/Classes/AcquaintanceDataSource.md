# Class: AcquaintanceDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c90830` | `AcquaintanceDataSource::ColumnName` | ✅ | virtual | Maps indices zero through four to Guid, Name, Acquaintance Type, Elapsed Time, and Online; other indices return an error label. |
| `00c90890` | `AcquaintanceDataSource::OnUpdateAquaintancesList` | ✅ | virtual | Calls FUN_009797a0 with the exact callback name SocialHandler:OnUpdateAquaintancesList and two zero arguments. |
| `00c908e0` | `AcquaintanceDataSource::GetField` | ✅ | virtual | Returns the dword at offset 0x0c from the object referenced by this+0x228. |
| `00c90930` | `AcquaintanceDataSource::GetColumnValue` | ✅ | virtual | Walks the acquaintance linked rows to the requested index and selects one of five fields, formatting or copying its value to the output. |
| `00c90a10` | `AcquaintanceDataSource::Dtor` | ✅ | dtor | Restores AcquaintanceDataSource vtables and destroys the embedded GUI user-data table. |
| `00c90a80` | `AcquaintanceDataSource::Ctor` | ✅ | ctor | Initializes the BaseClient.Acquaintances binding and vtables, then stores two supplied source values. |
| `00c90b00` | `AcquaintanceDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls AcquaintanceDataSource cleanup, conditionally frees the object when the deleting flag is set, and otherwise returns it. |
