# Class: SoeUtil::List<RatingDataEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c85140` | `ListDtor` | ✅ | dtor | Installs the list vtable, then repeatedly destroys the node at object offset 4 until the list is empty. |
| `00c854b0` | `ListScalarDeletingDtor` | ✅ | virtual | Destroys all linked nodes, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
