# Class: RailDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00928c20` | `RailDefinitionManager::Ctor` | ✅ | ctor | Installs BaseDefinitionManager then RailDefinitionManager vtables and initializes inherited storage through 00925040. |
| `00928c80` | `RailDefinitionManager::Dtor` | ✅ | dtor | Destroys the rail hash/list map through 0091fe30, then restores the BaseDefinitionManager vtable. |
| `00928ce0` | `RailDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Runs full manager cleanup and frees through 00401c00 when the deleting flag is set. |
| `0104a2b0` | `— ⚠reject` | ⚪ | virtual | Thiscall, one stack arg (path prefix string). Loads Rails.txt through the file reader (00fbf4a0, 00fbf4a0 open) line by line, skipping '#' rows: parses the rail id cell with 01071270 (separator 0x5e) and inserts a node … |
