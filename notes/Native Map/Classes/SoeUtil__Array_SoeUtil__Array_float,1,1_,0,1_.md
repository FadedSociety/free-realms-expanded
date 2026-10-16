# Class: SoeUtil::Array<SoeUtil::Array<float,1,1>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0e400` | `Array::AllocateBuffer` | ✅ | virtual | Returns existing storage when capacity suffices; otherwise computes a new capacity and allocates capacity times 0x18 bytes aligned to four. |
| `00b0e4a0` | `Array::FreeBuffer` | ✅ | virtual | Passes the supplied pointer to the shared memory free routine with alignment four. |
| `00b0e830` | `NestedFloatArray::Dtor → DynamicNestedFloatArray::Dtor` | ✅ | dtor | Restores the array vtable, destroys live elements, frees the dynamic buffer, and clears the pointer. |
| `00b0e960` | `NestedFloatArray::ScalarDeletingDtor → DynamicNestedFloatArray::ScalarDeletingD…` | ✅ | dtor | Restores the dynamic array vtable, destroys live elements, frees its buffer, and conditionally frees the object. |
| `00b128c0` | `NestedFloatArrayResetStorage ? ⚠rename` | 🟡 | virtual | Calls nested-array growth and element-destruction helpers on the source and this object, then clears temporary nested-array storage and releases its temporary allocation. |
