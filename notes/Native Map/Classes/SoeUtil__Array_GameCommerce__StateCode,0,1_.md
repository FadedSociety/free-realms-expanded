# Class: SoeUtil::Array<GameCommerce::StateCode,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b25370` | `SoeUtil::ArrayStateCodeAllocStorage ⚠downgrade` | 🟢 | virtual | Growth/allocate slot (vtable index 1): applies the 1.25x/1.2x policy against this+0xc, writes the new count through param_3 and returns AllocateMemory(param_2 * 0x48, 4). |
| `00b25410` | `SoeUtil::ArrayStateCodeFreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the buffer pointer. |
| `00b25430` | `SoeUtil::ArrayStateCodeSwapStorage` | ✅ | virtual | Virtual storage-swap slot (vtable index 3): forwards to the other array's slot 3 when param_3 is nonzero, else XOR-swaps the buffer pointer and this+8/this+0xc with the other array's. |
