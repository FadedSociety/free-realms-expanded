# Class: SensoryEffectDefinitionManager<ActorMeshEmitterDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090ef40` | `ActorMeshEmitterDefinitionManager::ActorMeshEmitterDefinitionManager` | ✅ | ctor | Constructs the sensory-effect loader base, installs both manager vtables, initializes manager substructures through 009074d0 and 00907550, then returns this. |
| `0090eff0` | `ActorMeshEmitterDefinitionManager::ScalarDeletingDtorAdjustor` | ✅ | virtual | Subtracts four from this and jumps to 009104f0, adapting the secondary manager-vtable pointer to its scalar deleting destructor. |
| `0090f000` | `ActorMeshEmitterDefinitionManager::HasDefinitions` | ✅ | virtual | Returns whether the signed dword at this+0x3c is greater than zero. |
| `0090f010` | `ActorMeshEmitterDefinitionManager::GetField34` | ✅ | virtual | Returns the dword stored at this+0x34. |
| `0090f020` | `ActorMeshEmitterDefinitionManager::GetCount` | ✅ | virtual | Returns the dword stored at this+0x3c. |
| `0090f160` | `ActorMeshEmitterDefinitionManager::GetEmitterField` | ✅ | virtual | Performs an RTTI-checked cast from SensoryEffectDefinition to ActorMeshEmitterDefinition and returns the derived field at +0xc0. |
| `0090f320` | `ActorMeshEmitterDefinitionManager::AddDefinition` | ✅ | virtual | Copies and hashes the definition name, inserts a hash-map entry, stores param_2 in that entry, frees the temporary object, and calls virtual slot 5 with the name. |
| `0090f390` | `ActorMeshEmitterDefinitionManager::~ActorMeshEmitterDefinitionManager` | ✅ | dtor | Restores manager and hash-map vtables, releases manager and hash-map contents through helpers, then destroys the sensory-effect loader base. |
| `009104f0` | `ActorMeshEmitterDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Calls 0090f390 to destroy the manager, frees it when flag bit 0 is set, and returns the object pointer. |
