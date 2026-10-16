# Class: SoeUtil::List<LobbyGameOptionDisplayOrder,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01028d20` | `— ⚠reject` | ⚪ | virtual | Virtual allocate slot [2]: calls the lock helper 006fb9c0, then allocates through the thread-manager entry 006fb860 and its vslot +4, else DAT_01bdf280 vslot +4, else 006fb950 with size 0x10 and alignment 4. |
| `01028d60` | `ListNodeFreeSlot` | ✅ | virtual | Virtual free slot [3]: calls 00401c00 (FreeMemory) with the node pointer and alignment 4, then returns. |
| `01029610` | `ListDtorBody` | ✅ | dtor | Destructor body: stores the List<LobbyGameOptionDisplayOrder,-1> vtable at offset 0, then drains the list through 01029550. |
| `01029630` | `ListScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the List<LobbyGameOptionDisplayOrder,-1> vtable, drains the list through 01029550, then calls _free on this when bit 0 of the flag argument is set; returns this. |
