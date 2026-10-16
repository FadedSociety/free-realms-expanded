# Class: RecipeUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8cc00` | `RecipeUpdatePacket::Dtor` | ✅ | dtor | Destroys embedded recipe map state, restores the BasePacket vtable, and returns. |
| `00c8cf40` | `RecipeUpdatePacket::RecipeUpdatePacket` | ✅ | ctor | Initializes BaseRecipePacket with type 5, installs RecipeUpdatePacket vtable, and constructs its embedded hash list. |
| `00c8cfa0` | `RecipeUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor and frees the object when deleting flag bit 0 is set. |
