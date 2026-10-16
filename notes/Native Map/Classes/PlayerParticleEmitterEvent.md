# Class: PlayerParticleEmitterEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095c530` | `PlayerParticleEmitterEvent::PlayerParticleEmitterEvent` | ✅ | ctor | Constructs the PlayerEffectEvent base, installs particle-emitter vtables, stores two constructor values, and increments the referenced object count with its zero-to-one callback. |
| `0095c5f0` | `PlayerParticleEmitterEvent::~PlayerParticleEmitterEvent` | ✅ | dtor | Restores the particle-emitter vtables, decrements the referenced object count if positive, then runs FUN_00957df0 cleanup. |
| `0095c660` | `PlayerParticleEmitterEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095c670` | `PlayerParticleEmitterEvent::ScalarDeletingDtor` | ✅ | virtual | Calls the class destructor and frees the object with _free when the deleting flag bit is set; otherwise returns the object pointer. |
| `0095c690` | `— ⚠reject` | ⚪ | virtual | Forwards to FUN_0095c670 and returns; the target performs conditional object deallocation. |
| `0095f760` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, selects an active handle when present, reads a resource value through FUN_00776f40, performs cleanup, and returns it or zero. |
