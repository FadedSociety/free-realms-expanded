# Class: ActorRumbleEffectEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c16d0` | `ActorRumbleEffectEmitter::CompareFloat0xac` | ✅ | virtual | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xac with DAT_01762a30 and returns a value whose low byte is 1 when equal, otherwise the comparison flags shif… |
| `007c1700` | `ActorRumbleEffectEmitter::ReleaseA8` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d30a0 first if the byte argument is non-zero, then 007d30b0 with the dword argument, and clears this+0xa8. |
| `007c1740` | `— ⚠reject` | ⚪ | dtor | Fastcall (ECX = object): sets the vtable to ActorRumbleEffectEmitter::vftable; when the release predicate holds (byte at +0x26 zero, or the +0xa0 sub-object's +0xac float equals DAT_01762a30) and object+0xa8 is non-null… |
| `007c1a50` | `ActorRumbleEffectEmitter::BuildObject` | ✅ | virtual | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero, stores the factory result uVar3 at this+0xa8, sets a temporary string from (+0xa0)+0x2c (00402ce0) and calls the vtable slot +0x28 of the object poin… |
