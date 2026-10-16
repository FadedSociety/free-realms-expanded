# Class: ActorParticleEmitterDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00914370` | `CtorWrap0090ea70 → ActorParticleEmitterDefinitionManager::Ctor` | 🟢 | ctor | thiscall (returns this in EAX). Calls FUN_0090ea70(arg2, arg3) with the two stack arguments passed through, then returns param_1. The class vtables ActorParticleEmitterDefinitionManager::vftable (01804f04, 01804eec) are… |
| `00914440` | `ThunkThisMinus4ToDeletingDtor` | 🟢 | virtual | Adjustor thunk (bytes 83 e9 04 e9 a8 05 00 00): subtracts 4 from ECX (this) and jumps to FUN_009149f0. It is vftable[0] of the second ActorParticleEmitterDefinitionManager vtable (01804eec). |
| `00914970` | `— ⚠reject` | ⚪ | virtual | Virtual at vftable[5] of ActorParticleEmitterDefinitionManager::vftable (01804f04). Stores arg into a local, then calls FUN_009148f0 with ECX=&local, stack arguments (this+0xc, &arg); returns nothing. |
| `009149a0` | `— ⚠reject` | ⚪ | dtor | __fastcall (this=ECX). Under an SEH frame: stores ActorParticleEmitterDefinitionManager::vftable (01804f04 variant) at [this+4]; then calls FUN_0090eeb0 (cleanup), and returns. Called by the scalar deleting destructor F… |
| `009149f0` | `ScalarDeletingDtor → ActorParticleEmitterDefinitionManagerScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 01804f04 (thiscall, one byte argument). Calls thiscall FUN_009149a0 (the destructor body) with ECX=this; then if (arg & 1) != 0 calls _free(this); returns this. |
