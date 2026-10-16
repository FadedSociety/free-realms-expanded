# Class: SoeUtil::List<ClientNudgeData,4>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1c160` | `ClientNudgeDataList4_Ctor` | 🟢 | ctor | Initializes base and fixed-list vftables, clears list and pool fields, initializes pool sentinel links, and sets capacity four. |
| `00a1cbc0` | `ClientNudgeDataList_AdjustedPop` | 🟢 | virtual | Adds 0x10 to ECX and jumps to 00a1cae0; this is an adjustor thunk and the target performs the pool pop. |
| `00a1cbd0` | `ClientNudgeDataList_AdjustedPush` | 🟢 | virtual | Adds 0x10 to ECX and jumps to 00a1cb50; this is an adjustor thunk and the target performs the pool push. |
| `00a1cbe0` | `ClientNudgeDataList4_Dtor` | 🟢 | dtor | Destroys list elements, frees both tree branches and header, clears the pool, switches to the base-list vftable, and drains remaining nodes. |
| `00a1ccb0` | `ClientNudgeDataList4_ScalarDeletingDtor` | 🟢 | dtor | Calls the fixed-list destructor, optionally frees this object, and otherwise returns it. |
