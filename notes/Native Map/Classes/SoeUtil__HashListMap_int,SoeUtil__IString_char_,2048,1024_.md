# Class: SoeUtil::HashListMap<int,SoeUtil::IString<char>,2048,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00917e70` | `SoeUtil::HashListMapIString2048DtorBody` | ✅ | dtor | Destructor body: installs the HashListMap<int,IString<char>,2048,1024> vtable and drains the node chain at +8 through FUN_008e4650. Reads the embedded tree header at +0x2014 (param_1[0x805]), clears fields at +0x201c an… |
| `00918690` | `— ⚠reject` | ⚪ | method | Thiscall: computes n = min(argument, clamp(FUN_00918110 result, 2, 0x1000)). Allocates n*0x20 + 0x1c bytes through the allocator (FUN_006fb950, or the object at DAT_01bdf280 vtable slot 1 when set), writes n to the chun… |
| `009187b0` | `HashListMapReleaseChunk` | 🟢 | method | Thiscall with a chunk header (param_2). Reads the chunk's node count at +0x14 and first node at +0. Walks the free list at this+8 and unlinks every node whose address lies inside the chunk range. Subtracts the chunk cou… |
| `00918810` | `— ⚠reject` | ⚪ | virtual | Pops the head of the free list at this+8: if the list is empty it first calls FUN_00918690 (grow chunk). Takes the head node, sets this+8 to its first dword (next), decrements this+0xc, and when the byte at this+0x10 is… |
| `00918860` | `HashListMapPushFreeNode → SoeUtil::HashListMap<int,SoeUtil::IString<char>,2048,…` | 🟢 | virtual | Pushes the node argument onto the free list: stores the old head into its first dword, increments this+0xc, and sets this+8 to the node. When the byte at this+0x10 is set it finds the owning chunk with FUN_00918170, inc… |
| `009188c0` | `HashListMapIString2048Vftable1ThisAdjustingThunkTo00918810 → SoeUtil::HashListM…` | ✅ | virtual | This-adjusting thunk: adds 0x2014 to ECX, then jumps to 00918810. Its vtable slot is [1]. |
| `009188d0` | `ThunkAdjustThis2014_HashListMapIString2048_s2 ? → SoeUtil::HashListMap<int,SoeU…` | 🟡 | virtual | Decoded from bytes: add ecx,0x2014 then jmp rel32 to 00918860. A this-adjusting thunk with no body of its own. |
| `00919420` | `SoeUtil::HashListMapIString2048DeletingDtor` | ✅ | dtor | Scalar deleting destructor, thiscall with flag byte: calls the destructor body 00917e70, then when flag bit 0 is set frees the object with _free. Returns the object pointer. |
| `00919460` | `SoeUtil::HashListMapIString2048Ctor` | ✅ | ctor | Thiscall constructor: installs the HashListMap<int,IString<char>,2048,-1> vtable; zeroes dwords at +0x10, +8 and +0xc; clears 0x2000 bytes starting at +0x14; zeroes +4; and installs the HashListMap<int,IString<char>,204… |
