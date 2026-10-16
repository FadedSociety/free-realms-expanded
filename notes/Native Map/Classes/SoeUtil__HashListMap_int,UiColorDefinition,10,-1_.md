# Class: SoeUtil::HashListMap<int,UiColorDefinition,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bde80` | `UiColorDefinitionHashListMap10::Allocate` | ✅ | virtual | Allocates a 0x18-byte map entry aligned to 4 after trying the thread and global allocators. |
| `008bdec0` | `UiColorDefinitionHashListMap10::FreeAllocation` | ✅ | virtual | Releases the supplied map entry with 4-byte alignment. |
| `008e0e80` | `HashListMapUiColorDefinition_Dtor` | ✅ | dtor | Stores the UiColorDefinition map vtable, then jumps to the generic map cleanup routine. |
| `008e2290` | `HashListMapUiColorDefinition_ScalarDeletingDtor` | ✅ | dtor | Installs the UI color map vtable, runs map cleanup, and optionally frees the object when flag bit 0 is set. |
