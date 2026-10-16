# Class: PlayerRemoveEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00959fa0` | `PlayerRemoveEventCtor ?` | 🟡 | ctor | Calls PlayerEvent base initializer and installs PlayerRemoveEvent vtable. |
| `0095a020` | `PlayerRemoveEventDtor ?` | 🟡 | dtor | Restores PlayerRemoveEvent then PlayerEvent vtables and calls base event cleanup. |
| `0095a080` | `PlayerRemoveEvent::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the event destructor and frees the allocation when deletion flag bit 0 is set. |
| `0096b320` | `PlayerRemoveEvent::ReleaseContextHandles ?` | 🟡 | virtual | Virtual slot 1 of PlayerRemoveEvent::vftable with a byte flag argument. It does not use its receiver: it obtains an object from 00ae2470 and, when nonnull, calls 007e7f00 and 00969910 on handles at object+0x6c8 and obje… |
