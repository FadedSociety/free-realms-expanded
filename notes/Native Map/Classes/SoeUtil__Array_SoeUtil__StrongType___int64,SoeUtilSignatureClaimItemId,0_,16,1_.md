# Class: SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a009f0` | `Array_ClaimItemId_16_FreeBackingStore` | 🟢 | virtual | Releases param_2 with alignment 8 unless it equals the object own inline storage ((this+0x17) & ~7); same body as 00a00830. |
| `00a015f0` | `Array_ClaimItemId_16_GetBuffer` | 🟢 | virtual | Buffer request of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>: requests of at most 0x10 elements return the inline storage at (this+0x17) & ~7 with *param_3 = 0x10, larger requests go… |
| `00a020a0` | `Array_ClaimItemId_16_ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor of SoeUtil::Array<SoeUtil::StrongType<__int64,SoeUtilSignatureClaimItemId,0>,16,1>: runs the 16-inline destructor body 00a01530 and frees this when the delete flag is set. |
| `00a02220` | `Array_ClaimItemId_16_Swap` | 🟢 | virtual | Content exchange of two 16-inline claim item id arrays through a stack temporary (decoded call sites: temp.Append(src.data, src.count) with ecx=&temp at 00a02278; src.count = 0; src.Append(this.data, this.count) with ec… |
