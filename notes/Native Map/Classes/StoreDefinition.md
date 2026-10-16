# Class: StoreDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3eb30` | `StoreDefinition::DeletingDtor` | 🟢 | dtor | Calls the StoreDefinition destructor and frees the object when the deleting-destructor flag bit is set. |
| `00c3f930` | `StoreDefinition::AppendBundleEntry` | 🟢 | method | Allocates a bundle entry through the container allocator, links it into the doubly linked sequence, records the tail, and increments entry count. |
| `00c3f9e0` | `StoreDefinition::AssignFrom` | ✅ | virtual | Copies scalar fields and a subobject from another store definition, removes existing bundle nodes, and rebuilds its bundle map from the source. |
| `00c3fa40` | `StoreDefinition::RebuildBundleMap` | ✅ | virtual | Walks linked bundle entries by +0x120 and invokes the bundle insertion routine for each entry's +0x128 data. |
