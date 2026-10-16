# Class: PlayerLightsaberTrailEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095cee0` | `PlayerLightsaberTrailEvent::PlayerLightsaberTrailEvent ⚠downgrade` | 🟢 | ctor | Constructs the shared base event, installs lightsaber-trail vtables, stores the referenced object, and increments its reference count using the activation callback. |
| `0095cf90` | `PlayerLightsaberTrailEvent::~PlayerLightsaberTrailEvent` | ✅ | dtor | Restores lightsaber-trail vtables, decrements the referenced object's count when positive, and invokes FUN_00957df0. |
| `0095d000` | `PlayerLightsaberTrailEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095d010` | `PlayerLightsaberTrailEvent::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_0095cf90 and frees the object when the deleting flag bit is set. |
| `0095d030` | `PlayerLightsaberTrailEvent::ScalarDeletingDtorThunk` | ✅ | virtual | Forwards to FUN_0095d010 and returns. |
| `0095fdd0` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, obtains a property through FUN_00776740, performs cleanup, and returns the value or zero. |
