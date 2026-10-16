# Class: SoeUtil::Array<ClaimItem,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00830` | `Array_ClaimItem_32_FreeBackingStore` | 🟢 | virtual | Releases param_2 with alignment 8 unless it points at the object own inline storage ((this+0x17) & ~7), in which case nothing is freed; 28 bytes, ret 8. |
| `00a014d0` | `Array_ClaimItem_32_GetBuffer` | 🟢 | virtual | Buffer request of SoeUtil::Array<ClaimItem,32,1>: requests of at most 0x20 elements return the inline storage at (this+0x17) & ~7 with *param_3 = 0x20, larger requests go through the allocation policy 00a00700. |
| `00a02080` | `Array_ClaimItem_32_ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor of SoeUtil::Array<ClaimItem,32,1>: runs the 32-inline destructor body 00a01400 and frees this when bit 0 of the flag is set. |
| `00a027f0` | `Array_ClaimItem_32_Swap` | 🟢 | virtual | Content exchange of two 32-inline claim item arrays through a stack temporary: temp.Append(src.data, src.count) with ecx=&temp, then src.Truncate(src.count) with ecx=src (009fff10 destroys and drops the elements), src.A… |
