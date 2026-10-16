# Class: SoeUtil::HashListMap<int,ItemDefinitionMetaData,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c71c70` | `HashListMap256BaseDtor` | ✅ | dtor | Stores the 256-bucket HashListMap vtable, then calls 00c70980 on the head at +8 until the head is null. No free. |
| `00c71d10` | `HashListMap256::ScalarDeletingDtor` | ✅ | dtor | Stores the 256-bucket HashListMap vtable, drains the head at +8 through 00c70980, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
| `00c72280` | `— ⚠reject` | ⚪ | dtor | Under an EH frame stores the 256-bucket HashListMap vtable, then calls 00c70980 on the head at +8 (passing a stack-cookie expression as extra argument) until the head is null. No free. |
