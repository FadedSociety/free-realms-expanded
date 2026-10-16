# Class: BaseItemDefinitionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c92c10` | `BaseItemDefinitionDataSource::ColumnName` | ✅ | virtual | Maps indices zero through forty to item definition, display, model, purchase, ownership, equipment, stack, and tint-related field names. |
| `00c93380` | `— ⚠reject` | ⚪ | virtual | Searches the item cache by key and either invalidates the matching cached definition or calls the update hook when no cached entry exists. |
| `00c933f0` | `BaseItemDefinitionDataSource::CleanupSlot3` | ✅ | virtual | Calls 009879a0 with this adjusted to the embedded subobject at +0x3654, restores ECX, and tail-jumps to 010c8e90. |
| `00c93990` | `BaseItemDefinitionDataSource::RemoveCachedDefinition` | ✅ | virtual | Searches the 1024-bucket item cache for the requested ID, removes a matching node, and invokes the update hook for that ID. |
| `00c942c0` | `— ⚠reject` | ⚪ | virtual | Finds the item-entry node in its 1024-bucket cache, removes the matching linked-list item, updates the associated ordered structure, and calls the ID update hook. |
| `00c956c0` | `BaseItemDefinitionDataSource::GetFieldValue` | ✅ | virtual | Switches on a column index, resolves item-related fields, formats values into the output string, and returns through the string provider. |
| `00c96120` | `BaseItemDefinitionDataSource::ItemEntryHashListCtor` | ✅ | ctor | Calls the embedded hash-list initializer, installs the HashList<BaseItemDefinitionDataSource::ItemEntry,1024,256> vtable, and returns the object. |
| `00c963b0` | `BaseItemDefinitionDataSource::GetRow` | ✅ | virtual | Refreshes if flagged, obtains the row key through a virtual call, searches the cached definition chain, requests a missing definition, then dispatches the row to a virtual formatter. |
| `00c96460` | `BaseItemDefinitionDataSource::GetCount` | ✅ | virtual | Calls the refresh helper when the dirty byte at this+0x5284 is set, then returns the 32-bit field at this+0x527c. |
| `00c968b0` | `BaseItemDefinitionDataSource::Dtor` | ✅ | dtor | Restores base and embedded data-source vtables, conditionally unregisters from shared state, destroys the filter map and embedded hashes, then releases holder storage. |
| `00c96a30` | `BaseItemDefinitionDataSource::Ctor` | ✅ | ctor | Initializes the base data-source object, hash-list and integer hash members, state flags, filter map, and registration in shared data-source state. |
| `00c96b40` | `BaseItemDefinitionDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the base data-source destructor, frees the object when the deleting flag bit is set, and otherwise returns its pointer. |
