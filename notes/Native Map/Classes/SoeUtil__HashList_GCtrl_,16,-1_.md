# Class: SoeUtil::HashList<GCtrl*,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00824a30` | `HashListAllocNode20` | 🟢 | virtual | Routes an allocation of 0x14 bytes with alignment 4 to the thread memory manager (006fb860 result, vtable slot 1), else the global manager DAT_01bdf280 (slot 1), else 006fb950(0x14, 4); returns the callee's EAX. Virtual… |
| `00824a70` | `HashListFreeNode` | 🟢 | virtual | Takes one stack argument (pointer) and calls 00401c00(pointer, 4): free with 4-byte alignment through the memory router; the this register is unused; returns with ret 4. |
