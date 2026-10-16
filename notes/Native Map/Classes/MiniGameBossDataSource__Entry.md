# Class: MiniGameBossDataSource::Entry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c48370` | `MiniGameBossDataSourceEntry_Destruct` | 🟢 | dtor | Destructor of one 32-byte Entry: resets the embedded IString subobject's vtable at entry+8, and when the string has an allocated buffer (capacity at +0x14 > 0) it atomically decrements the refcount stored 4 bytes before… |
| `00c483e0` | `MiniGameBossDataSourceEntry_Construct` | 🟢 | ctor | Builds a default 32-byte Entry in place: clears +0 and +4, installs the IString vtable at +8 with the static empty-string data pointer (DAT_01bee044) at +0xc, sets length and capacity (+0x10/+0x14) to 0, copies the glob… |
| `00c48450` | `MiniGameBossDataSourceEntry_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor for an Entry: runs the Entry destructor 00c48370 on the object and, when bit 0 of the flag byte is set, frees the object with CRT _free; returns the object pointer. |
| `00c484c0` | `MiniGameBossDataSourceEntry_CopyConstruct` | 🟢 | ctor | Copy-constructs an Entry from another: copies the key dword and id (+0 and +4), installs the IString vtable at +8 with the static empty-string pointer DAT_01bee04c, copy-assigns the source's embedded string through the … |
