# Class: SoeUtil::HashList<ClientProfileData_const*,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2fdc0` | `ClientProfileHashListCtor` | ✅ | ctor | Zeros list/map counters and pointers, then installs the HashList specialization vtable after its HashListMap base vtable. |
| `00c2fe40` | `ClientProfileHashListScalarDeletingDtor` | ✅ | dtor | Calls map cleanup and frees this object when flag bit 0 is set; otherwise returns this. |
