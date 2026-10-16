# Class: SoundControlDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00928060` | `SoundControlDefinitionManager::Ctor` | ✅ | ctor | Initializes base manager storage and installs the derived manager vtable into both subobjects. |
| `00928110` | `SoundControlDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Adjusts this by four bytes and jumps to the secondary-base deleting-destructor path. |
| `009283e0` | `CreateAndLinkSoundControlDefinition` | 🟢 | method | Allocates the definition through a virtual slot, initializes it on success, and links it into list and hash storage. |
| `00928460` | `SoundControlDefinitionManager::AddDefinition` | ✅ | virtual | Copies the supplied key locally and delegates insertion to 009283e0. |
| `00928480` | `SoundControlDefinitionManager::Dtor` | ✅ | dtor | Restores manager RTTI vtables, then invokes inherited cleanup routine 00924fb0. |
| `009284d0` | `SoundControlDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Runs full manager cleanup and frees the object only when the deleting flag is set. |
