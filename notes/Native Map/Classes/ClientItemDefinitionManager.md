# Class: ClientItemDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3d6e0` | `ClientItemDefinitionManager::ClientItemDefinitionManager` | ✅ | ctor | Constructor of the item-definition manager: installs ClientItemDefinitionManager::vftable, stores the owning client at +4, zeroes +8, initialises the member block at +0xc..+0x50 through 00a3b390, clears +0x58/+0x5c/+0x6… |
| `00a3d870` | `ClientItemDefinitionManager::FindDefinitionById` | ✅ | virtual | Uses id&0x3ff to select the bucket at this+0x64, walks node+0xb0 links, returns the node whose key at +0xb4 matches the id, or returns null when the chain ends. |
| `00a3d8a0` | `ClientItemDefinitionManager::~ClientItemDefinitionManager` | ✅ | dtor | Destructor body: installs the manager vtable, empties the definition cache with 00a3b530, deletes the lazy global object DAT_01be69c4 through its vtable slot 0 with argument 1, releases the four ref-counted handle slots… |
| `00a3ddd0` | `ItemDefinition_RequestById` | ✅ | method | Request-if-missing path for one definition id: it searches the cache chain at this+0x64 (key at +0xb4, chain +0xb0) and on a hit calls FUN_00c93c90; else it checks the pending-request tree through 00a3ac50 and logs "Ite… |
