# Class: SoeUtil::HashList<ProxiedCharacter::EquippedAttachmentData,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b340` | `HashList::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b380` | `HashList::FreeNode` | ✅ | virtual | Passes the supplied node pointer and alignment 4 to FUN_00401c00. |
| `0096f9e0` | `EquippedAttachmentDataHashListCtor` | ✅ | ctor | Installs the typed hash-map vtable, zeros its map and list bookkeeping fields, then installs the HashList base vtable. |
| `0096fa50` | `EquippedAttachmentDataHashListScalarDeletingDtor` | ✅ | dtor | Calls the equipped-attachment hash-list destructor and frees object storage when the deleting flag's low bit is set. |
