# Class: PlayerCameraEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095cac0` | `PlayerCameraEffectEvent::PlayerCameraEffectEvent` | ✅ | ctor | Constructs the base event, installs camera-effect vtables, stores the referenced object, and increments its reference count with the activation callback. |
| `0095cb70` | `PlayerCameraEffectEvent::~PlayerCameraEffectEvent` | ✅ | dtor | Restores camera-effect vtables, decrements the referenced object's count when positive, and invokes FUN_00957df0. |
| `0095cbe0` | `PlayerCameraEffectEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095cbf0` | `PlayerCameraEffectEvent::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_0095cb70 and frees param_1 when the deleting flag bit is set. |
| `0095cc10` | `— ⚠reject` | ⚪ | virtual | Forwards to FUN_0095cbf0 and returns. |
| `0095fb20` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, obtains a resource value through FUN_007764d0, performs cleanup, and returns the value or zero. |
