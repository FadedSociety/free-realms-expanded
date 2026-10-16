# Class: SoeUtil::HashListMap<int,GameCommerce::StoreBundleCategoryGroupDefinition,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34c00` | `BundleCategoryGroupMap64Ctor` | ✅ | ctor | Installs the category-group map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c36010` | `CategoryGroupMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x24 bytes with alignment 4. |
| `00c36050` | `CategoryGroupMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c3afc0` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Base-class destructor of the (64,-1) store-bundle-category-group map: stores its vtable and frees every slot of the free list at this+8 through 00b28dd0 until the head is null. |
| `00c3b180` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the (64,-1) store-bundle-category-group map: stores the base vtable, frees every slot of the free list at this+8 via 00b28dd0, and when the delete flag byte is set releases the object with … |
