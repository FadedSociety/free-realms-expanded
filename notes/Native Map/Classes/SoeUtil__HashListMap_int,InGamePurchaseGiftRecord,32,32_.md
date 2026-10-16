# Class: SoeUtil::HashListMap<int,InGamePurchaseGiftRecord,32,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34360` | `— ⚠reject` | ⚪ | dtor | Installs the derived map vtable, removes all list nodes, frees two auxiliary blocks with alignment 8, then clears the base map. |
| `00c34440` | `HashListMapPoolAllocAdjustorThunk` | ✅ | virtual | Adds 0x194 to ECX and jumps to 00a911c0, adjusting this for the inherited virtual allocation slot. |
| `00c34450` | `HashListMapPoolFreeAdjustorThunk` | ✅ | virtual | Adds 0x194 to ECX and jumps to 00a91230, adjusting this for the inherited virtual free slot. |
| `00c34460` | `HashListMapGiftRecord32FixedScalarDtor` | ✅ | dtor | Calls the derived map destructor and frees the object allocation when the low flag bit is set. |
| `00c34480` | `HashListMapGiftRecord32FixedCtor` | ✅ | ctor | Initializes empty head, tail, size and bucket storage, then installs the derived 32-bucket vtable. |
