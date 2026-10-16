# Class: SoeUtil::List<PetTrickInfo_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccc080` | `PetTrickInfoList::Clear` | 🟢 | method | Repeatedly removes the head node, updates both adjacent links and the list endpoints, decrements the count, and invokes the cleanup slot. |
| `00ccc190` | `PetTrickInfoListDtor` | ✅ | dtor | Restores the typed trick-list vtable and clears all linked nodes through the list cleanup helper. |
| `00ccc1d0` | `PetTrickInfoListScalarDeletingDtor` | ✅ | dtor | Restores the typed trick-list vtable, clears its nodes, and frees the list allocation when the deleting flag is set. |
