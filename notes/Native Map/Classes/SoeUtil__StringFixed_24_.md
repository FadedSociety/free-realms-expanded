# Class: SoeUtil::StringFixed<24>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008e0100` | `StringFixed24_Ctor` | ✅ | ctor | Calls the base string initializer and writes the StringFixed<24> vtable to the object. |
| `008e0160` | `StringFixed24_ScalarDeletingDtor` | ✅ | dtor | Calls fixed-string cleanup; when flag bit 0 is set, frees the object allocation, otherwise returns the object pointer. |
| `00b2c5b0` | `StringFixed24::Constructor` | ✅ | ctor | Initializes fixed 24-byte IString storage and writes the StringFixed<24> vtable. |
| `00b2d5f0` | `StringFixed24::Constructor` | ✅ | ctor | Initializes fixed 24-byte IString storage from the argument and writes the StringFixed<24> vtable. |
