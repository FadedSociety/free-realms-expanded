# Class: SoeUtil::Array<SensoryEffectDefinition*,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b0f0` | `InlinePointerArrayFreeStorage` | ✅ | virtual | Frees the supplied pointer at alignment 4 only when it is not this object's inline-buffer address. |
| `0095e390` | `Array::Destructor → SoeUtil::Array<SensoryEffectDefinition*,16,1>::Destructor` | ✅ | dtor | Installs the 16-element array vtable and clears the count at this+8. If the pointer at this+4 is not the aligned inline buffer, calls FUN_00401c00 with that pointer and alignment 4. It clears this+4, installs the zero-i… |
| `0095e450` | `InlineArrayAllocateStorage` | ✅ | virtual | Returns the aligned inline buffer with capacity 16 for requests up to 16 elements, otherwise calls FUN_0095afc0. |
| `00962730` | `ArraySensoryEffectDefinitionInline16Ctor` | ✅ | ctor | Installs the dynamic array vtable, zeros three bookkeeping words, then installs the inline-capacity-16 SensoryEffectDefinition pointer-array vtable. |
| `009627c0` | `ArraySensoryEffectDefinitionScalarDeletingDestructor` | ✅ | dtor | Calls FUN_0095e390 to destroy the inline-16 array; if flag bit 0 of param_2 is set, frees param_1; otherwise returns the original pointer. |
| `00964170` | `AssignAndResetSensoryEffectArray ? ⚠rename` | 🟡 | virtual | Passes param_2's buffer and positive count, or zero values, to FUN_0095f370; resets param_2's count; passes this array's buffer/count to the same helper; resets this count and invokes FUN_0095f370 with zero values. |
