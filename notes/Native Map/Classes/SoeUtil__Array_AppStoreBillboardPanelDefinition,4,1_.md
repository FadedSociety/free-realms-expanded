# Class: SoeUtil::Array<AppStoreBillboardPanelDefinition,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c36360` | `InlineArrayFree` | ✅ | virtual | Frees the supplied pointer with alignment 4 only when it differs from the object's aligned inline buffer address. |
| `00c37280` | `InlineBillboardPanelArrayDtor` | ✅ | dtor | Installs the inline-array vtable, destroys elements, frees dynamic storage unless it points at the inline buffer, then invokes base-array cleanup. |
| `00c37350` | `InlineArrayAllocatePanels` | ✅ | virtual | Returns the aligned inline buffer and capacity 4 when the request is at most four; otherwise jumps to 00c36230 for dynamic allocation. |
| `00c375b0` | `InlineBillboardPanelArrayScalarDtor` | ✅ | dtor | Calls the derived inline-array destructor and frees the object allocation when the low flag bit is set. |
| `00c37850` | `InlineBillboardPanelArrayCtor` | ✅ | ctor | Installs the base array vtable, clears pointer, length and capacity, then installs the four-element inline-array vtable. |
| `00c37f30` | `— ⚠reject` | ⚪ | virtual | Copies elements from a source array, clears the source and destination elements as required, then releases the dynamic backing block. |
