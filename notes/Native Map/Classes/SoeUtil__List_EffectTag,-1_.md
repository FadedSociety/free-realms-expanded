# Class: SoeUtil::List<EffectTag,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc290` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x70 bytes aligned to eight through the thread manager, global manager, or aligned CRT fallback. |
| `008bc2d0` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
