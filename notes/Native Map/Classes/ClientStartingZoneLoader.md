# Class: ClientStartingZoneLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009cdf80` | `— ⚠reject` | ⚪ | dtor | When the nested count is positive, destroys its ClientLuaParam array and nested object, then nulls the object pointer and runs cleanup. |
| `009ce0d0` | `ClientStartingZoneLoader::Ctor` | ✅ | ctor | Stores the supplied pointer, installs ClientStartingZoneLoader and FileInfo-list vtables, clears fields, and initializes two helper objects. |
| `009ce180` | `ClientStartingZoneLoader::Dtor` | ✅ | dtor | Restores helper objects, drains the owned list at this+0x10, cleans both helper objects, then restores and drains the FileInfo list. |
