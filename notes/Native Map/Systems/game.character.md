# System: game.character

33 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00712340` | `CharacterAttachmentRecord_DtorBody` |  | Destructor body of the CharacterAttachment record; reached from EH unwind funclets and from 00713bf0. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `008b2c80` | `CharacterVariables::CharacterVariables` | CharacterVariables | Default constructor of a character variable block: a vtable plus a 0x40-byte table of per-character values, built by its three callers 0091cac0, 0096fa70 and 01042bd0. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b2cf0` | `CharacterVariables::~CharacterVariables` | CharacterVariables | Non-deleting destructor of the CharacterVariables block, invoked from exception-handling unwind paths when a partially built character-variable object must be rolled back. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b2d40` | `CharacterVariables::ScalarDeletingDestructor` | CharacterVariables | Deleting-destructor half of the CharacterVariables pair (the vtable-only half is 008b2cf0), used when a character variable block is destroyed and its storage released. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008c6d30` | `LocalPlayerLoadFailed` |  | Failure path of local player actor loading: clears the loaded-actor field and reports the message through the object's log slot. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `00963410` | `ProxiedCharacter::OnProfileSwapComplete` | ProxiedCharacter | ProxiedCharacter virtual slot 3 handles a profile-swap completion event; its later action remains unidentified. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00bfb4d0` | `PlayerSwitchProfileEvent::PlayerSwitchProfileEvent` | PlayerSwitchProfileEvent | Initializes the event object created by the profile-swap processor. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfb550` | `PlayerSwitchProfileEvent::~PlayerSwitchProfileEvent` | PlayerSwitchProfileEvent | Tears down base event state when a PlayerSwitchProfileEvent is destroyed. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfb5c0` | `PlayerSwitchProfileEvent::GetFlag50` | PlayerSwitchProfileEvent | Reads the event's flag byte. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfb5e0` | `PlayerSwitchProfileEvent::SetFlag50WhenBit0` | PlayerSwitchProfileEvent | Sets the event flag when the caller supplies bit 0. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfb600` | `PlayerSwitchProfileEvent::DeleteWhenFlagSet` | PlayerSwitchProfileEvent | Conditional deleting wrapper for PlayerSwitchProfileEvent. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfb620` | `ProcessProfileSwap` |  | Runs the local profile-swap flow after checking that required equipment slots have matching items. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfcfe0` | `CharacterDataSource::GetAboutMeValue` | CharacterDataSource | Provides character profile/About Me values including wallet, level, collection, quest and age or play-time displays. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfdd20` | `CharacterDataSource::DtorBody` | CharacterDataSource | Cleans the data-source strings and base table during object destruction. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfdf20` | `CharacterDataSource::Ctor` | CharacterDataSource | Initializes the player profile data source and its owned strings. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe070` | `CharacterDataSource::ScalarDeletingDtor` | CharacterDataSource | Implements the scalar deleting destructor for CharacterDataSource. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `0102b8a0` | `LoadCharacterStatDefinitions` |  | Loads the CharacterStatDefinitions.txt table (stat ids 1..70) into the 71-bucket CharacterStatDefinition hash list. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102c930` | `LoadCharacterStatSetMappings` |  | Loads CharacterStatSetMappings.txt into the 10-bucket CharacterStatSet table, grouping stat entries under each key. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |

## 🟢 likely (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00712110` | `CharacterAttachmentRecord_Ctor` |  | Constructor of the CharacterAttachment record: six StringFixed<32> members and an embedded CharacterAttachmentData list at +0x14c. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00712440` | `StringListRecord_CopyAssign` |  | Copy-assignment of the same record layout that 00712110 constructs; the caller 00719550 is the account character-list reply handler, so the record role rests on that neighbour. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713b90` | `RecordAndString_Ctor` |  | Constructs the record-plus-string sub-object at +8 of the list-node element built by 00713e10. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713bf0` | `RecordAndString_DtorBody` |  | Destructor body of the record-plus-string sub-object; reached from EH unwind funclets and from 00713c90. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713c90` | `ListNodeWithRecord_DtorBody` |  | Destructor body of the list-node element stored by 00713f20 and 00713fc0; reached from the scalar deleting destructor 00714060. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713e10` | `ListNodeWithRecord_Ctor` |  | Constructs the element stored in the list manipulated by 00713f20 and 00713fc0 (its wrapper 00713ed0 allocates and constructs it). | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713ed0` | `Wrapper_00713e10` |  | Constructs the list-node element after allocation; used by 00713f20 and 00713fc0. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713f20` | `ListNodePushFront` |  | Front insertion into the list of list-node elements; caller 007140c0 (ClientAccountManager::CreatePlayer, confirmed neighbour). | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713fc0` | `ListNodePushBack` |  | Back insertion into the list of list-node elements; callers 007140c0 (CreatePlayer) and 00719550 (OnRequestCharacterList) are confirmed neighbours. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00714060` | `ListNodeWithRecord_ScalarDtor` |  | Scalar deleting destructor of the list-node element. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00960960` | `ClearStateAndUpdateWaterRelatedObject` |  | Clears per-object state and notifies the related water-state path when the stored identifiers match. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00b49a10` | `SetTitleIdAndNotify` |  | Updates one title id and notifies its associated owner callback only when the id changes. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49a50` | `Thunk_00cc55d0` |  | Forwards a title-state notification call through the owner pointer to an unresolved target. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00713e80` | `Wrapper_00713c90 ?` |  | Calls 00713c90 with ECX=this and no stack arguments; returns void. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `009616c0` | `CanRunCharacterAction ? ⚠rename` |  | A boolean gate over related-object state and character flags; exact action contract is not named in the body. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |

