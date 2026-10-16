# Class: SoeUtil::Map<int,GuiDataSourceObserver*,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00822180` | `MapAllocNode24` | 🟢 | virtual | Routes an allocation of 0x18 bytes with alignment 4 (pushed as 4, then 0x18) to the thread memory manager (vtable slot 1), else the global manager DAT_01bdf280 (vtable slot 1), else 006fb950; returns the callee's EAX. V… |
| `008221c0` | `MapFreeNode` | 🟢 | virtual | Takes one stack argument (pointer) and calls 00401c00(pointer, 4): free with 4-byte alignment through the memory router. The this register is unused; returns with ret 4. |
| `00822580` | `MapDtorBody` | 🟢 | dtor | Sets the Map vtable at this+0, calls 00821ef0 on the children (+0xc, +0x10) of the root at this+4 and vtable slot 3 (008221c0) on that root when non-NULL, then zeroes this+8 and this+4. Reached from EH unwind funclets 0… |
| `008225c0` | `MapScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor shape: stores the Map vtable at this+0, runs the same tree teardown as 00822580 (00821ef0 walks, vtable slot 3 on the root), zeroes this+4 and this+8, then calls _free(this) (thunk 00d82463) o… |
