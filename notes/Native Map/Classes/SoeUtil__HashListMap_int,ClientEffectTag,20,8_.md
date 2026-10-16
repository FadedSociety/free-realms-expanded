# Class: SoeUtil::HashListMap<int,ClientEffectTag,20,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a170f0` | `HashListMap_ClientEffectTag_20_8_Dtor` | ✅ | dtor | Destructor of the derived effect-tag map: installs the derived vftable, drains the tag nodes with 00a14750, frees the tree at +0x64 through 00a14d10 and FUN_00401c00, clears those fields, then reinstalls the base vftabl… |
| `00a17b70` | `HashListMap_ClientEffectTag_20_8_AllocatorSlot1` | ✅ | virtual | Eight-byte adjustor thunk (bytes 83c164 e918ffffff): adds 0x64 to ecx, retargeting the call to the pool sub-object at this+0x64, and tail-jumps to the pool block allocator 00a17a90. |
| `00a17b80` | `HashListMap_ClientEffectTag_20_8_AllocatorSlot2` | ✅ | virtual | Eight-byte adjustor thunk, the twin of 00a17b70: adds 0x64 to ecx and tail-jumps to the pool block free routine 00a17b00. |
| `00a187a0` | `HashListMap_ClientEffectTag_20_8_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): calls the derived destructor 00a170f0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00a18840` | `HashListMap_ClientEffectTag_20_8_Ctor ⚠downgrade` | 🟢 | ctor | Writes the base HashListMap vftable, zeros head/tail/count and 20 four-byte buckets at +0x14, installs the derived vftable, then calls FUN_00a12f70 with ECX=this+0x64 to initialize the embedded subobject. |
