# Class: SoeUtil::List<_anon_95091C26::MemberStub,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb58c0` | `MemberListNodeAllocate` | ✅ | virtual | Initializes allocation state, uses the active memory manager when available, falls back to the global manager, and requests 0xa0 bytes aligned to 8. |
| `00cb5900` | `MemberListNodeFree` | ✅ | virtual | Frees the supplied allocation through the aligned memory helper with alignment 8. |
| `00cb60b0` | `MemberListDtor` | ✅ | dtor | Restores the list vtable and removes each current head node until the list is empty. |
| `00cb60e0` | `MemberListScalarDeletingDtor` | ✅ | dtor | Destroys all member nodes and frees the list object when the deleting flag's low bit is set. |
