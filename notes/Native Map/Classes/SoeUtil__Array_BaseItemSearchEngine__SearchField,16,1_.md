# Class: SoeUtil::Array<BaseItemSearchEngine::SearchField,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2e540` | `InlineArrayFreeBuffer` | ✅ | method | Aligns this+0x13 down to four bytes and frees the supplied pointer only when it is not the inline buffer. |
| `00c30670` | `SearchFieldInlineArrayDtor` | ✅ | dtor | Destroys elements, conditionally frees external storage, clears the data pointer, restores the zero-inline base vtable, and performs base cleanup. |
| `00c30740` | `SearchFieldInlineArrayResizeAllocator` | ✅ | method | Delegates requests above 16 to the base allocator; otherwise writes capacity 16 and returns the aligned inline buffer at this+0x13. |
| `00c308f0` | `SearchFieldInlineArrayCtor` | ✅ | ctor | Zeros the base array pointer/size/capacity, then installs the inline-capacity-16 specialization vtable. |
| `00c30950` | `SearchFieldInlineArrayScalarDeletingDtor` | ✅ | dtor | Calls inline SearchField array teardown, then frees this object when flag bit 0 is set. |
| `00c30970` | `— ⚠reject` | ⚪ | method | Copies source elements to temporary storage and clears the source, copies destination elements to a second temporary and clears it, then destroys both temporaries and buffers. |
