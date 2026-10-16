# Class: SoeUtil::Array<unsigned_char,4096,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d334b0` | `Array4096::ReleaseExternalBuffer` | ✅ | virtual | Frees the supplied buffer with alignment one only when its pointer is not the array's inline buffer at this plus 0x10. |
| `00d334d0` | `Array4096::~Array4096` | ✅ | dtor | Restores the array vtables, clears its length and pointer fields, and frees non-inline backing storage. |
| `00d33520` | `Array4096::GetBuffer` | ✅ | virtual | For requested sizes up to 0x1000, sets capacity to 0x1000 and returns the inline buffer at this plus 0x10; larger sizes go to the allocation helper. |
| `00d33570` | `Array4096::ScalarDeletingDtor` | ✅ | dtor | Releases external storage, restores the empty-array vtable, frees the object when flag bit zero is set, and returns its pointer otherwise. |
| `00d33850` | `Array4096::SwapContents` | ✅ | virtual | Copies the second array contents to temporary storage, copies the first into the second, restores the saved contents into the first, and releases the temporary buffer. |
