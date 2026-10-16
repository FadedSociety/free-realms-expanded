# Class: ActorProjectileEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00964ff0` | `ActorProjectileEmitter::Constructor` | ✅ | ctor | Calls FUN_00775f70 for the base, installs ActorProjectileEmitter's vtable, initializes its IString member, copies ProjectileParameters, initializes Target-array bookkeeping, and reserves one target. |
| `009650e0` | `ActorProjectileEmitter::IsSlot0cClear` | ✅ | virtual | Calls the object's virtual slot +0xc and returns true exactly when that call returns zero. |
| `00965100` | `ActorProjectileEmitter::ScalarDeletingDestructor` | ✅ | dtor | Calls FUN_010c0f60 for emitter cleanup, frees param_1 when flag bit 0 is set, and otherwise returns the pointer. |
