# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>,GameCommerce::StoreBundleGroupDefinition,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34bd0` | `BundleGroupMap16Ctor` | ✅ | ctor | Installs the bundle-group map vtable, clears map fields, and zeroes its 16 bucket heads. |
| `00c35f00` | `BundleGroupMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x80 bytes with alignment 4. |
| `00c35f40` | `BundleGroupMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c3af30` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Base-class destructor of the (16,-1) store-bundle-group map: stores its vtable and frees every slot of the free list at this+8 through 00b28d20 until the head is null; nothing else is touched. |
| `00c3b140` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the (16,-1) store-bundle-group map: stores the base vtable, frees every slot of the free list at this+8 via 00b28d20, and when the delete flag byte is set releases the object with _free. |
