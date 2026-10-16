# Class: ActorModelMaterialDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0093cc80` | `ActorModelMaterialDefinitionManager::Ctor` | ✅ | ctor | Constructs the sensory-effect manager base, then installs derived manager vtables in both subobjects. |
| `0093cd40` | `ActorModelMaterialDefinitionManager::ScalarDeletingDtorThunk` | ✅ | dtor | Adjusts ECX by four bytes, then jumps to 0093d160; this thunk does not contain the destructor body. |
| `0093d0e0` | `ActorModelMaterialDefinitionManager::AddDefinition` | ✅ | virtual | Copies the supplied identifier into a temporary key and delegates insertion to the definition-node routine. |
| `0093d110` | `ActorModelMaterialDefinitionManager::DtorBody` | ✅ | dtor | Restores derived manager vtables and calls the sensory-effect manager destructor body to release owned maps and loader state. |
| `0093d160` | `ActorModelMaterialDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Calls the derived manager destructor body and frees the object only when the deleting flag low bit is set. |
