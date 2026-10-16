# Class: SoeUtil::HashList<EncounterDetailsCommon,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a29440` | `EncounterDetailsHashListInit` | ✅ | function | Zeros map bookkeeping and 1024-bucket table, then installs hash-list vtable. |
| `00a294c0` | `EncounterDetailsHashListScalarDeletingDtor` | ✅ | virtual | Clears hash-list entries and frees object when deletion flag bit zero is set. |
