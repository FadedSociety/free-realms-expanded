# Class: SoeUtil::Set<int,500,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d068a0` | `— ⚠reject` | ⚪ | ctor | Zeroes this+0x4 and this+0x8, installs the SoeUtil::Set<int,500,0> vtable at offset 0, and calls 00d05b60 (the 100-node pool builder with capacity 500) on this. |
| `00d07020` | `TreePoolGrow20` | 🟢 | method | Chooses an entry count max(arg, clamp(00d06a90(), 3, 0x1000)), allocates count*0x14+0x1c bytes through the memory-manager vtable (006fb860 / DAT_01bdf280) or the 006fb950 fallback with alignment 4, sets the chunk capaci… |
| `00d070f0` | `TreePoolChunkRelease20` | 🟢 | method | For chunk param_2 (capacity at +0x14), removes the free-list entries whose addresses lie inside the chunk from this+0x8, subtracts the capacity from this+0xc, erases the chunk header with 00d06e70, and frees the chunk w… |
| `00d07150` | `TreePoolAlloc20` | 🟢 | method | Refills the free list through 00d07020(0) when it is empty, pops the head entry (head = its link word), decrements the free count this+0xc, and when the flag byte at this+0x10 is set finds the owning chunk with 00d06af0… |
| `00d071c0` | `TreePoolNodeFree20` | 🟢 | method | Pushes entry param_2 onto the free list at this+0x8 (link = old head), increments the free count this+0xc; when the flag byte at +0x10 is set, finds the owning chunk with 00d06af0, increments its free count (+0x18) and … |
| `00d07230` | `SetInt500DtorBody` | ✅ | dtor | Non-deleting destructor body: installs the Set<int,500,0> vtable, releases the tree nodes at +0x4 through 007a2470 and vtable slot +0xc, frees the pool chunk tree at +0xc through 00d06240 and 00401c00(chunk, 4), then in… |
| `00d07330` | `ThunkAdd0c_00d07150` | ✅ | virtual | Thunk: adds 0xc to ECX (bytes 83 c1 0c) and tail-jumps to 00d07150 (e9 jump). |
| `00d07340` | `ThunkAdd0c_00d071c0` | ✅ | virtual | Thunk: adds 0xc to ECX (bytes 83 c1 0c) and tail-jumps to 00d071c0 (e9 jump). |
| `00d07350` | `SetInt500ScalarDtor` | ✅ | dtor | Scalar deleting destructor: saves this, calls the Set<int,500,0> body 00d07230, frees the object with _free when bit 0 of the flag argument is set (test byte [esp+8],1), returns this with ret 4. |
