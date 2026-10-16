# Class: SoeUtil::List<ClientStartingZoneLoader::FileInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009cd8e0` | `FileInfoList::AllocateNode` | ✅ | virtual | Obtains a block from the current or global allocator, falling back to an aligned 0x30-byte allocation. |
| `009cd920` | `FileInfoList::FreeNode` | ✅ | virtual | Frees the supplied node through the memory manager with alignment 4. |
| `009ce060` | `FileInfoListDtor` | ✅ | dtor | Installs the list vtable and repeatedly destroys the head pointer at this+4 until the list is empty. |
| `009ce090` | `FileInfoListScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, drains the head nodes through FUN_009cdee0, and frees the object when the deleting flag is set. |
