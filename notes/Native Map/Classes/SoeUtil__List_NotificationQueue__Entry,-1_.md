# Class: SoeUtil::List<NotificationQueue::Entry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a95e40` | `ListDestructorBody` | ✅ | dtor | Installs the list vtable and unlinks the current first entry through 00a91410 until the head is empty. |
| `00a95e80` | `AllocateEntryStorage` | ✅ | virtual | Checks the active allocation manager, uses its virtual allocation slot when present, otherwise requests 0x30 bytes aligned to 4. |
| `00a95ec0` | `FreeEntryStorage` | ✅ | virtual | Calls 00401c00 with the supplied node pointer and alignment 4. |
| `00a95ee0` | `ListScalarDeletingDestructor` | ✅ | dtor | Sets the base-list vtable, drains nodes until empty, then frees this when flag bit 0 is set. |
