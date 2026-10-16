# Class: SoeUtil::HashList<AppStoreBundleDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b347d0` | `AppStoreBundleDefinitionHashList::Construct` | ✅ | ctor | Installs base map vtable, zeros counters and bucket storage, then installs the plain HashList specialization vtable. |
| `00b348b0` | `AppStoreBundleDefinitionHashList::ScalarDeletingDtor` | ✅ | dtor | Calls list destructor; when flag bit 0 is set frees object, otherwise returns original pointer. |
