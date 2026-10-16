# Class: SoeUtil::List<PetClientInfo_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccc020` | `PetClientInfoList::Clear` | 🟢 | method | Repeatedly removes the head node, repairs predecessor/successor links, decrements list size, and invokes the element cleanup slot. |
| `00ccc180` | `PetClientInfoListDtor` | ✅ | dtor | Restores the typed list vtable and clears all linked nodes through the list cleanup helper. |
| `00ccc1a0` | `PetClientInfoListScalarDeletingDtor` | ✅ | dtor | Restores the typed list vtable, clears its nodes, and frees the list allocation when the deleting flag is set. |
