# Class: SoeUtil::Hash<int,128,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d02050` | `HashPoolGrow` | 🟢 | method | Chooses an entry count max(arg, clamp(00d01ac0(), 5, 0x1000)), allocates count*12+0x1c bytes through the memory-manager vtable (006fb860 / DAT_01bdf280) or the 006fb950 fallback, sets chunk capacity and free count (+0x1… |
| `00d02170` | `HashPoolChunkRelease` | 🟢 | method | For a chunk (param_2, capacity at +0x14), removes every free-list entry whose address lies inside the chunk from this+0x8, subtracts the chunk capacity from this+0xc, erases the chunk header from the tree with 00d01ea0,… |
| `00d021d0` | `HashPoolNodeAlloc` | 🟢 | method | Pops the head entry of the pool free list at this+0x8, refilling it with 00d02050(0) when empty; stores the popped entry's link word as the new head, decrements free count this+0xc, and when the flag byte at this+0x10 i… |
| `00d02240` | `HashPoolNodeFree` | 🟢 | method | Pushes entry param_2 onto the pool free list at this+0x8 (its link word = old head), increments free count this+0xc; when the flag byte at +0x10 is set, finds the owning chunk with 00d01b20, increments the chunk free co… |
| `00d022a0` | `ThunkAdd20c_00d021d0` | ✅ | virtual | Thunk: adds 0x20c to ECX (bytes 81 c1 0c 02 00 00) and tail-jumps to 00d021d0 (e9 jump); returns the callee's value. |
| `00d022b0` | `ThunkAdd20c_00d02240` | ✅ | virtual | Thunk: adds 0x20c to ECX (bytes 81 c1 0c 02 00 00) and tail-jumps to 00d02240 (e9 jump); returns the callee's value. |
| `00d02510` | `HashInt128Ctor` | ✅ | ctor | Calls the HashMap<int,int,128,-1> base initialiser 00d022e0 on this, then stores the SoeUtil::Hash<int,128,128> vtable at offset 0 and returns this. |
| `00d02570` | `HashInt128ScalarDtor` | ✅ | dtor | Scalar deleting destructor: sets up the cookie-guarded frame, calls the base body 00d01850, frees the object with _free when bit 0 of the flag argument is set, returns this. |
