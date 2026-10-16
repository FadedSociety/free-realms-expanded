# Class: MysteryChestBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c6bd0` | `MysteryChestBasePacketCtor` | ✅ | ctor | Installs BasePacket then MysteryChestBasePacket vtables, sets the base opcode field to 0xca, and clears the subtype field. |
| `009c6c30` | `MysteryChestBasePacketInitSubtype` | ✅ | ctor | Installs BasePacket and MysteryChestBasePacket vtables, stores opcode 0xca, and writes the supplied subtype field. |
| `009c7400` | `MysteryChestBasePacketScalarDeletingDtor` | ✅ | dtor | Restores BasePacket vtable and frees the object only when the deleting flag is set, otherwise returns the pointer. |
