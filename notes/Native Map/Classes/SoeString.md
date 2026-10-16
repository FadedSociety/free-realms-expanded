# Class: SoeString

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00e704b0` | `SoeString::Free` | ✅ | method | thiscall (ECX = owner object), data pointer in stack arg 1: a NULL pointer returns success (DAT_0185f358). Otherwise it calls the owner's virtual slot [1] (vtable+4) to get a pool object: when that is NULL it frees the … |
