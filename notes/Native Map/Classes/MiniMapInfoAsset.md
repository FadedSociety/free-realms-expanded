# Class: MiniMapInfoAsset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0c820` | `MiniMapInfoAsset::Ctor` | ✅ | ctor | Calls the asset base constructor, installs MiniMapInfoAsset's vtable, and zeros member index 10. |
| `00b0c880` | `MiniMapInfoAsset::Dtor` | ✅ | dtor | Restores the asset vtable, decrements and conditionally destroys the shared member, then calls base asset cleanup. |
| `00b0c900` | `MiniMapInfoAsset::OnLoaded` | ✅ | virtual | Drops an old shared reference, terminates nonempty input text, increments the new reference count, stores its pointer, sets a ready byte, and calls a completion callback. |
| `00b0ca30` | `MiniMapInfoAsset::ScalarDeletingDtor` | ✅ | dtor | Calls the asset cleanup and frees the object when the low flag bit is set; otherwise returns the pointer. |
