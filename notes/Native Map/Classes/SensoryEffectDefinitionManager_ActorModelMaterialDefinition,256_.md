# Class: SensoryEffectDefinitionManager<ActorModelMaterialDefinition,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0093c150` | `SensoryEffectDefinitionManager::Ctor → SensoryEffectDefinitionManager<ActorMode…` | ✅ | ctor | Constructs the sensory-effect loader, installs manager vtables, initializes the 256-bucket hash-list map, and initializes remaining manager state. |
| `0093c200` | `SensoryEffectDefinitionManager::ScalarDeletingDtorThunk → SensoryEffectDefiniti…` | ✅ | dtor | Adjusts ECX by four bytes, then jumps to 0093c630; this thunk does not contain the destructor body. |
| `0093c210` | `SensoryEffectDefinitionManager::HasDefinitions` | ✅ | virtual | Returns whether the signed count at object offset 0x3c is greater than zero. |
| `0093c220` | `SensoryEffectDefinitionManager::GetManagerField34` | 🟢 | virtual | Returns the dword stored at object offset 0x34. |
| `0093c230` | `SensoryEffectDefinitionManager::GetEntryCount` | ✅ | virtual | Returns the dword stored at object offset 0x3c. |
| `0093c370` | `GetNextActorModelMaterialDefinition` | 🟢 | virtual | RTTI-casts the supplied object to ActorModelMaterialDefinition through SensoryEffectDefinition and returns the dword at cast-result offset 0x60. |
| `0093c530` | `— ⚠reject` | ⚪ | virtual | Builds a name-map entry using 00705b00, inserts it through 0093c4a0, stores param_2 at returned-entry offset 0x10, releases the original record, then dispatches virtual slot +0x14 with param_2 and the source record +4 v… |
| `0093c5a0` | `SensoryEffectDefinitionManager::Dtor` | ✅ | dtor | Restores manager and hash-map vtables, releases manager-owned nodes, and destroys the sensory-effect loader subobject. |
| `0093c630` | `SensoryEffectDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Calls the manager destructor body and frees the object only when the deleting flag low bit is set. |
