# Class: SensoryEffectDefinitionManager<ActorParticleEmitterDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090ea70` | `ActorParticleEmitterDefinitionManager::ActorParticleEmitterDefinitionManager` | ✅ | ctor | Constructs the sensory-effect loader base, installs both manager vtables, initializes two manager substructures through 00907150 and 009071d0, then returns this. |
| `0090eb10` | `ActorParticleEmitterDefinitionManager::ScalarDeletingDtorAdjustor` | ✅ | virtual | Subtracts four from this and jumps to 009104d0, adapting the secondary manager-vtable pointer to its scalar deleting destructor. |
| `0090eb20` | `ActorParticleEmitterDefinitionManager::HasDefinitions` | ✅ | virtual | Returns whether the signed dword at this+0x3c is greater than zero. |
| `0090eb30` | `ActorParticleEmitterDefinitionManager::GetField34` | ✅ | virtual | Returns the dword stored at this+0x34. |
| `0090eb40` | `ActorParticleEmitterDefinitionManager::GetCount` | ✅ | virtual | Returns the dword stored at this+0x3c. |
| `0090ec70` | `ActorParticleEmitterDefinitionManager::GetEmitterField` | ✅ | virtual | Performs an RTTI-checked cast from SensoryEffectDefinition to ActorParticleEmitterDefinition and returns the derived field at +0xd0. |
| `0090ee40` | `ActorParticleEmitterDefinitionManager::AddDefinition` | ✅ | virtual | Copies and hashes the definition name, inserts a name-map entry, stores param_2 in the new entry, frees the temporary object, then calls virtual slot 5 with the name. |
| `0090eeb0` | `ActorParticleEmitterDefinitionManager::~ActorParticleEmitterDefinitionManager` | ✅ | dtor | Restores manager and hash-map vtables, releases manager and hash-map contents through helpers, then destroys the sensory-effect loader base. |
| `009104d0` | `ActorParticleEmitterDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Calls 0090eeb0 to destroy the manager, frees it when flag bit 0 is set, and returns the object pointer. |
