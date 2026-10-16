# Class: CollectionCategoryManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009fd4e0` | `CollectionCategoryManager::LoadCategories` | ✅ | method | Reads CollectionCategories.txt, parses caret-delimited id/name/icon rows, inserts non-comment rows into linked hash buckets, then refreshes the category table. |
| `009fd850` | `CollectionCategoryManager::Dtor` | ✅ | dtor | Unregisters BaseClient.CollectionCategory, releases its data-source reference, and removes each category node from the owned hash/list table. |
| `009fd9d0` | `CollectionCategoryManager::Ctor` | 🟢 | ctor | Clears manager fields, stores its client context, allocates a category data source, and retains the resulting object reference. |
