# Class: AchievementObjectiveDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce3980` | `AchievementObjectiveDataSource::ColumnNameByIndex` | ✅ | virtual | Virtual at AchievementObjectiveDataSource::vftable +0x30 slot [1] (one stack argument): switch on the index returning Name, Description, Status, Completed, NotUsed, Id for 0..5, and the text <error> otherwise. |
| `00ce3c40` | `AchievementObjectiveDataSource::LinkedIdOrZero ?` | 🟡 | virtual | Fastcall with no stack arguments: returns the dword at +0xf0 of the object pointed to by this+0x22c, or 0 when that pointer is null. |
| `00ce4770` | `— ⚠reject` | ⚪ | virtual | Virtual at AchievementObjectiveDataSource::vftable +0x30 slot [4] (two stack arguments): returns 0 unless the dword at this+0x22c is nonzero, the vtbl+0x10 call on the object at this+4 with param_2 returns a nonnull poi… |
| `00ce5200` | `AchievementObjectiveDataSource::DtorBody ?` | 🟡 | dtor | Restores objective data source vtables at the object and two embedded subobjects, then destroys the embedded GUI data table holder. |
| `00ce57e0` | `AchievementObjectiveDataSource::Ctor ?` | 🟡 | ctor | Calls the base constructor, installs three objective source vtables, and stores two constructor arguments at indices 0x95 and 0x96. |
| `00ce5870` | `AchievementObjectiveDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the objective source destructor body, conditionally frees the object when flag bit 0 is set, then returns its pointer. |
