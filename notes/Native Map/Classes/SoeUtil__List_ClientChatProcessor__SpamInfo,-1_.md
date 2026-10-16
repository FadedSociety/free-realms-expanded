# Class: SoeUtil::List<ClientChatProcessor::SpamInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0e370` | `List::FreeNode → SoeUtil::List<ClientChatProcessor::SpamInfo,-1>::FreeNode` | ✅ | virtual | Frees the supplied node pointer using alignment 8. |
| `00b12860` | `SpamInfoListBaseDtor` | ✅ | dtor | Installs the unbounded SpamInfo list vtable and repeatedly destroys the head node until the list is empty. |
| `00b129c0` | `SpamInfoListScalarDeletingDtor` | ✅ | dtor | Drains the SpamInfo list, tests flag bit 0, conditionally frees param_1, and otherwise returns the original pointer. |
| `00b13f50` | `SpamInfoListBaseDtor` | ✅ | dtor | Drains SpamInfo nodes, frees both child subtrees of the block tree, clears tree metadata, and drains the unbounded base list; installs both the 256-specialization and base-list vtables during teardown. |
