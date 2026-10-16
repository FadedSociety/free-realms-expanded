# Class: PlayerHitpointEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae20d0` | `PlayerHitpointEventBaseDtor` | ✅ | dtor | Writes PlayerHitpointEvent then PlayerEvent vtables and invokes the PlayerEvent base cleanup helper. |
| `00ae2190` | `PlayerHitpointEventCtor` | ✅ | ctor | Constructs PlayerEvent base, copies six constructor values into event offsets +0x54 through +0x68, sets event vtable, and assigns an incrementing context counter when the game context exists. |
| `00ae2260` | `PlayerHitpointEvent::ScalarDeletingDtor` | ✅ | dtor | Calls the event destructor body, optionally frees this on flag bit 0, and returns this otherwise. |
| `00ae2280` | `— ⚠reject` | ⚪ | virtual | If apply flag bit 0 is set, updates the current player hitpoint state, optionally applies a second bounded value, invokes its virtual follow-up, and returns packed status. |
