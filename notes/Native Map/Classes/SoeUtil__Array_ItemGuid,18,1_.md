# Class: SoeUtil::Array<ItemGuid,18,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb0d70` | `ArrayInlineItemGuidFree` | ✅ | virtual | Compares the supplied pointer with this array's aligned inline storage address and frees it with alignment 4 only when it is not inline storage. |
| `00cb0de0` | `ArrayItemGuidDtor` | ✅ | dtor | Restores the inline-array vtable, clears its count, frees heap storage when present, then resets the base array vtable and fields. |
| `00cb0e70` | `ArrayItemGuidAllocate` | ✅ | virtual | Uses inline storage for requests up to 18 elements and returns its aligned address; larger requests call the allocation helper. |
| `00cb11d0` | `ArrayItemGuidInlineCtor` | ✅ | ctor | Initializes the zero-capacity ItemGuid array fields, then installs the 18-item inline specialization vtable. |
| `00cb1230` | `ArrayItemGuidScalarDeletingDtor` | ✅ | dtor | Calls the ItemGuid array destructor and frees the object when the deleting flag's low bit is set; otherwise returns the object pointer. |
| `00cb1250` | `— ⚠reject` | ⚪ | virtual | Clears elements in both supplied array objects through the element cleanup helper, zeros their counts, and releases the temporary empty allocation. |
