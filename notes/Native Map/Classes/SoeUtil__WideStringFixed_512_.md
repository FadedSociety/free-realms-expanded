# Class: SoeUtil::WideStringFixed<512>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d00c00` | `WideStringFixed512AllocateStorage` | ✅ | virtual | For requests below 0x405, returns the inline buffer at offset 0x10 and reports capacity 0x404; larger requests delegate to the shared wide-string allocator. |
| `00d00c30` | `WideStringFixed512FreeExternalStorage` | ✅ | virtual | Frees the supplied buffer only when it differs from the object's inline buffer at offset 0x10, using alignment zero. |
| `00d00c50` | `WideStringFixed512Cleanup` | ✅ | dtor | Restores the fixed wide-string vtable, decrements the shared buffer reference count and frees it when ownership ends, resets fields to the static empty buffer, then restores the base IString vtable. |
| `00d01050` | `WideStringFixed512Initialize` | ✅ | ctor | Initializes the embedded wide IStringFixed base, then installs the WideStringFixed<512> vtable and returns the object pointer. |
| `00d010d0` | `WideStringFixed512ScalarDeletingDtor` | ✅ | dtor | Runs wide-string cleanup and frees the string object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |
