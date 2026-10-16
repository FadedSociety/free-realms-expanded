# Class: DirDBM

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `013c30c0` | `DirDBM::WriteBlobToFile` | 🟢 | virtual | Virtual slot [7] of DirDBM: takes the file name from this+4 (inline or heap pointer by capacity at +0x18), opens it with fopen mode wb (returns false when that fails), gets the data pointer from FUN_013c2260 and the len… |
| `013c39a0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [6] of DirDBM: first looks up the record through FUN_013c3860; when the lookup succeeds it sizes the buffer with FUN_013c2370 and copies the record bytes with _memcpy. Then, in all cases, it opens the file … |
| `013c55b0` | `DirDBM::~DirDBM` | 🟢 | dtor | Calls FUN_013c4fe0 (DestructorBody: restores DirDBM::vftable and releases embedded strings/buffer) and then, when flag bit 0 of param_2 is set, calls operator delete on the object and returns it. |
| `013c55d0` | `DirDBM::ScanDirectory` | 🟢 | method | Tests member string at +0xc with GetFileAttributesA for the directory attribute 0x10, copies it and appends member string at +0x28 (with DAT_018db808 copied as a separator) into a stack buffer, FindFirstFileA/FindNextFi… |
