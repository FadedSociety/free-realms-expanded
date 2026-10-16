# Class: ActorCameraEffectEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c1040` | `ActorCameraEffectEmitter::CompareFloat0xa8` | ✅ | virtual | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xa8 with DAT_01762a30 and returns a value whose low byte is 1 when equal, otherwise the comparison flags shif… |
| `007c1070` | `ActorCameraEffectEmitter::ReleaseA8` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d1b00 first if the byte argument is non-zero, then 007d21d0 with the dword argument, and clears this+0xa8. |
| `007c10b0` | `— ⚠reject` | ⚪ | dtor | Fastcall (ECX = object): sets the vtable to ActorCameraEffectEmitter::vftable; when the release predicate holds (byte at +0x26 zero, or the +0xa0 sub-object's +0xa8 float equals DAT_01762a30) and object+0xa8 is non-null… |
| `007c13c0` | `ActorCameraEffectEmitter::BuildObject` | ✅ | virtual | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero, stores the factory result uVar3 at this+0xa8, sets a temporary string from the C string at (+0xa0)+0x2c (00402ce0), and calls the vtable slot +0x28 o… |
