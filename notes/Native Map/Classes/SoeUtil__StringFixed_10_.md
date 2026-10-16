# Class: SoeUtil::StringFixed<10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a55ad0` | `SoeUtil::StringFixed10_Allocate` | ✅ | virtual | Returns inline storage at this+0x10 with capacity 0xe for requests below 0xf; larger requests use FUN_00401cb0. |
| `00a55b00` | `SoeUtil::StringFixed10_Free` | ✅ | virtual | Leaves the inline buffer at this+0x10 allocated in place and forwards other pointers to FUN_00401c00 with alignment zero. |
| `00a5c410` | `StringFixed10Ctor → StringFixed10CopyCtor` | ✅ | ctor | Constructs the string member with the fixed-string helper and installs the StringFixed<10> vtable. |
| `00a5c470` | `StringFixed10ScalarDeletingDtor` | ✅ | dtor | Calls the string cleanup helper and frees the object when flag bit 0 is set. |
| `00c68f90` | `SoeUtil::StringFixed10Ctor` | ✅ | ctor | Constructor of the owning derivation: runs the IStringFixed<char,10> base constructor FUN_00c68f30 under an SEH frame, then overwrites slot +0 with SoeUtil::StringFixed<10>::vftable and returns the same object pointer. |
