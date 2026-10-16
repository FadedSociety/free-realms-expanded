# Class: SoeUtil::HashListMap<int,MiniMapImage,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0c400` | `MiniMapImageMap::Ctor` | ✅ | ctor | Installs the hash-list map vtable, zeros list metadata, clears 1024 bucket heads, and returns the map object. |
| `00b0c650` | `HashListMap::FreeNode → MiniMapImageMap::FreeNode` | ✅ | virtual | Frees the supplied node pointer with alignment 4. |
| `00b0cac0` | `MiniMapImageMap::Dtor` | ✅ | dtor | Restores the hash-list map vtable and removes head nodes until the map is empty. |
| `00b0cb80` | `MiniMapImageMap::ScalarDeletingDtor` | ✅ | dtor | Restores the map vtable, removes entries, and frees the container when the low flag bit is set. |
| `00b0cc00` | `MiniMapImageMap::Clear` | ✅ | dtor | Restores the map vtable and repeatedly removes its head element until empty. |
| `00b0ce80` | `MiniMapImageMap::Ctor` | ✅ | ctor | Initializes map metadata and clears 1024 bucket heads, then installs the HashList<MiniMapImage> vtable. |
