# Class: SoeUtil::Array<GameCommerce::CountryCode,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b25220` | `SoeUtil::ArrayCountryCodeAllocStorage` | ✅ | virtual | Growth/allocate slot (vtable index 1): applies the 1.25x/1.2x policy against this+0xc, writes the new count through param_3 and returns AllocateMemory(param_2 << 7, 4) - 128-byte elements. |
| `00b252c0` | `SoeUtil::ArrayCountryCodeFreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the buffer pointer. |
| `00b252e0` | `— ⚠reject` | ⚪ | virtual | Virtual storage-swap slot (vtable index 3): forwards to the other array's slot 3 when param_3 is nonzero, else XOR-swaps the buffer pointer and this+8/this+0xc with the other array's. |
