# Class: SoeUtil::Hash<int,2048,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c313d0` | `IntHashMapCtor ? ⚠rename` | 🟡 | ctor | Installs the HashMap base vtable, zeros the count, clears 0x2000 bucket bytes, then switches to the Hash specialization vtable. |
| `00c31450` | `IntHashScalarDeletingDtor` | ✅ | dtor | Temporarily restores the HashMap base vtable, clears entries, then frees the object if flag bit 0 is set. |
