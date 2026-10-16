# Class: SoeUtil::List<CharacterStat,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbf80` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x14 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bbfc0` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `008e0640` | `ListCharacterStat_Dtor` | ✅ | dtor | Writes the SoeUtil::List<CharacterStat,-1> vtable to this, then tail-jumps to 008d45b0 for list cleanup. |
| `008e1fd0` | `ListCharacterStat_ScalarDeletingDtor` | ✅ | dtor | Installs the CharacterStat list vtable, runs list cleanup, and frees the object when flag bit 0 is set. |
