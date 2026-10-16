# Class: SoeUtil::List<CharacterAttachmentData,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad5150` | `— ⚠reject` | ⚪ | ctor | Installs the 16-inline-node list vtable, clears its fields, then initializes the embedded node sentinel structure. |
| `00ad5b80` | `ListPopFrontVirtual → SoeUtil::List<CharacterAttachmentData,16>::PopFront` | ✅ | virtual | Forwards to 00ad5aa0 to remove and return a node from the list's front. |
| `00ad5b90` | `ListPushFrontVirtual` | ✅ | virtual | Forwards to 00ad5b10 to insert a node at the list's front. |
| `00ad5ba0` | `CharacterAttachmentListDtor` | ✅ | dtor | Destroys all active list nodes, frees pooled backing storage and clears list fields, then repeats base-list cleanup for the dynamic-capacity base. |
| `00ad5c70` | `CharacterAttachmentListScalarDeletingDtor` | ✅ | dtor | Calls the list destructor and frees the list allocation when the deleting-destructor flag bit is set. |
