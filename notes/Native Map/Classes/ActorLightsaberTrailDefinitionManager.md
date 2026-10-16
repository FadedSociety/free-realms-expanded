# Class: ActorLightsaberTrailDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00927900` | `ActorLightsaberTrailDefinitionManager::Ctor` | ✅ | ctor | Initializes inherited manager storage and installs both derived manager vtables. |
| `009279b0` | `ActorLightsaberTrailDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Adjusts this by four bytes and jumps to the secondary-base deleting-destructor path. |
| `00927cf0` | `CreateAndLinkActorLightsaberTrailDefinition` | 🟢 | method | Allocates and initializes a trail definition, then links it into manager list and hash storage. |
| `00927d70` | `ActorLightsaberTrailDefinitionManager::AddDefinition` | ✅ | virtual | Copies the key locally and forwards the source data to insertion routine 00927cf0. |
| `00927da0` | `ActorLightsaberTrailDefinitionManager::Dtor` | ✅ | dtor | Restores manager RTTI vtables and calls inherited cleanup routine 00924a70. |
| `00927df0` | `ActorLightsaberTrailDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Runs complete cleanup and frees the manager only when the deleting flag is set. |
