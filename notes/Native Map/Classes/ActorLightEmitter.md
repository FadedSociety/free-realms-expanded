# Class: ActorLightEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c0910` | `ActorLightEmitter::CompareFloat0xbc` | ✅ | virtual | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xbc with DAT_01762a30; returns a value whose low byte is 1 when they are equal, otherwise the comparison flag… |
| `007c0940` | `ActorLightEmitter::ReleaseA8` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d01f0 first if the byte argument is non-zero, then calls 007d06b0 with the dword argument, and clears this+0xa8. |
| `007c09d0` | `— ⚠reject` | ⚪ | dtor | Fastcall (ECX = object, no stack args): sets the vtable to ActorLightEmitter::vftable; when the object+0xa0 release is due (byte at +0x26 zero, or the +0xa0 sub-object's +0xbc float equals DAT_01762a30) and object+0xa8 … |
| `007c0ce0` | `ActorLightEmitter::BuildLightObject` | 🟢 | virtual | Virtual slot [1] (fastcall, ECX = object, no stack args). When 007cfe50 is non-zero it computes four floats from the sub-object at +0xa0 (+0xa0..0xac scaled by +0xb4), calls 00769d00 and 007d1950 with them and stores th… |
