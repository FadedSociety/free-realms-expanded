# Class: ActorCompositeEffectEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c2ab0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1] (fastcall, ECX = object): calls the object's vtable slot +8 with (0, 1); reads 007cfe50 and 00769710 (+0x16c dword); calls 00769f90 on the sub-object at +0xa0 and 007df320 on it, stores the result at ob… |
| `007c2b20` | `ActorCompositeEffectEmitter::ReleaseA4` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa4 is non-null, calls 007dc400 first if the byte argument is non-zero, then 007dc450 with the dword argument, and clears this+0xa4. |
| `007c2b60` | `ActorCompositeEffectEmitter::~ActorCompositeEffectEmitter` | ✅ | dtor | Fastcall (ECX = object): sets the vtable to ActorCompositeEffectEmitter::vftable; computes a flag from the sub-object at +0xa0 (its vtable slot +0x10 result, or 0 when null); when object+0xa4 is non-null calls 007dc400 … |
