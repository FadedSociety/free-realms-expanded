# Class: DeepAssetHelper::ParticleDefinitionAsset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071bed0` | `ParticleDefinitionAsset::Constructor` | ✅ | ctor | Constructs Asset base and AssetRequester member, installs particle-asset vtables, initializes texture and string arrays and flags, and stores inputs. |
| `0071bfa0` | `ParticleDefinitionAsset::ScalarDeletingDtor` | ✅ | dtor | Calls 00d33ff0 particle-asset teardown and frees object when deleting flag bit is set. |
| `0071bfc0` | `ParticleDefinitionAssetAdjustedDeletingDtor` | ✅ | dtor | Subtracts 0x28 from ECX and jumps to 0071bfa0 scalar deleting destructor. |
