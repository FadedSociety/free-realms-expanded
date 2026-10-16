# Class: ImageDirectory

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987f20` | `ImageDirectory_ProcessRequest ? ⚠rename` | 🟡 | virtual | When param_3 is nonnull and its length field at +8 is nonzero, calls 0083e860 with param_1, param_2, the data pointer at +4 and that length. |
| `00987f60` | `ImageDirectory_FindImageId` | 🟢 | function | Searches three hash-table regions for param_2 using masks 0x3fff, 0x3ff and 0x3ff; on a matching key returns the first word of the value slot, or returns zero if no table has a match. |
| `0098d860` | `ImageDirectory::Destructor` | ✅ | dtor | Installs the ImageDirectory vtable, calls shared service helpers, clears nested-map and string/integer caches, and clears pending image-load and portrait maps. |
| `0098dab0` | `ImageDirectory::Constructor` | ✅ | ctor | Installs the ImageDirectory vtable, clears manager state, initializes pending-load and portrait maps plus integer and string lookup maps, sets a timestamp-like field to 1500000000, and registers the directory with share… |
