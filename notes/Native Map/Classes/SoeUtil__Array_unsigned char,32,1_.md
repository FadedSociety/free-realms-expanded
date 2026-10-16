# Class: SoeUtil::Array<unsigned char,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007030b0` | `InlineArrayFreeMemory ?` | 🟡 | virtual | Frees array storage only when its data pointer is not the inline buffer at offset sixteen. |
| `00703230` | `InlineArrayDestructor ?` | 🟡 | dtor | Destroys the inline-capacity array and its base array subobject. |
| `007032b0` | `InlineArrayAllocate ?` | 🟡 | method | Returns the inline buffer for requests within capacity and delegates larger requests to heap allocation. |
| `00703560` | `InlineArrayScalarDeletingDestructor ?` | 🟡 | dtor | Calls the inline-array destructor and conditionally frees the enclosing object when the deleting bit is set. |
