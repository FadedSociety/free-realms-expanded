# Class: ItemDefinitionDsUpdateDispatcher

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c92f70` | `ItemDefinitionDsUpdateDispatcher::DtorBody` | ✅ | dtor | Restores the Singleton<ItemDefinitionDsUpdateDispatcher> vtable during unwind cleanup. |
| `00c93090` | `ItemDefinitionDsUpdateDispatcher::ScalarDeletingDtor → Singleton<ItemDefinition…` | ✅ | dtor | Restores the singleton dispatcher vtable and conditionally frees the object when the deleting flag's low bit is set. |
| `00c93410` | `ItemDefinitionDsUpdateDispatcher::Ctor` | ✅ | ctor | Installs the singleton and dispatcher vtables and zeros the three dispatcher state fields. |
| `00c93470` | `ItemDefinitionDsUpdateDispatcher::Dtor` | ✅ | dtor | Restores the dispatcher and singleton vtables on the object. |
| `00c93c20` | `ItemDefinitionDsUpdateDispatcher::ScalarDeletingDtor` | ✅ | dtor | Restores the dispatcher vtable, then the singleton base vtable, and frees the object only when the deleting flag is set. |
| `00c93c90` | `ItemDefinitionDsUpdateDispatcher::NotifyListeners` | 🟢 | function | Adjusts this by four bytes, passes the embedded listener list and callback 00c93300 to FUN_00c93740, then returns. |
