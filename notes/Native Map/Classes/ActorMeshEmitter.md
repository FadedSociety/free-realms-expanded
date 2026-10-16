# Class: ActorMeshEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c1d40` | `ActorMeshEmitter::CompareFloat0xac` | ✅ | virtual | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xac with DAT_01762a30 and returns a value whose low byte is 1 when equal, otherwise the comparison flags shif… |
| `007c1d70` | `ActorMeshEmitter::ReleaseA8` | ✅ | virtual | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007ab880 first if the byte argument is non-zero, then 007ac200 with the dword argument, and clears this+0xa8. |
| `007c1dc0` | `ActorMeshEmitter::CallSlot3IsZero` | ✅ | virtual | Virtual slot [4] (fastcall, ECX = object, no stack args): calls the object's vtable slot [3] (+0xc) and returns 1 when that call's byte result is zero, else 0. |
| `007c1de0` | `— ⚠reject` | ⚪ | dtor | Fastcall (ECX = object, no stack args): sets the vtable to ActorMeshEmitter::vftable; when the release predicate holds (byte at +0x26 zero, or the +0xa0 sub-object's +0xac float equals DAT_01762a30) and object+0xa8 is n… |
| `007c2160` | `ActorMeshEmitter::BuildObjectIfAbsent` | ✅ | virtual | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero and this+0xa8 is null, stores uVar5 at this+0xa8, sets a temporary string from (+0xa0)+0x2c (00402ce0), calls 007660b0 with this+0xa8, the 16-dword bl… |
