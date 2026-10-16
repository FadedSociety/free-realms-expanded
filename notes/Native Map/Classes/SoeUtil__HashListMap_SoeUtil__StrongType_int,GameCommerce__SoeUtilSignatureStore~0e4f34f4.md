# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleCategoryId,1>,GameCommerce::StoreBundleCategoryTree<GameCommerce::StoreBundleCategoryDefinition>::CategoryNode,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34ea0` | `BundleCategoryNodeMap128Ctor` | ✅ | ctor | Installs the category-node map vtable, clears map fields, and zeroes 512 bytes of bucket storage. |
| `00c35fb0` | `BundleCategoryNodeMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x5c bytes with alignment 4. |
| `00c35ff0` | `BundleCategoryNodeMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c3dca0` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Base-class destructor of the category-node map: stores its vtable and frees every slot of the free list at this+8 through 00b2d760 until the head is null. |
| `00c3dd60` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the category-node map: stores the base (128,-1) vtable, frees every slot of the free list at this+8 via 00b2d760, and when the delete flag byte is set releases the object with _free. |
