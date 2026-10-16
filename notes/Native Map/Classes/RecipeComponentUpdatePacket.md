# Class: RecipeComponentUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8a430` | `RecipeComponentUpdatePacket::RecipeComponentUpdatePacket` | ✅ | ctor | Initializes BaseRecipePacket with type 2, stores values at +0xc and +0x10, and copies an optional dword to +0x14. |
| `00c8a990` | `RecipeComponentUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Restores the BasePacket vtable and frees the object when deleting flag bit 0 is set. |
