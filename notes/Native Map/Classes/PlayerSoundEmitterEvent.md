# Class: PlayerSoundEmitterEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095c6a0` | `PlayerSoundEmitterEvent::PlayerSoundEmitterEvent` | ✅ | ctor | Constructs the PlayerEffectEvent base, installs sound-emitter vtables, stores the referenced object, and increments its reference count with the zero-to-one callback. |
| `0095c750` | `PlayerSoundEmitterEvent::~PlayerSoundEmitterEvent` | ✅ | dtor | Restores the sound-emitter vtables, decrements its referenced object's count when positive, then calls FUN_00957df0. |
| `0095c7c0` | `PlayerSoundEmitterEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095c7d0` | `PlayerSoundEmitterEvent::ScalarDeletingDtor` | ✅ | virtual | Runs FUN_0095c750, then frees param_1 with _free when the deleting flag bit is set. |
| `0095c7f0` | `PlayerSoundEmitterEvent::ScalarDeletingDtorAdjustorThunk` | ✅ | virtual | Subtracts 0x50 from ECX, then jumps to 0095c7d0. |
| `0095f860` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, obtains a value through FUN_00776160, performs cleanup, and returns the value or zero. |
