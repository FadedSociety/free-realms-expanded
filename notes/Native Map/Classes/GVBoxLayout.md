# Class: GVBoxLayout

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00409aeb` | `GetStaticPair503C` | ✅ | function | When param_3 is non-null, stores param_1 through it, and stores the global dword DAT_01AF503C through param_2. |
| `0043b8b2` | `GVBoxLayout::GVBoxLayout` | ✅ | ctor | Calls layout base constructor 00435c70 with the supplied argument, writes the GVBoxLayout vtable at object offsets +0 and +4, and returns this. |
| `0043b8d1` | `GVBoxLayout::ForwardTo5017F8` | ✅ | method | The stack adapter forwards its receiver and two stack arguments to 005017f8, cleans the temporary 12-byte argument area, then returns with an 8-byte callee-pop; x86 bytes show a normal call wrapper, not a tail jump. |
| `0043b8e5` | `GVBoxLayout::AdjustorThunkToDestructor` | ✅ | method | Adjusts ECX by subtracting 4, then jumps directly to 0043baf9; it does not contain the target destructor body. |
| `0043baf9` | `GVBoxLayout::ScalarDeletingDestructor` | ✅ | dtor | Runs destructor body 0043591d; when flag bit 0 is set, frees the object through 004c6a61; otherwise returns the original object pointer. The x86 body has the scalar-deleting destructor shape. |
| `0043e765` | `GVBoxLayout::GVBoxLayout` | ✅ | ctor | Constructor: for a non-null object pointer runs the base initialiser FUN_0043B78D(1,0) and installs the GVBoxLayout vtable at slot [0] and the secondary vtable at [1]; returns the object pointer or null. |
| `0043fb2c` | `GVBoxLayout::CreateInstance` | 🟢 | function | Mode-selector factory for GVBoxLayout: mode 0 constructs in place (base initialiser FUN_0043B78D(1,0) plus the two GVBoxLayout vtable slots); mode 1 copy-constructs in place from param_3 (FUN_0043B8B2); mode 2 allocates… |
