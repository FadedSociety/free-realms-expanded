# Class: SoeUtil::List<RatingDataEntry,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c85980` | `RatingDataListCtor` | ✅ | ctor | Zeros the list links and count, installs the 32-inline-element list vtable, and initializes the embedded fixed-capacity storage. |
| `00c863b0` | `ListPopFront` | ✅ | virtual | Calls the shared list pop-front helper and returns after the call. |
| `00c863c0` | `ListPushFront` | ✅ | virtual | Calls the shared list push-front helper and returns after the call. |
| `00c863d0` | `List32Dtor` | ✅ | dtor | Installs the 32-inline list vtable, destroys linked nodes, releases inline-block metadata, and then destroys the base list nodes. |
| `00c864c0` | `List32ScalarDeletingDtor` | ✅ | virtual | Destroys the list and inline block metadata, frees the object when flag bit 0 is set, and otherwise returns it. |
