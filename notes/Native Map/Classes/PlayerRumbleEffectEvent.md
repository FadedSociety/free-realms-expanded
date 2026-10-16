# Class: PlayerRumbleEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095cc20` | `PlayerRumbleEffectEvent::PlayerRumbleEffectEvent ⚠downgrade` | 🟢 | ctor | Constructs the base event, installs rumble-effect vtables, stores a referenced object, and increments its reference count with its activation callback. |
| `0095ccd0` | `PlayerRumbleEffectEvent::~PlayerRumbleEffectEvent` | ✅ | dtor | Restores rumble-effect vtables, decrements the referenced object's count when positive, then invokes FUN_00957df0. |
| `0095cd40` | `PlayerRumbleEffectEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095cd50` | `PlayerRumbleEffectEvent::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_0095ccd0 and conditionally frees param_1 when deleting flag bit 0 is set. |
| `0095cd70` | `PlayerRumbleEffectEvent::ScalarDeletingDtorThunk` | ✅ | virtual | Forwards to FUN_0095cd50 and returns. |
| `0095fc10` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, reads a value through FUN_007765b0, performs cleanup, and returns the value or zero. |
