# Class: SoeUtil::List<DefinitionEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b07590` | `AllocateDefinitionEntryNode` | 🟢 | function | Initializes allocator selection, tries the selected or global allocator, then falls back to 0x1c bytes aligned to 4. |
| `00b075d0` | `FreeDefinitionEntryNode` | 🟢 | function | Forwards the supplied pointer to the aligned free routine with alignment 4. |
| `00b08150` | `DefinitionEntryListDestructor` | 🟢 | dtor | Writes the DefinitionEntry list vtable and jumps to 00b07e70 for list cleanup. |
| `00b081a0` | `ScalarDeletingDtor ? → SoeUtil::List<DefinitionEntry,-1>::ScalarDeletingDestruc…` | 🟡 | dtor | Sets the DefinitionEntry list vtable, clears entries, and optionally frees the containing allocation. |
