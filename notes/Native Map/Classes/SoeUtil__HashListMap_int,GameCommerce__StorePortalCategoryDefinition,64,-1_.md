# Class: SoeUtil::HashListMap<int,GameCommerce::StorePortalCategoryDefinition,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34c60` | `StorePortalCategoryMap64Ctor` | ✅ | ctor | Installs the portal-category map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c36120` | `PortalCategoryMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x5c bytes with alignment 4. |
| `00c36160` | `PortalCategoryMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c38b70` | `PortalCategoryMapDtor` | ✅ | dtor | Installs the portal-category map vtable and repeatedly removes its head node through 00c371a0. |
| `00c38f40` | `PortalCategoryMapScalarDeletingDtor` | ✅ | dtor | Clears all portal-category nodes through 00c371a0 and frees the map object when the low flag bit is set. |
