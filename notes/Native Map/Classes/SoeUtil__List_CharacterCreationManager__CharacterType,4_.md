# Class: SoeUtil::List<CharacterCreationManager::CharacterType,4>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00981900` | `List_CharacterType4_Ctor` | ✅ | ctor | Constructor of the pooled CharacterType list: installs the base vtable, zeroes head/tail/count, installs the List<...,4> vtable, zeroes the pool fields and builds the inline free list of four 0x78-byte nodes at +0x24 (c… |
| `00982360` | `List_PoolPopNodeThunk` | 🟢 | function | Jump thunk: adds 0x10 to this (the pool sub-object embedded in the list) and jumps to the pool node pop 00982280 (bytes: add ecx,0x10; jmp 00982280). |
| `00982370` | `List_PoolPushNodeThunk → List_CharacterType4_FreeNodeThunk` | 🟢 | function | Jump thunk: adds 0x10 to this and jumps to the pool node push 009822f0 (bytes: add ecx,0x10; jmp 009822f0). |
| `00982380` | `List_CharacterType4_Dtor` | ✅ | dtor | Destructor of the pooled CharacterType list: installs the List<...,4> vtable, drains list nodes with 009800a0, frees any heap-expanded pool block tree (root at +0x10, children freed with 0097e6c0 then FUN_00401c00(root,… |
| `009824d0` | `List_CharacterType4_ScalarDeletingDtor` | 🟢 | dtor | Deleting destructor wrapper: runs the pooled-list destructor 00982380 and frees the object with _free when the delete flag is set. |
