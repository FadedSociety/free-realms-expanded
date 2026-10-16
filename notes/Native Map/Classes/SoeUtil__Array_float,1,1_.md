# Class: SoeUtil::Array<float,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0eb30` | `FloatArray::Ctor` | ✅ | ctor | Initializes dynamic float-array metadata and installs the inline-capacity array vtable. |
| `00b0eb90` | `FloatArray::AllocateBuffer` | ✅ | virtual | Returns inline storage for one element or less, otherwise delegates to the dynamic allocator. |
| `00b0ebc0` | `FloatArray::FreeBuffer` | ✅ | virtual | Frees the buffer with alignment four unless it equals the array's aligned inline-storage address. |
| `00b0ebe0` | `FloatArray::Dtor` | ✅ | dtor | Restores inline and dynamic float-array vtables, resets size, and frees dynamic backing storage in both specializations. |
| `00b0ec70` | `FloatArray::ScalarDeletingDtor` | ✅ | dtor | Calls the float-array destructor and frees the object when the low flag bit is set. |
| `00b0ef70` | `— ⚠reject` | ⚪ | ctor | Initializes dynamic array base fields, installs the inline float-array vtable, and calls an allocator setup helper with the supplied argument. |
| `00b0f400` | `FloatArray::SwapOrRelease → FloatArray::Swap` | ✅ | virtual | Destroys helper objects, restores a dynamic-array vtable on a temporary, and releases its buffer with alignment four. |
