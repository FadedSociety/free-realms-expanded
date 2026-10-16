# Class: ItemDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00720d60` | `ItemDefinitionManager::~ItemDefinitionManager` | 🟢 | dtor | Restores ItemDefinitionManager::vftable at +0 and at +4 (the secondary base slot) and then writes BaseDefinitionManager::vftable at +0, i.e. the base-object destruction order of the manager. |
| `007218e0` | `ItemDefinitionManager::ItemDefinitionManager` | ✅ | ctor | Constructs the definition-manager base chain in place: writes BaseDefinitionManager::vftable then BaseItemDefinitionLookup::vftable into +0/+4 and then the two ItemDefinitionManager vtables into +0/+4. |
| `00722750` | `ItemDefinitionManager::ScalarDeletingDestructor` | ✅ | dtor | Restores the two ItemDefinitionManager vtables at +0/+4, the BaseDefinitionManager vtable at +0, and frees through FUN_00401c00 when bit 0 of the flag byte is set. |
| `007227c0` | `ItemDefinitionManager::RemoveAll` | 🟢 | method | Walks the doubly-linked list from this+4 (tail fields +0x250/+0x254 on the nodes): unlinks every node from the 10000-bucket hash (00721490), fixes the neighbour links, decrements the count at +0xc and invokes each node … |
