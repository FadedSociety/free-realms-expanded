# Class: SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a008c0` | `— ⚠reject` | ⚪ | virtual | Capacity policy of an 8-byte-element array: grows the request by 5/4 when it exceeds the current capacity, shrinks it by 6/5 when it is below three quarters of it, writes the chosen capacity to *param_3 and returns the … |
| `00a00960` | `Array_ClaimItemId_0_FreeBackingStore` | 🟢 | virtual | Free wrapper: releases the buffer with 00401c00 at alignment 8 (same body as 00a007a0). |
| `00a00980` | `— ⚠reject` | ⚪ | virtual | Array swap slot (same body as 00a007c0): with the flag set it tail-jumps into virtual slot 3 of param_2 passing this and 0, otherwise it xor-swaps the data pointer, size and capacity dwords at +4/+8/+0xc with param_2[1.… |
| `00a01ad0` | `Array_ClaimItemId_0_ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,0,1>: stores the class vtable, resets the count, frees the backing store and frees this when the delete flag is set. |
