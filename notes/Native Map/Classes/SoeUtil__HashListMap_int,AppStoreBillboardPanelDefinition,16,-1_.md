# Class: SoeUtil::HashListMap<int,AppStoreBillboardPanelDefinition,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34d30` | `BillboardPanelMap16Ctor` | ✅ | ctor | Installs the billboard-panel map vtable, clears map fields, and zeroes its 16 bucket heads. |
| `00c36410` | `BillboardPanelMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x6c bytes with alignment 4. |
| `00c36450` | `BillboardPanelMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c3b0f0` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Base-class destructor of the (16,-1) billboard-panel map: stores its vtable and frees every slot of the free list at this+8 through 00c37bf0 until the head is null. |
| `00c3b200` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the (16,-1) billboard-panel map: stores the base vtable, frees every slot of the free list at this+8 via 00c37bf0, and when the delete flag byte is set releases the object with _free. |
