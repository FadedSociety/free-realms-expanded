# Class: SoeUtil::HashList<CharacterAttachmentData,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b3a0` | `HashList::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b3e0` | `HashList::FreeNode` | ✅ | virtual | Passes the supplied node pointer to FUN_00401c00 with alignment 4. |
| `0096e760` | `CharacterAttachmentDataHashListCtor` | ✅ | ctor | Installs HashListMap<int,CharacterAttachmentData,4,-1> then HashList<CharacterAttachmentData,4,-1> vtables and zeros the list/map bookkeeping fields. |
| `0096f2f0` | `CharacterAttachmentDataHashListScalarDeletingDtor` | ✅ | dtor | Calls the hash-list destructor and, when the deleting flag's low bit is set, frees the object storage. |
