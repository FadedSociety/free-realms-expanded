# Class: SoeUtil::Array<float,6,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d3190` | `InlineFloatArrayFree` | ✅ | virtual | Frees the supplied pointer with four-byte alignment unless it is the array's inline buffer address rounded up to four-byte alignment. |
| `009d37e0` | `InlineFloatArray::Dtor` | ✅ | dtor | Restores the six-inline-float array vtable, clears its count, frees non-inline storage, then restores and clears the base dynamic array. |
| `009d3870` | `InlineFloatArray::Allocate` | ✅ | virtual | Reports inline capacity six and returns the array's inline storage address aligned to four bytes. |
| `009d3ea0` | `InlineFloatArrayCtor` | ✅ | ctor | Initializes the base float array with empty pointer, count, and capacity, then installs the six-inline-float specialization vtable. |
| `009d3f00` | `InlineFloatArray::ScalarDeletingDtor` | ✅ | dtor | Runs the inline float-array destructor and frees the object when the deleting flag is set. |
