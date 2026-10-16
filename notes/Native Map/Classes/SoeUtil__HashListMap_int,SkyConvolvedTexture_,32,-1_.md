# Class: SoeUtil::HashListMap<int,SkyConvolvedTexture*,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00805ab0` | `— ⚠reject` | ⚪ | virtual | Calls 006fb9c0 then 006fb860 (TlsGetValue of the index it yields). If that thread memory-manager is non-null it calls its vtable slot +4 with no arguments; else if global DAT_01bdf280 is non-null it does the same; else … |
| `00805af0` | `FreeAlign4StdcallWrapper ?` | 🟡 | virtual | Reads its one stack argument at [esp+4], pushes alignment 4 and that argument, calls 00401c00 (FreeMemory(ptr, align)), pops 8 and returns with ret 4. ECX is not used. |
