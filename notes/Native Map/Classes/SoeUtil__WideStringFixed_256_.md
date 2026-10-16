# Class: SoeUtil::WideStringFixed<256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a75c10` | `SoeUtil::WideStringFixed::WideStringFixed` | ✅ | ctor | Constructor: sets up an SEH frame, calls base constructor 00a75920 with this in ECX (it installs the IString<unsigned_short> vtable, the static empty buffer DAT_01be73a0, zero count and the IStringFixed<unsigned_short,2… |
| `00a75c70` | `SoeUtil::WideStringFixed::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: sets up an SEH frame, calls base destructor 00a759d0 on this, then if bit 0 of the second argument is set calls _free (thunk to 00d8a9d8) on this, and returns this. |
| `00c2d290` | `WideStringFixed256Ctor` | ✅ | ctor | Derived constructor: constructs the IStringFixed<unsigned_short,256> base through 00c2d1b0 and then overwrites the vtable with SoeUtil::WideStringFixed<256>::vftable, returning this. |
| `00c2d2f0` | `— ⚠reject` | ⚪ | dtor | Destructor: calls the shared wide-string teardown 00a759d0 with a stack cookie and returns; the binary records no vtable store, consistent with a base-class-style destructor body. |
| `00c2d340` | `WideStringFixed256CopyCtor` | ✅ | ctor | Derived copy-style constructor: constructs the IStringFixed<unsigned_short,256> base through 00c2d220 and then overwrites the vtable with SoeUtil::WideStringFixed<256>::vftable, returning this. |
