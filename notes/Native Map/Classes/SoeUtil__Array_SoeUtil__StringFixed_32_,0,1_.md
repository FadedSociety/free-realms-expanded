# Class: SoeUtil::Array<SoeUtil::StringFixed<32>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63e30` | `ArrayString32::Dtor ?` | 🟡 | dtor | Installs the zero-capacity string array vtable, releases its string storage, frees its pointer buffer, and clears the buffer field. |
| `00d640e0` | `ArrayString32::ScalarDeletingDtor ?` | 🟡 | dtor | Releases string elements and array storage, clears the pointer, and conditionally frees the containing object. |
