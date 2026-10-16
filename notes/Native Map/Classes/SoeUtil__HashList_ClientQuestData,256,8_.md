# Class: SoeUtil::HashList<ClientQuestData,256,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7c9c0` | `ClientQuestHashList::Construct → SoeUtil::HashList<ClientQuestData,256,8>::Cons…` | ✅ | ctor | Calls the typed map constructor on the supplied object, then installs the SoeUtil::HashList<ClientQuestData,256,8> vtable. |
| `00c7ca20` | `ClientQuestHashList::DeletingDtor → SoeUtil::HashList<ClientQuestData,256,8>::D…` | ✅ | dtor | Calls the quest-data store destructor body and frees the list object when flag bit 0 is set; otherwise restores exception state and returns the pointer. |
