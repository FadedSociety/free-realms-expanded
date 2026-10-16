# Class: SoeUtil::List<RecipeComponentData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbd80` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x2c bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bbdc0` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
