# Class: SoeUtil::WideStringFixed<32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008e01c0` | `WideStringFixed32_Ctor` | ✅ | ctor | Calls the wide-string base initializer and writes the WideStringFixed<32> vtable to the destination. |
| `008e0220` | `WideStringFixed32_ScalarDeletingDtor` | ✅ | dtor | Calls wide fixed-string cleanup; if flag bit 0 is set, frees the object allocation, otherwise returns the object pointer. |
| `00cf3130` | `WideStringFixed32Ctor` | ✅ | ctor | Constructs the underlying fixed 32-character wide string and installs the WideStringFixed<32> vtable. |
