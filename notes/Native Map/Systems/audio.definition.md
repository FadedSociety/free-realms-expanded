# System: audio.definition

28 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008da880` | `SoundControlDefinition::~SoundControlDefinition` | SoundControlDefinition | Destructor of the RTTI class SoundControlDefinition, which derives from ActorEffectDefinition; the call is reached without direct callers in this image, so it is invoked indirectly. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `0090c310` | `HashMapSoundPendingOneShot::Cleanup` | SoeUtil::HashMap<int,SoundOneShotDefini… | Destroys pending one-shot sound-definition lookup state. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090f900` | `SoundOneShotPendingMap::SoundOneShotPendingMap` | SoeUtil::HashMap<int,SoundOneShotDefini… | Initializes the pending one-shot definition lookup map. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `00910530` | `SoundOneShotPendingMap::ScalarDeletingDtor` | SoeUtil::HashMap<int,SoundOneShotDefini… | Implements scalar deletion for the pending one-shot map. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `00915640` | `ScalarDeletingDtor → SoundOneShotDefinitionLoaderScalarDeletingDtor` | SoundOneShotDefinitionLoader | Scalar deleting destructor of SoundOneShotDefinitionLoader. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00928060` | `SoundControlDefinitionManager::Ctor` | SoundControlDefinitionManager | Constructs the sound-control definition manager. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928110` | `SoundControlDefinitionManager::ScalarDeletingDtor` | SoundControlDefinitionManager | Provides a secondary-base deleting-destructor entry. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928290` | `SoundControlDefinition::Ctor` | SoundControlDefinition | Constructs a sound-control asset definition. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928460` | `SoundControlDefinitionManager::AddDefinition` | SoundControlDefinitionManager | Adds a parsed sound-control definition. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928480` | `SoundControlDefinitionManager::Dtor` | SoundControlDefinitionManager | Destroys manager storage and base state. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `009284d0` | `SoundControlDefinitionManager::ScalarDeletingDtor` | SoundControlDefinitionManager | Implements this manager's scalar deleting destructor. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00cebd80` | `ClientAreaAmbientSoundProperty::Enable` | ClientAreaAmbientSoundProperty | Enables this ambient sound property. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ced770` | `ClientAreaCombatMusicProperty::WriteFields` | ClientAreaCombatMusicProperty | Serializes combat-music definition data. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ced7c0` | `ClientAreaCombatMusicProperty::ReadSoundDefinitionId` | ClientAreaCombatMusicProperty | Loads the sound-definition identifier for combat music. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ced7f0` | `ClientAreaCombatMusicProperty::Start` | ClientAreaCombatMusicProperty | Starts or selects combat music for this client area. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009155a0` | `ThunkThisMinus8ToDeletingDtor` | SoundOneShotDefinitionLoader | Non-primary-base adjustor for the scalar deleting destructor of SoundOneShotDefinitionLoader. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `009155b0` | `DtorBody` | SoundOneShotDefinitionLoader | Destructor body of SoundOneShotDefinitionLoader, called by its scalar deleting destructor 00915640. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00928360` | `SoundControlDefinition::InitWithKey` | SoundControlDefinition | Initializes the sound-control definition key. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `009283e0` | `CreateAndLinkSoundControlDefinition` | SoundControlDefinitionManager | Adds one sound-control definition. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00cebc60` | `SetAmbientSoundDefinition` |  | Audio domain is supported by the same-function sound start/stop calls and its property caller. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cebd20` | `ClientAreaAmbientSoundProperty::Update ⚠downgrade` | ClientAreaAmbientSoundProperty | Virtual update hook for the ambient-sound property. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ced860` | `ClientAreaCombatMusicProperty::Stop ⚠downgrade` | ClientAreaCombatMusicProperty | Stops and releases this combat-music property. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00928310` | `LinkDefinitionAfterTail ?` |  | Links node after tail, updates empty-list head, inserts into the selected hash bucket, and increments count. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00ceba60` | `AmbientSoundStartIfEligible ? ⚠downgrade` |  | Runtime start logic is inferred from sound-definition lookup and the ambient-sound property callers. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cebba0` | `AmbientSoundStopIfEligible ? ⚠downgrade` |  | Role follows the sound property state and nearby validated runtime start routine. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00915510` | `— ⚠reject` | SoundOneShotDefinitionLoader | Constructor of SoundOneShotDefinitionLoader: builds the asset-loader base and the pending one-shot map, then installs its interface vtables. Called from PlayClient::Initialize (0092b170, accepted:confirmed). | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00c1c5a0` | `— ⚠reject` |  | Adds a definition to a SoundGroup-like client object; class attribution follows the constructor it calls. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00cebdc0` | `— ⚠reject` | ClientAreaAmbientSoundProperty | Disables and releases this ambient sound property. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

