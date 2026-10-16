# Class: ActorCameraEffectDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00926ee0` | `ActorCameraEffectDefinitionManager::Ctor` | ✅ | ctor | Initializes base manager storage and installs the derived manager vtable in both subobjects. |
| `00926f90` | `ActorCameraEffectDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Adjusts this by four bytes and jumps to its secondary-base deleting-destructor path. |
| `009272d0` | `CreateAndLinkActorCameraEffectDefinition` | 🟢 | method | Allocates a camera-effect definition through a virtual slot, initializes it, then links it into manager list and hash storage. |
| `00927350` | `ActorCameraEffectDefinitionManager::AddDefinition` | ✅ | virtual | Copies the supplied key and delegates definition insertion to 009272d0. |
| `00927380` | `ActorCameraEffectDefinitionManager::Dtor` | ✅ | dtor | Restores manager RTTI vtables and invokes inherited cleanup routine 009240b0. |
| `009273d0` | `ActorCameraEffectDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Runs full cleanup and frees the object only when the deleting flag is set. |
