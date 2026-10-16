# Class: SoeGems::TextIndex<int>::ProcessingNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8e210` | `— ⚠reject` | ⚪ | ctor | Installs WeightedProvider and ProcessingNode vtables, stores two child providers and a weight, and primes both child cursors. |
| `00b8e2a0` | `ProcessingNode::GetSize` | ✅ | virtual | Calls size virtual methods on both child providers and returns the sum. |
| `00b8e2c0` | `— ⚠reject` | ⚪ | virtual | Merges two child streams by key/weight comparison, copies the selected four-dword value, advances one child or both, and returns status. |
| `00b8e460` | `ProcessingNode::DtorBase` | ✅ | dtor | Restores ProcessingNode vtable, invokes deleting destructors on each non-null child, and restores WeightedProvider vtable. |
| `00b8e4e0` | `ProcessingNode::ScalarDeletingDtor` | ✅ | dtor | Calls the ProcessingNode destructor and frees the object when scalar-delete flag bit zero is set. |
