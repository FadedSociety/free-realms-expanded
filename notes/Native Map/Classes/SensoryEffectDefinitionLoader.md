# Class: SensoryEffectDefinitionLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cf920` | `SensoryEffectDefinitionLoader::SensoryEffectDefinitionLoader` | ✅ | ctor | Constructor of the sensory-effect definition loader: installs the two interface vtables (SensoryEffectDefinitionManagerInterface then AssetManagement::AssetDeliveryHandlerInterface), stores param_2 at +8 and again at +0… |
| `008cf9d0` | `SensoryEffectDefinitionLoader::~SensoryEffectDefinitionLoader` | ✅ | dtor | Destructor of the sensory-effect definition loader: rewrites both loader interface vtables, releases the embedded SoeUtil::IString at +0x14 (capacity +0x20) through the locked refcount, runs the member cleanup FUN_01093… |
| `008cfa70` | `SensoryEffectDefinitionLoader::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the loader destructor body FUN_008cf9d0 and frees the object with _free when bit 0 of the flag argument is set. |
