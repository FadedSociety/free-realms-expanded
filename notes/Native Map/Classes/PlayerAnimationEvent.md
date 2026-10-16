# Class: PlayerAnimationEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00959bf0` | `PlayerAnimationEventCtor ?` | 🟡 | ctor | Calls PlayerEvent base setup, installs PlayerAnimationEvent vtable, and copies three dwords and three bytes from constructor arguments into fields 0x50-0x5e. |
| `00959ca0` | `PlayerAnimationEventDtor ?` | 🟡 | dtor | Restores PlayerAnimationEvent then PlayerEvent vtables and invokes base event cleanup. |
| `00959d20` | `PlayerAnimationEvent::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the event destructor and frees the allocation when deletion flag bit 0 is set. |
| `0096f6e0` | `PlayerAnimationEvent::Dispatch` | ✅ | virtual | Examines bits in param_3, conditionally fetches shared game context and updates animation-related fields through virtual calls, invokes FUN_0096c780 with stored values, then calls a helper and returns its high-byte-clea… |
