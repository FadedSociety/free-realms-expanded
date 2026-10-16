# Class: PlayerProjectileEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00964e00` | `PlayerProjectileEvent::Constructor` | ✅ | ctor | Calls FUN_0095c230 for the base event, installs both PlayerProjectileEvent vtable pointers, stores the supplied pointer, copies ProjectileParameters, initializes a Target array, reserves one target, and increments a ref… |
| `00964f20` | `PlayerProjectileEvent::Destructor` | ✅ | dtor | Restores both event vtables, decrements the referenced object's count when positive, destroys the Target array elements, frees its buffer, destroys ProjectileParameters, then calls the PlayerEffectEvent destructor. |
| `00964fd0` | `PlayerProjectileEvent::ScalarDeletingDestructor` | ✅ | dtor | Calls FUN_00964f20 for event cleanup, frees param_1 when flag bit 0 is set, and otherwise returns the original pointer. |
| `00965120` | `PlayerProjectileEvent::CreateActorProjectileEmitter` | ✅ | virtual | Obtains a context object through FUN_00ae2470, validates it with a virtual call, allocates an ActorProjectileEmitter from the related object's +0x360 value, invokes FUN_010c1060, and returns the new pointer or zero. |
