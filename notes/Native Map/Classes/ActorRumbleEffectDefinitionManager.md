# Class: ActorRumbleEffectDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009273f0` | `ActorRumbleEffectDefinitionManager::Ctor` | ✅ | ctor | Initializes inherited manager storage and installs the derived vtable in both subobjects. |
| `009274a0` | `ActorRumbleEffectDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Subtracts 4 from ECX, then jumps to 009278e0. |
| `009277e0` | `— ⚠reject` | ⚪ | method | Allocates a rumble-effect definition through a virtual slot, initializes it, then links it into list and hash storage. |
| `00927860` | `ActorRumbleEffectDefinitionManager::AddDefinition` | ✅ | virtual | Copies the input key and delegates to the manager insertion routine 009277e0. |
| `00927890` | `ActorRumbleEffectDefinitionManager::Dtor` | ✅ | dtor | Restores both RTTI vtables and invokes inherited cleanup routine 00924590. |
| `009278e0` | `ActorRumbleEffectDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Runs the complete destructor and frees the object only when the deleting flag is set. |
