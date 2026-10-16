# Class: MysteryChestInfoReplyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c8520` | `MysteryChestInfoReplyPacketDtor` | ✅ | dtor | Destroys the packet's embedded Mystery Chest info map, then restores the BasePacket vtable. |
| `009c8a30` | `MysteryChestInfoReplyPacketCtor` | ✅ | ctor | Initializes MysteryChestBasePacket with subtype 2, installs the reply packet vtable, and initializes its embedded Mystery Chest info map. |
| `009c8a90` | `MysteryChestInfoReplyScalarDeletingDtor` | ✅ | dtor | Destroys the embedded Mystery Chest info map, conditionally frees the packet allocation, and otherwise returns the object pointer. |
