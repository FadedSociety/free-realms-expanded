# Class: ParticleDefinitionAsset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d33d60` | `ParticleDefinitionAsset::CreateTextureAsset` | ✅ | virtual | Allocates 0x34 bytes, constructs a TextureAsset in that block, and returns the constructor result. |
| `00d33ff0` | `ParticleDefinitionAsset::~ParticleDefinitionAsset` | ✅ | dtor | Releases the requester and ref-counted entry state, cleans the object's arrays, then calls the base Asset destructor. |
| `00d341e0` | `ParticleDefinitionAsset::OnAssetUnavailable` | ✅ | virtual | Sets the unavailable state, logs the missing particle asset to ParticleLoadErrors.log, and invokes the optional requester callback. |
| `00d342a0` | `ParticleDefinitionAsset::CheckParseAndNotify` | ✅ | method | Checks parse and load-completion state, logs parse failure to ParticleDefinitions.log, and notifies the requester when the asset is ready. |
| `00d343d0` | `ParticleDefinitionAsset::AddDefinitionEntry` | ✅ | virtual | Appends and retains a child definition object, updates the entry count, and then checks parse completion and requester notification. |
