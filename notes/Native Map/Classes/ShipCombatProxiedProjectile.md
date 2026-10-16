# Class: ShipCombatProxiedProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a50d00` | `ShipCombatProxiedProjectile::CtorAdapter` | 🟢 | callback | EH-framed constructor adapter to 00cabeb0. The decoded call sequence places the incoming object in ECX and forwards the three dereferenced stack arguments; the Ghidra stack-cookie expression is not an argument. |
| `00a53d20` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor shape: under an SEH frame with the stack cookie it calls FUN_00cabd60 (the ShipCombatProxiedProjectile destructor body, whose code references ShipCombatProxiedProjectile::vftable), then f… |
| `00cabd60` | `ShipCombatProxiedProjectile::Dtor` | ✅ | dtor | Resets projectile state, releases its referenced object according to a flag, removes a tracked effect when present, and runs base cleanup. |
| `00cabeb0` | `ShipCombatProxiedProjectile::Ctor ⚠downgrade` | 🟢 | ctor | Calls a base initializer, installs the projectile vtable, stores source and owner references, and zeroes remaining state fields. |
| `00cabf50` | `ShipCombatProxiedProjectile::ScalarDeletingDtor` | ✅ | dtor | Calls projectile destructor and frees the object when the delete flag's low bit is set. |
| `00cac100` | `ShipCombatProxiedProjectile::Update` | ✅ | virtual | Updates transform from the referenced object, draws the cannonball shadow, and forwards a time or frame value to a helper. |
