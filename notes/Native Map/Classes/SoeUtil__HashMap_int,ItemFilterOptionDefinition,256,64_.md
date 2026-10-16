# Class: SoeUtil::HashMap<int,ItemFilterOptionDefinition,256,64>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107ed90` | `HashMapDerivedDtorBody` | 🟢 | dtor | Destructor body of the 256,64 variant: stores the HashMap<int,ItemFilterOptionDefinition,256,64> vftable, clears the buckets through 0107de40, destroys the tree at +0x40c (children through 00b8ead0, root through FreeMem… |
| `0107ee40` | `HashMapSlot1PopThunk` | 🟢 | virtual | Virtual slot 1 this-adjusting thunk (bytes: add ecx,0x40c; jmp 00b94f80). Adjusts the receiver by +0x40c and jumps to 00b94f80 (label HashList_PopFreeNode in nmq, reopened:likely), which returns one free node of the emb… |
| `0107ee50` | `HashMapSlot2PushThunk` | 🟢 | virtual | Virtual slot 2 this-adjusting thunk (bytes: add ecx,0x40c; jmp 00b94fd0). Adjusts the receiver by +0x40c and jumps to 00b94fd0 (label HashList_PushFreeNode in nmq, reopened:likely), which returns one node to the embedde… |
| `0107ee80` | `HashMap256x64ScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 0107ed90 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
| `0107ef20` | `HashMap256x64Ctor` | 🟢 | ctor | Constructor body: stores the base HashMap<int,ItemFilterOptionDefinition,256,-1> vftable, zeroes this+0x8, clears the 256-bucket array at this+0xc (_memset 0x400 bytes), zeroes +0x4, stores the HashMap<int,ItemFilterOpt… |
