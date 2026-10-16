# Class: SoeUtil::StringFixed<50>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae2890` | `StringFixed50Init` | ✅ | ctor | Calls the IStringFixed<char,50> base constructor and then installs StringFixed<50> vtable. |
| `00ae2910` | `StringFixed50::ScalarDeletingDtor` | ✅ | dtor | Calls base fixed-string destructor 008c7c90, optionally frees this when flag bit 0 is set, and returns this otherwise. |
