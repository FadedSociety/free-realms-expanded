# Class: SoeUtil::HashListMap<int,int,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e13d0` | `IntHashMapDtor` | ✅ | dtor | Sets map vtable and repeatedly destroys head nodes until empty. |
| `009e1cf0` | `IntHashMapCtor` | ✅ | ctor | Installs map vtable, clears links/count and 16 buckets, installs base HashList vtable. |
