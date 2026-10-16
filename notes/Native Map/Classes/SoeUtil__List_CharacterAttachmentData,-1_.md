# Class: SoeUtil::List<CharacterAttachmentData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007086e0` | `ListAlloc` | ✅ | virtual | Initializes the list allocator path, asks the current memory manager for an allocation, falls back to the global manager, then requests 0x1d0 bytes aligned to 4. |
| `00708720` | `ListFree → CharacterAttachmentListFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `00711f80` | `CharacterAttachmentDataListDestructor` | ✅ | dtor | Installs list vtable and removes list head repeatedly until empty. |
