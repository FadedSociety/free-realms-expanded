# Class: SoeUtil::HashListMap<int,SoeUtil::IString<char>,2048,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc400` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x20 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bc440` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `009105e0` | `DtorBodyClearHead → SoeUtil::HashListMap<int,SoeUtil::IString<char>,2048,-1>::D…` | ✅ | dtor | Stores HashListMap vftable 0x01801e34 at [ecx]; then, while [this+8] is nonzero, calls thiscall FUN_008e4650([this+8]) (callee pops its argument). Loop test precedes each call. |
| `00911330` | `ScalarDeletingDtor → SoeUtil::HashListMapIStringChar2048ScalarDtor` | ✅ | dtor | Virtual at vftable[0] of 01801e34 (thiscall, one byte argument). Stores the HashListMap<int,IString<char>,2048,-1> vftable at [this]; runs the clear loop (while [this+8] != 0 call FUN_008e4650([this+8])); then if (arg &… |
