# Class: SoeUtil::Array<PacketInGamePurchaseStoreBundleContentResponse::ResponseData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24890` | `— ⚠reject` | ⚪ | virtual | Growth/allocate slot (vtable index 1): adjusts the requested count by 1.25x/1.2x against the current capacity at this+0xc, writes the final count through param_3 and returns AllocateMemory(param_2 * 8, 4); returns the e… |
| `00b24930` | `SoeUtil::ArrayStoreBundleContentResponseDataFreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2): calls FreeMemory(param_1, 4) on the buffer pointer. |
| `00b24950` | `— ⚠reject` | ⚪ | virtual | Virtual storage-swap slot (vtable index 3): with param_3 nonzero it forwards to the other array's slot 3; otherwise XOR-swaps the buffer pointer and the two dwords at this+8/this+0xc with the argument array's. |
| `00b28670` | `SoeUtil::ArrayStoreBundleContentResponseDataDtor → SoeUtil::Array<PacketInGameP…` | ✅ | dtor | Destructor (called from three EH unwind funclets): writes its RTTI vtable, zeroes the size at +8, frees the buffer at +4 with FreeMemory and zeroes the pointer. |
| `00b28980` | `SoeUtil::ArrayStoreBundleContentResponseDataScalarDtor → SoeUtil::Array<PacketI…` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): writes its RTTI vtable, clears the size, frees the buffer with FreeMemory, zeroes the pointer and optionally frees the object itself via _free. |
