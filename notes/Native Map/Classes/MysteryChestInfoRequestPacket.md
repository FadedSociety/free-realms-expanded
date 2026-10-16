# Class: MysteryChestInfoRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c6cf0` | `MysteryChestInfoRequestPacketCtor` | ✅ | ctor | Initializes MysteryChestBasePacket with subtype 1, then installs the MysteryChestInfoRequestPacket vtable. |
| `009c7460` | `MysteryChestInfoRequestScalarDeletingDtor` | ✅ | dtor | Restores BasePacket vtable and frees the allocation when the deleting flag is set; otherwise returns the original pointer. |
