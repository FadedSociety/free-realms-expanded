# Class: SoeUtil::HashListMap<int,GameCommerce::StoreShortcutDefinition,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c34c30` | `StoreShortcutMap64Ctor` | ✅ | ctor | Installs the shortcut map vtable, clears map fields, and zeroes its 64 bucket heads. |
| `00c360c0` | `ShortcutMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x24 bytes with alignment 4. |
| `00c36100` | `ShortcutMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 4. |
| `00c3aff0` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Base-class destructor of the (64,-1) store-shortcut map: stores its vtable and frees every slot of the free list at this+8 through 00c37ac0 until the head is null. |
| `00c3b1c0` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the (64,-1) store-shortcut map: stores the base vtable, frees every slot of the free list at this+8 via 00c37ac0, and when the delete flag byte is set releases the object with _free. |
