# Class: SoeUtil::Array<SoeUtil::Array<float,1,1>,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0e530` | `Array::FreeBuffer → SoeUtil::Array<SoeUtil::Array<float,1,1>,1,1>::FreeBuffer` | ✅ | virtual | Frees the supplied buffer with alignment four unless it equals the aligned inline storage address. |
| `00b0e860` | `— ⚠reject` | ⚪ | dtor | Restores inline then dynamic array vtables, destroys live elements, and releases both dynamic and base buffers. |
| `00b0e930` | `NestedFloatArray::AllocateBuffer` | ✅ | virtual | Returns aligned inline storage for capacities through one, otherwise delegates to the dynamic array allocator. |
| `00b0ed20` | `NestedFloatArray::Ctor` | ✅ | ctor | Initializes dynamic nested-array fields and installs the one-element inline specialization vtable. |
| `00b0ed80` | `NestedFloatArray::ScalarDeletingDtor → InlineNestedFloatArray::ScalarDeletingDt…` | ✅ | dtor | Calls the nested-array destructor and frees the object when the low flag bit is set. |
