# Class: SoeUtil::Array<AppStoreBillboardPanelDefinition,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c36230` | `— ⚠reject` | ⚪ | virtual | Returns existing storage when capacity suffices or the zero-capacity special case applies; otherwise calculates a growth capacity and allocates capacity times 0x5c bytes. |
| `00c362d0` | `BillboardPanelArrayFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c362f0` | `ArraySwapStorage` | ✅ | virtual | When the flag is set, invokes a virtual operation on the supplied array; otherwise swaps the backing pointer, length and capacity fields. |
| `00c37250` | `BillboardPanelArrayDtor` | ✅ | dtor | Installs the array vtable, destroys all live elements, frees its backing allocation with alignment 4, and clears the pointer. |
| `00c37570` | `BillboardPanelArrayScalarDtor` | ✅ | dtor | Installs the array vtable, destroys live elements, frees backing storage with alignment 4, and frees the object when the low flag bit is set. |
