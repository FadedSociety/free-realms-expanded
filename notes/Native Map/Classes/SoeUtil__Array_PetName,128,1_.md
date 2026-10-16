# Class: SoeUtil::Array<PetName,128,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b57a50` | `SoeUtil::Array_PetName_Inline128_Dtor` | ✅ | dtor | Destroys the 128-inline PetName array specialization, frees only non-inline storage, then restores and destroys the zero-inline array base. |
| `00b57b20` | `SoeUtil::Array_PetName_Inline128_Allocate` | ✅ | virtual | Returns the inline buffer and capacity 0x80 for requests up to 128 elements; delegates larger requests to the 36-byte array growth helper. |
| `00b59190` | `SoeUtil::Array_PetName_Inline128_Ctor` | ✅ | ctor | Initializes the zero-inline array fields, then installs the inline-128 PetName array vtable. |
| `00b591f0` | `SoeUtil::Array_PetName_Inline128_ScalarDeletingDtor` | ✅ | dtor | Calls the inline-128 array destructor and frees the object only when param_2 bit 0 is set. |
| `00b59320` | `— ⚠reject` | ⚪ | virtual | Copy-constructs source records into temporary storage, destroys both arrays' current elements, then calls the deallocation routine for temporary storage. |
