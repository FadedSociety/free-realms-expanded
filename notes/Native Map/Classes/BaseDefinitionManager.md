# Class: BaseDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00720d50` | `BaseDefinitionManager::~BaseDefinitionManager` | 🟢 | dtor | Writes BaseDefinitionManager::vftable into *param_1 and returns; the only callers are EH unwind funclets. |
| `00722720` | `BaseDefinitionManager::ScalarDeletingDestructor` | ✅ | dtor | Installs BaseDefinitionManager::vftable into this and calls FUN_00401c00 (FreeMemory) when bit 0 of the flag byte is set. |
| `00bb9b80` | `BaseDefinitionManager::~BaseDefinitionManager ⚠downgrade` | 🟢 | dtor | Replaces the object's vtable with BaseDefinitionManager::vftable and returns; EH unwind funclets call it during cleanup. |
| `00bbd360` | `BaseDefinitionManager::~BaseDefinitionManager` | ✅ | dtor | Restores the BaseDefinitionManager vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
