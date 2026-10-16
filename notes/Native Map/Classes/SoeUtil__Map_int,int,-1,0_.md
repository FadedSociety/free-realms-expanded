# Class: SoeUtil::Map<int,int,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00824950` | `MapAllocNode24` | 🟢 | virtual | Routes an allocation of 0x18 bytes with alignment 4 to the thread memory manager (006fb860 result, vtable slot 1), else the global manager DAT_01bdf280 (slot 1), else 006fb950(0x18, 4). Same body as 00822180 for the Map… |
| `00824990` | `MapFreeNode` | 🟢 | virtual | Takes one stack argument (pointer) and calls 00401c00(pointer, 4): free with 4-byte alignment through the memory router; the this register is unused; returns with ret 4. |
| `00825130` | `MapDtorBody` | 🟢 | dtor | Sets the Map<int,int,-1,0> vtable at this+0, calls 008234e0 on the children (+0xc, +0x10) of the root at this+4 and vtable slot 3 (00824990) on the root when non-NULL, then zeroes this+8 and this+4. Returns nothing. |
| `00825190` | `MapScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor shape: sets the Map<int,int,-1,0> vtable at this, runs the tree teardown (008234e0 walks, vtable slot 3 on the root), zeroes this+4 and this+8, then calls _free (thunk 00d82463) on this only w… |
