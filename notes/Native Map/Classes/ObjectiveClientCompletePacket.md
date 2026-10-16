# Class: ObjectiveClientCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cff7d0` | `ObjectiveClientCompletePacket::ObjectiveClientCompletePacket` | ✅ | ctor | Calls the objective packet base constructor with subopcode 7, installs the derived packet vtable, and stores the supplied objective identifier at object offset 0x0c. |
| `00cff890` | `ObjectiveClientCompletePacket::ScalarDeletingDtor` | ✅ | dtor | Restores the BasePacket vtable and frees the packet object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |
