# Class: BaseItemFilterManager<ClientItemDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb20c0` | `BaseItemFilterManagerDtor` | ✅ | dtor | Seven-byte destructor body (mov dword [ecx],0x018250cc; ret): writes BaseItemFilterManager<ClientItemDefinition>::vftable into this+0 and returns without touching anything else. Called only from the MSVC unwind funclets… |
| `00bb20d0` | `BaseItemFilterManagerScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 1: rewrites BaseItemFilterManager<ClientItemDefinition>::vftable into this+0 and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this … |
