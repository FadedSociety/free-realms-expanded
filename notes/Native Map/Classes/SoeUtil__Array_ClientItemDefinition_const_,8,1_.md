# Class: SoeUtil::Array<ClientItemDefinition_const*,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00b40` | `Array_ItemDefPtr_8_FreeBackingStore` | 🟢 | virtual | Releases param_2 with alignment 4 unless it equals the object own 4-byte-aligned inline storage ((this+0x13) & ~3); 28 bytes, ret 8. |
| `00a018e0` | `Array_ItemDefPtr_8_GetBuffer` | 🟢 | virtual | Buffer request of SoeUtil::Array<ClientItemDefinition const*,8,1>: requests of at most 8 elements return the inline storage (this+0x13) & ~3 with *param_3 = 8, larger requests go through the allocation policy 00a00a10. |
| `00a020c0` | `Array_ItemDefPtr_8_ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor of SoeUtil::Array<ClientItemDefinition const*,8,1>: runs the 8-inline destructor body 00a01820 and frees this when the delete flag is set. |
| `00a02310` | `Array_ItemDefPtr_8_Swap` | 🟢 | virtual | Content exchange of two 8-inline item definition pointer arrays through a stack temporary: temp.Append(src.data, src.count) (ecx=&temp), src.count = 0, src.Append(this.data, this.count) with ecx=src, this.count = 0, thi… |
