# Class: ActorLightEmitterDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009269c0` | `ActorLightEmitterDefinitionManager::Ctor` | ✅ | ctor | Initializes inherited manager storage and installs the derived vtable in both manager subobjects. |
| `00926a70` | `ActorLightEmitterDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Applies a four-byte this adjustment and jumps through the secondary-base deleting-destructor path. |
| `00926dc0` | `CreateAndLinkActorLightEmitterDefinition` | 🟢 | method | Allocates a definition through a virtual slot, initializes it on success, and links it into list and hash storage. |
| `00926e40` | `ActorLightEmitterDefinitionManager::AddDefinition` | ✅ | virtual | Copies the supplied key and forwards it with source data to 00926dc0. |
| `00926e70` | `ActorLightEmitterDefinitionManager::Dtor` | ✅ | dtor | Restores manager vtables and invokes inherited cleanup routine 00923bd0. |
| `00926ec0` | `ActorLightEmitterDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Runs the complete destructor and frees the object only when the deleting flag is set. |
