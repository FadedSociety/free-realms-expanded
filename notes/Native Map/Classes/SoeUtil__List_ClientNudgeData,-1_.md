# Class: SoeUtil::List<ClientNudgeData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1beb0` | `ClientNudgeDataList_Dtor` | 🟢 | dtor | Installs the ClientNudgeData list vftable and repeatedly destroys the head node through 00909e60 until empty. |
| `00a1bf20` | `ClientNudgeDataList_ScalarDeletingDtor` | 🟢 | dtor | Destroys ClientNudgeData list nodes through 00909e60, optionally frees the object, and otherwise returns it. |
