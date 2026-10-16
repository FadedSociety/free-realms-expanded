# Class: GHBoxLayout

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00409b11` | `GetStaticPair50EC` | ✅ | function | When param_3 is non-null, stores param_1 through it, and stores the global dword DAT_01AF50EC through param_2. |
| `0043b8ed` | `GHBoxLayout::GHBoxLayout` | ✅ | ctor | Calls layout base constructor 00435c70 with the supplied argument, writes the GHBoxLayout vtable at object offsets +0 and +4, and returns this. |
| `0043b90c` | `GHBoxLayout::ForwardTo5017F8` | ✅ | method | The stack adapter forwards its receiver and two stack arguments to 005017f8, cleans the temporary 12-byte argument area, then returns with an 8-byte callee-pop; x86 bytes show a normal call wrapper, not a tail jump. |
| `0043b920` | `GHBoxLayout::AdjustorThunkToDestructor` | ✅ | method | Adjusts ECX by subtracting 4, then jumps directly to 0043bb36; it does not contain the target destructor body. |
| `0043bb36` | `GHBoxLayout::ScalarDeletingDestructor` | ✅ | dtor | Runs destructor body 0043591d; when flag bit 0 is set, frees the object through 004c6a61; otherwise returns the original object pointer. The x86 body has the scalar-deleting destructor shape. |
| `0043fbca` | `GHBoxLayout::CreateInstance` | 🟢 | function | Mode-selector factory for GHBoxLayout: mode 0 constructs in place (base initialiser FUN_0043B78D(0,0) plus the two GHBoxLayout vtable slots); mode 1 copy-constructs in place from param_3 (FUN_0043B8ED); mode 2 allocates… |
