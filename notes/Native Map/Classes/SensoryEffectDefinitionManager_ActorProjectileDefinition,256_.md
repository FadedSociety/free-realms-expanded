# Class: SensoryEffectDefinitionManager<ActorProjectileDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090f420` | `ActorProjectileDefinitionManager::ActorProjectileDefinitionManager` | ✅ | ctor | Constructs the sensory-effect loader base, installs both manager vtables, initializes manager substructures through 009076d0 and 00907750, then returns this. |
| `0090f4d0` | `ActorProjectileDefinitionManager::ScalarDeletingDtorAdjustor` | ✅ | virtual | Subtracts four from this and jumps to 00910510, adapting the secondary manager-vtable pointer to its scalar deleting destructor. |
| `0090f4e0` | `ActorProjectileDefinitionManager::HasDefinitions` | ✅ | virtual | Returns whether the signed dword at this+0x3c is greater than zero. |
| `0090f4f0` | `ActorProjectileDefinitionManager::GetField34` | ✅ | virtual | Returns the dword stored at this+0x34. |
| `0090f500` | `ActorProjectileDefinitionManager::GetCount` | ✅ | virtual | Returns the dword stored at this+0x3c. |
| `0090f640` | `ActorProjectileDefinitionManager::GetProjectileField` | ✅ | virtual | Performs an RTTI-checked cast from SensoryEffectDefinition to ActorProjectileDefinition and returns the derived field at +0xb0. |
| `0090f800` | `ActorProjectileDefinitionManager::AddDefinition` | ✅ | virtual | Copies and hashes the definition name, inserts a hash-map entry, stores param_2 in that entry, frees the temporary object, then calls virtual slot 5 with the name. |
| `0090f870` | `ActorProjectileDefinitionManager::~ActorProjectileDefinitionManager` | ✅ | dtor | Restores manager and hash-map vtables, releases manager and hash-map contents through helpers, then destroys the sensory-effect loader base. |
| `00910510` | `ActorProjectileDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Calls 0090f870 to destroy the manager, frees it when flag bit 0 is set, and returns the object pointer. |
