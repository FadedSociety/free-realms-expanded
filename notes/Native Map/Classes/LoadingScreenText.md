# Class: LoadingScreenText

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca91e0` | `LoadingScreenText::Ctor` | ✅ | ctor | Constructs a LoadingScreenText entry: calls the RefCounted base initialiser 0073b300, installs LoadingScreenText::vftable at +0, installs the IString vtable and the empty-string buffer in the embedded IString at +0x10, … |
| `00ca9250` | `LoadingScreenText::DtorBody ⚠downgrade` | 🟢 | dtor | Destructor body of LoadingScreenText: installs the IString vtable, releases the embedded string buffer when its count is positive (decrement, IString slot 2 at zero, or the 00401d30 free path), and finally restores the … |
| `00ca9630` | `LoadingScreenText::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of LoadingScreenText): calls the body 00ca9250 on this, then frees this through _free when bit 0 of the flag argument is set; returns this. |
