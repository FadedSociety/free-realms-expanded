# Class: SoeUtil::Array<TooltipDataSource::Node,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a757f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1]. Same capacity policy shape as 00a71580: reads the capacity at this+0xc; with flag 0 it grows to 5/4 of a larger request or shrinks to 6/5 when capacity exceeds 4/3 of the request; with flag nonzero it … |
| `00a75890` | `TooltipDataSourceNodeArray::FreeBuffer` | ✅ | virtual | Virtual slot [2]. Reads its first stack argument and calls FreeMemory(pointer, 4); returns with ret 8. |
| `00a758b0` | `TooltipDataSourceNodeArray::SwapOrDelegate ?` | 🟡 | virtual | Virtual slot [3]. Same shape as 00a71640: when the byte argument is nonzero it calls vtable slot +0xc of the other object and returns; otherwise it exchanges buffer (+4) and XOR-swaps count (+8) and capacity (+0xc) with… |
| `00a761e0` | `TooltipDataSourceNodeArray_DtorBody ⚠downgrade` | 🟢 | dtor | Destructor body: stores the SoeUtil::Array<TooltipDataSource::Node,0,1> vtable at [this], calls 00a75d50 (ShrinkBy) with the element count at [this+8] so every 32-byte element is destroyed, then frees the element buffer… |
| `00a76360` | `TooltipDataSourceNodeArray_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores the array vtable at [this], calls 00a75d50 with the count at [this+8] to destroy all elements, frees the buffer at [this+4] through 00401c00 (alignment 4), nulls it, th… |
