# Class: ActorParticleEmitterDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c04f0` | `ActorParticleEmitterDefinition::ScalarDeletingDestructor` | 🟢 | dtor | Calls 0x007e90b0 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00914710` | `Ctor → ActorParticleEmitterDefinition::Ctor` | ✅ | ctor | thiscall (this=arg1, arg2 on stack; returns this). Under an SEH frame: calls FUN_00914670() (ActorEffectDefinition constructor); stores AssetRequestInterface::AssetRequester vftable at [this+0x90] and arg2 at [this+0x94… |
