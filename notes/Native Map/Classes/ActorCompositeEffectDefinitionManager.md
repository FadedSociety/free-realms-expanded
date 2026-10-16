# Class: ActorCompositeEffectDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00940fb0` | `ActorCompositeEffectManagerCtor` | ✅ | ctor | Constructs the templated sensory-effect manager base, then installs the concrete manager vtable in both subobjects. |
| `00941070` | `ActorCompositeEffectManagerScalarDtor` | ✅ | dtor | Bytes subtract 4 from ECX and jump to 00941430, an adjustor thunk to shared scalar deletion. |
| `009413c0` | `— ⚠reject` | ⚪ | virtual | Copies the argument dword to a local stack slot, passes the copied value and original argument address to 00941310 with ECX=this+0x2c, and returns. |
| `009413e0` | `DestroyActorCompositeEffectManager` | ✅ | dtor | Restores the concrete manager vtables and calls 0093ff40 with ECX=this to destroy the templated manager state, then returns. |
| `00941430` | `ActorCompositeEffectManagerScalarDtor` | ✅ | dtor | Calls 009413e0 to destroy manager state, then frees the object only when the deleting flag is set. |
