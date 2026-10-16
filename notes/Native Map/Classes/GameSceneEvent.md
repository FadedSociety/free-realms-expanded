# Class: GameSceneEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b8690` | `GameSceneEvent::GameSceneEvent` | ✅ | ctor | Installs GuiEventHandler then GameSceneEvent vtables, stores param_2, and registers the object with event id 0xe20548a9. |
| `008b8710` | `GameSceneEvent::DtorBody` | ✅ | dtor | Restores GameSceneEvent vtable, unregisters event id 0xe20548a9 for this object, then restores GuiEventHandler vtable. |
| `008b8780` | `GameSceneEvent::Handle` | 🟢 | virtual | When the global scene state at +0xddc0 is zero, calls FUN_009c04b0 with an event-owned field and FUN_009c0a40 with param_3, then returns 1. |
| `008b87c0` | `GameSceneEvent::ScalarDeletingDtor` | ✅ | dtor | Calls GameSceneEvent cleanup and frees the object when the deleting flag bit is set. |
