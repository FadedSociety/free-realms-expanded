# Class: SoeUtil::Array<int,65536,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf27b0` | `Array65536FreeBuffer` | ✅ | virtual | Returns without freeing when the pointer equals the object's aligned inline-buffer address; otherwise frees it through 00401c00 with alignment 4. |
| `00cf2b00` | `— ⚠reject` | ⚪ | dtor | Restores the 65536-element array vtable, clears size, frees any non-inline buffer, switches to the zero-capacity base array vtable, clears size again and releases its remaining buffer. |
| `00cf2b90` | `Array65536Allocate` | ✅ | virtual | Rejects requested sizes greater than 0x10000 through 0073a540; otherwise stores capacity 0x10000 and returns the object's aligned inline buffer address. |
| `00cf2bc0` | `— ⚠reject` | ⚪ | ctor | Initializes the zero-capacity array vtable and clears four fields, then replaces the vtable with the 65536-capacity array vtable. |
| `00cf2c20` | `Array65536ScalarDeletingDtor` | ✅ | dtor | Calls the array destructor, frees this object when the low bit of the flag is set, and returns this otherwise. |
