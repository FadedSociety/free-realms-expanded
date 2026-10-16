# Class: NameValueDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1ad0` | `NameValueDataSource::ScalarDeletingDestructor` | 🟢 | dtor | Deleting destructor wrapper: calls FUN_0098dcf0 (which stores NameValueDataSource::vftable) and then calls _free(this) when param_2 bit 0 is set. |
| `0098dcf0` | `NameValueDataSource::Destructor` | ✅ | dtor | Stores the NameValueDataSource vtable at object offset zero and returns. |
