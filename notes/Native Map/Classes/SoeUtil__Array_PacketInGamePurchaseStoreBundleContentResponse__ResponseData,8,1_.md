# Class: SoeUtil::Array<PacketInGamePurchaseStoreBundleContentResponse::ResponseData,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b249c0` | `SoeUtil::ArrayStoreBundleContentResponseData8FreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2) of the inline variant: frees only when the pointer is not the inline buffer at (this+0x13) & ~3. |
| `00b286a0` | `— ⚠reject` | ⚪ | dtor | Destructor (called from two EH unwind funclets): writes the 8-element-inline vtable, clears its size, frees the buffer unless it is the inline one, then writes the base Array<...,0,1> vtable and frees/clears that buffer… |
| `00b28760` | `SoeUtil::ArrayStoreBundleContentResponseData8AllocSlot` | ✅ | virtual | Virtual allocate slot (vtable index 1) of the inline array: requests above 8 elements delegate to FUN_00b24890; otherwise it reports capacity 8 through param_3 and returns the inline buffer. |
| `00b2a790` | `ResponseDataArray::Constructor` | ✅ | ctor | Initializes an empty response-data array, switching from the zero-inline vtable to its eight-element-inline vtable. |
| `00b2a900` | `ResponseDataArray::ScalarDeletingDtor` | ✅ | virtual | Calls the response array destructor, then conditionally frees the containing object. |
