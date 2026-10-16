# Class: PlayerMeshEmitterEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095cd80` | `PlayerMeshEmitterEvent::PlayerMeshEmitterEvent` | 🟢 | ctor | Calls the PlayerEffectEvent base constructor, installs PlayerMeshEmitterEvent vtables at this and this+0x50, stores the input object pointer at this+0xa0, increments that object?s +8 count, and calls its virtual slot +0… |
| `0095ce30` | `PlayerMeshEmitterEvent::~PlayerMeshEmitterEvent` | ✅ | dtor | Restores mesh-emitter vtables, decrements the referenced object's count if positive, and runs FUN_00957df0. |
| `0095cea0` | `PlayerMeshEmitterEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095ceb0` | `PlayerMeshEmitterEvent::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_0095ce30 and conditionally frees the object when deleting flag bit 0 is set. |
| `0095ced0` | `PlayerMeshEmitterEvent::ScalarDeletingDtorThunk` | ✅ | virtual | Forwards to FUN_0095ceb0 and returns. |
| `0095fcf0` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, obtains a value through FUN_00776690, performs cleanup, and returns the value or zero. |
