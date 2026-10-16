# Class: PlayerLightEmitterEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095c960` | `PlayerLightEmitterEvent::PlayerLightEmitterEvent` | ✅ | ctor | Constructs the base event, installs light-emitter event vtables, stores the referenced object, and increments its reference count with the zero-to-one callback. |
| `0095ca10` | `PlayerLightEmitterEvent::~PlayerLightEmitterEvent` | ✅ | dtor | Restores light-emitter vtables, decrements the referenced object's count when positive, and calls FUN_00957df0. |
| `0095ca80` | `PlayerLightEmitterEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095ca90` | `PlayerLightEmitterEvent::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_0095ca10 and frees the object when the deleting flag bit is set. |
| `0095cab0` | `— ⚠reject` | ⚪ | virtual | Forwards to FUN_0095ca90 and returns. |
| `0095fa30` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, obtains a resource value through FUN_007763f0, performs cleanup, and returns the value or zero. |
