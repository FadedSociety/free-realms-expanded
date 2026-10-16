# Class: SoeUtil::List<GCtrl*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00824aa0` | `ListAllocNode12` | 🟢 | virtual | Routes an allocation of 0xC bytes with alignment 4 to the thread memory manager (006fb860 result, vtable slot 1), else the global manager DAT_01bdf280 (slot 1), else 006fb950(0xc, 4); returns the callee's EAX. Virtual a… |
| `00824ae0` | `ListFreeNode` | 🟢 | virtual | Takes one stack argument (pointer) and calls 00401c00(pointer, 4): free with 4-byte alignment through the memory router; the this register is unused; returns with ret 4. |
