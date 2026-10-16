# Class: ClientTimerEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c52b0` | `ClientTimerEventCtor` | ✅ | ctor | Constructs the ExecuteLuaEvent base and installs ClientTimerEvent's vtable, then clears its timer-specific flag byte. |
| `009c5330` | `ClientTimerEventScalarDeletingDtor` | ✅ | dtor | Calls the timer-event teardown routine, conditionally frees the object when the deleting flag is set, and otherwise returns its pointer. |
