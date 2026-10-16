# Class: BaseRecipePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8a2b0` | `BaseRecipePacket::BaseRecipePacket` | ✅ | ctor | Sets BasePacket then BaseRecipePacket vtable, writes discriminator 0x3f, and stores the supplied packet type at field +8. |
| `00c8a8d0` | `BaseRecipePacket::ScalarDeletingDtor` | ✅ | dtor | Restores the BasePacket vtable and frees the object when deleting flag bit 0 is set. |
