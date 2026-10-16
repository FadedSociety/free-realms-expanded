# Class: ActorLightsaberTrailEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c2470` | `ActorLightsaberTrailEmitter::ReleaseA8` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d3da0 first if the byte argument is non-zero, then 007d4580 with the dword argument, and clears this+0xa8. |
| `007c24f0` | `ActorLightsaberTrailEmitter::~ActorLightsaberTrailEmitter` | ✅ | dtor | Fastcall (ECX = object): sets the vtable to ActorLightsaberTrailEmitter::vftable; when object+0xa8 is non-null calls 007d3da0 and then 007d4580(0) and clears +0xa8; decrements the count at (object+0xa0)+8 when positive;… |
| `007c27c0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero, copies the dword at (+0xa0)+0xa8 into this+0xa8, then calls the vtable slot +0x28 of the object pointed to by (this+0xa8)+8, and continues with furth… |
