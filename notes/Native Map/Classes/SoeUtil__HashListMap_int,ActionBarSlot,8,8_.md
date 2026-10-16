# Class: SoeUtil::HashListMap<int,ActionBarSlot,8,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c44770` | `HashListMap8x8_Destruct` | ✅ | dtor | Destructor of the pooled action-bar map: reinstalls the HashListMap<int,ActionBarSlot,8,8> vftable and frees the node list at +8 through FUN_008cae00, then tears down the inline chunk pool (block list at +0x34 rooted at… |
| `00c451f0` | `HashListMap8x8_AllocThunk` | 🟢 | virtual | 8-byte jump thunk at the pooled map's virtual allocator slot (bytes 83 C1 34 E9 18 FF FF FF): adds 0x34 to this and jumps to the chunk-pool allocate 00c45110, so the pool object lives at map+0x34. |
| `00c45200` | `HashListMap8x8_FreeThunk` | 🟢 | virtual | 8-byte jump thunk at the pooled map's virtual free slot: adds 0x34 to this and jumps to the chunk-pool free 00c45180. |
| `00c45210` | `HashListMap8x8_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the pooled map destructor 00c44770 and frees the object with _free when the low bit of the flag argument is set. |
| `00c45230` | `HashListMap8x8_Construct` | ✅ | ctor | Constructor of the pooled action-bar map: zeroes +4..+0xc, installs the HashListMap<int,ActionBarSlot,8,-1> vftable and then overwrites it with the HashListMap<int,ActionBarSlot,8,8> vftable (the first store is dead), a… |
| `00c452c0` | `HashListMap8x8_DestructWrapper` | 🟢 | dtor | SEH-framed destructor wrapper that calls the pooled map destructor 00c44770 with the object in ECX (no other work); invoked from EH unwind funclets. |
