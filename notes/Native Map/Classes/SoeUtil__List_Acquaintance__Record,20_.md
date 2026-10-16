# Class: SoeUtil::List<Acquaintance::Record,20>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1af80` | `AcquaintanceRecordList20_Ctor` | 🟢 | ctor | Initializes the list head, tail, and count, switches to the 20-element list vftable, then initializes its pool heads. |
| `00a1b5f0` | `AcquaintanceRecordList_AdjustedPop` | 🟢 | virtual | Adds 0x10 to ECX and jumps to 00a1b510; this is an adjustor thunk and the target body performs the pool pop. |
| `00a1b600` | `AcquaintanceRecordList_AdjustedPush` | 🟢 | virtual | Adds 0x10 to ECX and jumps to 00a1b580; this is an adjustor thunk and the target body performs the pool push. |
| `00a1b610` | `AcquaintanceRecordList20_Dtor` | 🟢 | dtor | Destroys linked nodes, frees both red-black tree branches and header, clears pool fields, switches to the base-list vftable, and drains remaining nodes. |
| `00a1b6e0` | `AcquaintanceRecordList20_ScalarDeletingDtor` | 🟢 | dtor | Calls the list destructor, conditionally frees the list object, and returns it when the deleting flag is clear. |
