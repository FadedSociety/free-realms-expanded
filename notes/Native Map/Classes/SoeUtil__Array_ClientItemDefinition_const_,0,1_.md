# Class: SoeUtil::Array<ClientItemDefinition_const*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00a10` | `— ⚠reject` | ⚪ | virtual | Capacity policy of a 4-byte-element array: grows the request by 5/4 when it exceeds the current capacity, shrinks it by 6/5 when it is well below it, writes the chosen capacity to *param_3 and returns the existing block… |
| `00a00ab0` | `Array_ItemDefPtr_0_FreeBackingStore` | 🟢 | virtual | Free wrapper: releases the buffer with 00401c00 at alignment 4 (same body as 00a006e0). |
| `00a00ad0` | `— ⚠reject` | ⚪ | virtual | Array swap slot (same body as 00a007c0): with the flag set it tail-jumps into virtual slot 3 of param_2 passing this and 0, otherwise it xor-swaps the data pointer, size and capacity at +4/+8/+0xc with param_2[1..3]. |
| `00a01b10` | `Array_ItemDefPtr_0_ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::Array<ClientItemDefinition const*,0,1>: stores the class vtable, resets the count, frees the backing store with alignment 4 and frees this when the delete flag is set. |
