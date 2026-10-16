# Class: SoeUtil::HashListMap<int,GameCommerce::StoreBillboardDefinition<AppStoreBillboardPanelDefinition>,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34ca0` | `StoreBillboardMap64Ctor` | ✅ | ctor | Installs the billboard map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c361d0` | `BillboardDefinitionMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x1a0 bytes with alignment 4. |
| `00c36210` | `BillboardDefinitionMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c3dcd0` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Base-class destructor of the billboard-definition map: stores its vtable and frees every slot of the free list at this+8 through 00c38be0 until the head is null. |
| `00c3dda0` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the billboard-definition map: stores the base vtable, frees every slot of the free list at this+8 via 00c38be0, and when the delete flag byte is set releases the object with _free. |
