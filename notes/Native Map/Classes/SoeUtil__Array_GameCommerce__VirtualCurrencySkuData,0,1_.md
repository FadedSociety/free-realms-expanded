# Class: SoeUtil::Array<GameCommerce::VirtualCurrencySkuData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24ef0` | `SoeUtil::ArrayVirtualCurrencySkuDataAllocStorage` | ✅ | virtual | Growth/allocate slot (vtable index 1): applies the 1.25x/1.2x capacity policy against this+0xc, writes the new count through param_3 and returns AllocateMemory(param_2 * 0x60, 4); reuses the existing buffer when the cou… |
| `00b24f90` | `SoeUtil::ArrayVirtualCurrencySkuDataFreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the buffer pointer. |
| `00b24fb0` | `— ⚠reject` | ⚪ | virtual | Virtual storage-swap slot (vtable index 3): forwards to the other array's slot 3 when param_3 is nonzero, else XOR-swaps the buffer pointer and the dwords at this+8/this+0xc with the other array's. |
