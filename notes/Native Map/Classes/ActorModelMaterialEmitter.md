# Class: ActorModelMaterialEmitter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007c0690` | `ActorModelMaterialEmitter::CallWith0xa0_0x40` | ✅ | virtual | Virtual slot [1] (fastcall, ECX = object, no stack args): calls 007cfe50; when it returns non-zero, reads the pointer at object+0xa0 and calls 0076f2d0 with the dwords at that pointer's +0x40 and +0x44, the address of i… |
| `007c06c0` | `ActorModelMaterialEmitter::CallWith0xa0_0x58` | ✅ | virtual | Virtual slot [2] (fastcall, ECX = object): calls 007cfe50 and, when it returns non-zero, calls 0076f390 with the dword at the sub-object (object+0xa0) +0x58. |
| `007c06f0` | `ActorModelMaterialEmitter::~ActorModelMaterialEmitter` | ✅ | dtor | Fastcall (ECX = object, no stack args): sets the vtable to ActorModelMaterialEmitter::vftable, calls 007cfe50 and, when non-zero, 0076f390 with the dword at (object+0xa0)+0x58; decrements the count at (object+0xa0)+8 wh… |
