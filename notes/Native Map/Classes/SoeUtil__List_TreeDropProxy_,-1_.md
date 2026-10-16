# Class: SoeUtil::List<TreeDropProxy*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081ba60` | `ListTreeDropProxy_DtorBody` | ✅ | dtor | Destructor body (does not free the object): restores SoeUtil::List<TreeDropProxy*,-1>::vftable at [0], then calls 0081b330 to clear the list. |
| `0081bb80` | `ListTreeDropProxy_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag). Restores the List<TreeDropProxy*,-1> vtable at [0], calls 0081b330 to clear the list, and frees this through the _free thunk when flag bit 0 is set. Returns th… |
