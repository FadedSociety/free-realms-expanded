# Class: ActorParticleEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c2dc0` | `ActorParticleEmitter::ReleaseA4` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa4 is non-null, calls 0086a8f0 first if the byte argument is non-zero, then calls the vtable slot +0xc of the object at +0xa4 with the dword argument, and … |
| `007c2ee0` | `— ⚠reject` | ⚪ | dtor | Fastcall (ECX = object): sets the vtable to ActorParticleEmitter::vftable and tail-jumps to _free on the pointer at object+0xb8. |
| `007c2fe0` | `ActorParticleEmitter::BuildObject` | 🟢 | virtual | Virtual slot [1] (fastcall, ECX = object, 602 bytes): calls 007cfe50 and, on the build path, uses 00705bc0, 00729610, 007660b0, 00769710, 00769cb0, 0076f5b0, 0076f5e0, 0076f9b0, 007707f0, 0079f4d0, 0079f4f0, 0079fab0, 0… |
