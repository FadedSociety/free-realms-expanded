# Class: MiniMapImages

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0cc70` | `MiniMapImages::Dtor` | ✅ | dtor | Restores MiniMapImages and IString vtables, releases the string reference, clears two hash lists, and detaches the asset requester. |
| `00b0cd30` | `MiniMapImages::RequestTileInfo ?` | 🟡 | function | Formats %s_TileInfo.txt from a key, stores the request parameter, and requests that asset through the requester interface. |
| `00b0cff0` | `MiniMapImages::Ctor` | ✅ | ctor | Sets the requester base, constructs two hash-list maps, initializes fixed slots and an IString member, then zeros counters. |
| `00b0d100` | `MiniMapImages::ScalarDeletingDtor` | ✅ | dtor | Calls the MiniMapImages destructor and conditionally frees the object. |
| `00b0d3e0` | `MiniMapImages::ParseTileInfo` | 🟢 | function | Reads records from a byte buffer, parses numeric fields, updates extents, hashes each record name, stores tile entries, then rebuilds an index. |
| `00b0d7a0` | `MiniMapImages::OnTileInfoLoaded` | 🟢 | virtual | Selects the loaded string buffer when present, passes it to the tile-info parser, and returns success. |
