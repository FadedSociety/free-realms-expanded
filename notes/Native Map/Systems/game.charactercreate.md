# System: game.charactercreate

163 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (61)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00705610` | `AccountManagerStateToString` | ClientAccountManager | Names the account-manager states used by server selection and character creation. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `0070cc80` | `LogAcceptedCreationName` |  | Logs accepted character-creation name fields. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070d0c0` | `LogNameCheckInput` |  | Records character name-check input. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070ea10` | `ClientAccountManager::OnRequestFriendsList` | ClientAccountManager | Processes account friend-list callback results. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070f170` | `LoadCreateCharacterErrors` |  | Loads create-character error mapping rows. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070f1b0` | `LoadCheckNameErrors` |  | Loads name-check error mapping rows. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070f1f0` | `LoadLoginResultErrors` |  | Loads login-result error mapping rows. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070f230` | `LoadCharacterDeleteErrors` |  | Loads character-delete error mapping rows. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070f490` | `ClientAccountManager::Initialize` | ClientAccountManager | Initializes character-create, check-name, login and delete error resources. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `007100c0` | `ClientAccountManager::OnRequestCharacterCreate` | ClientAccountManager | Processes character-create service callback results. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `00710e20` | `ClientAccountManager::OnRequestCheckName` | ClientAccountManager | Processes name-check results for character creation. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `00711220` | `ClientAccountManager::OnRequestCharacterLogin` | ClientAccountManager | Processes character-login callback and stores gateway credentials. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `00711a60` | `SelectCharacterDummyByRaceGender` |  | Selects a character-creation dummy by race and gender. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `00711ca0` | `ClientAccountManager::OnRequestServerList` | ClientAccountManager | Processes server-list callback results for account selection. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `007140c0` | `ClientAccountManager::CreatePlayer` | ClientAccountManager | Client-side character creation record setup. The function's own log strings name ClientAccountManager::CreatePlayer and its starting-set, clothing, and customization lookup failures; no server request is shown. | [[Native Map/Shards/S0147 007140c0-00715ea0|S0147]] |
| `0071a530` | `ClientAccountManager::Constructor` | ClientAccountManager | Creates the account and character-creation manager. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071a8f0` | `ClientAccountManager::Destructor` | ClientAccountManager | Tears down ClientAccountManager-owned state. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071b350` | `ReplaceRefCountedMemberA260` | ClientAccountInterface | Replaces a reference-counted member; its semantic identity is unknown. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071c150` | `HandleCustomizationSlot` |  | Applies character-creation changes for hair, model, and other appearance slots. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071cb20` | `ClientAccountInterface::Destructor` | ClientAccountInterface | Destroys ClientAccountInterface and its owned manager. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071cec0` | `ClientAccountInterface::Constructor` | ClientAccountInterface | Constructs character-creation account interface and data manager. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071d010` | `ClientAccountInterface::ScalarDeletingDtor` | ClientAccountInterface | Scalar deleting destructor for ClientAccountInterface. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `00721990` | `CCItemMappingsDefinition_ScalarDeletingDtor` | CCItemMappingsDefinition | MSVC scalar deleting destructor of a character-create item-mapping row (no members need tearing down). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007219f0` | `CCItemTypesDefinition_ScalarDeletingDtor` | CCItemTypesDefinition | MSVC scalar deleting destructor of a character-create item-type row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721a50` | `CCItemClothingSetsDefinition_ScalarDeletingDtor` | CCItemClothingSetsDefinition | MSVC scalar deleting destructor of a character-create clothing-set row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721a70` | `NewCCItemMappingsDefinition` |  | Allocating constructor for one CharacterCreateItemMappings.txt row (first column kept at +0x14, owner use at +0x14/+8 link fields). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721af0` | `NewCCItemTypesDefinition` |  | Allocating constructor for one CharacterCreateItemTypes.txt row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721b70` | `NewCCItemClothingSetsDefinition` |  | Allocating constructor for one CharacterCreateItemClothingSets.txt row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722c60` | `CCNamesDefinition::~CCNamesDefinition` | CCNamesDefinition | Destructor of one CharacterCreateNames row, releasing its embedded name string. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722cf0` | `CCNamesDefinition::CCNamesDefinition` | CCNamesDefinition | Constructor body of one CharacterCreateNames row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723200` | `CCItemStartingSetsDefinition::CCItemStartingSetsDefinition` | CCItemStartingSetsDefinition | Constructor of one CharacterCreateItemStartingSets.txt row (per race/gender starting appearance). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007232c0` | `CCItemStartingSetsDefinition::~CCItemStartingSetsDefinition` | CCItemStartingSetsDefinition | Destructor of one starting-set row, releasing its embedded fixed string. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007236c0` | `LoadCharacterCreateNames` |  | Loads the character-name pool table used by the CharacterCreate namer UI. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723860` | `LoadCharacterCreateItemMappings` |  | Loads the creator item-mapping table that maps a creator slot choice to a stored item/definition id. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723a70` | `LoadCharacterCreateItemTypes` |  | Loads the creator item-type table (defines the per-type columns used by the other creator tables). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723c80` | `LoadCharacterCreateItemClothingSets` |  | Loads the creator clothing-set table (which item ids make up chest/legs/feet per set). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723e90` | `LoadCharacterCreateItemStartingSets` |  | Loads the per race/gender starting clothing and appearance sets used when a character is first created. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724d80` | `LoadAllDefinitionFiles` |  | Single entry point that loads every definition table the character creator needs and then the five model-customization mapping files. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725110` | `ListCCNames_Destruct` |  | Destructor body of the CharacterCreateNames list container. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725140` | `ListCCNames_ScalarDeletingDtor` |  | MSVC scalar deleting destructor of the CharacterCreateNames list container. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725180` | `ClientAccountDataManager::ClientAccountDataManager` | ClientAccountDataManager | Constructor of the account data manager that owns every definition container the character creator and item lookups use. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725470` | `ClientAccountDataManager::~ClientAccountDataManager` | ClientAccountDataManager | Destructor of the account data manager, releasing every definition container it owns. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725540` | `CCNamesDefinition_ParseFields` | CCNamesDefinition | Field parser for one CharacterCreateNames.txt row (name text plus per-row attributes). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725f40` | `IdToStringLookup::IdToStringLookup` | IdToStringLookup | Constructor of the shared id-to-string mapping table used by the five CharacterCreate mapping files. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725ff0` | `IdToStringLookup::~IdToStringLookup` | IdToStringLookup | Destructor of an id-to-string mapping table (the file-name string plus all loaded entries). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `0097c960` | `CharacterCreationManager_CreateStartingSetsDataSource` | CharacterCreationManager | One of the manager's data-source bootstrap steps: it loads the CharacterCreateItemStartingSets table (starting equipment/appearance sets for a new character) as a client GDS data source. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097eeb0` | `CustomizationCategoryNode_CtorBase` |  | Sets up the embedded data-source-name-suffix string and first fixed string of a category node before 0097f070 stores the category id. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f690` | `CharacterCreationManager_CreateCustomizationCategoryDataSource` | CharacterCreationManager | Loads CharacterCreateCustomizationCategory.txt (the table of customization categories) as a GDS data source for the creator. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f790` | `CharacterCreationManager_CreateNameDataSource` | CharacterCreationManager | Creates the per-name-category GDS data source ('BaseClient.<category name>') while walking the name categories in 00980c50. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f960` | `CharacterCreationManager_CreateServerListDataSource` | CharacterCreationManager | Loads the realm/server list the character-create screens show (and that the audit note ties to the default server 0/1 handling). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097fa70` | `CharacterCreationManager_CreateAccountInfoDataSource` | CharacterCreationManager | Loads the account information (available character slots, entitlements) shown during character creation. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097fb80` | `CharacterCreationManager_CreatePresetDataSource` | CharacterCreationManager | Prepares the file-based character-preset scanning rooted at Resources\ and its GDS data source, which the preset browser reads. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00980370` | `CharacterCreationManager_ParseCharacterTypes` | CharacterCreationManager | Loads the table of playable character type rows (typeId, localized name id, race and gender) that the creator uses to enumerate Human/Pixie male/female combinations. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00980540` | `CharacterCreationManager_ParseCustomizationCategories` | CharacterCreationManager | Loads the category table (SkinTone, FaceType, HairStyle, ...) that drives the per-category customization data sources; the DataSourceNameSuffix is the stem used by 0097f570. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00980740` | `CharacterCreationManager_ParseCustomizationCategoryData` | CharacterCreationManager | Loads one row per selectable customization element (icon plus display strings) into the list the category UI reads. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `009809f0` | `CharacterCreationManager_CreateCustomizationDataSources` | CharacterCreationManager | Bootstraps one GDS data source per customization category so the creator can read each category's rows; this is the loop that produces names like BaseClient.CharacterCreateCustomizationSkinTone. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00980c50` | `CharacterCreationManager_CreateNameCategoryDataSources` | CharacterCreationManager | Builds one GDS data source per name category so the name screen can pull candidate names. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `009829f0` | `CharacterCreationManager_ParseCharacterNames` | CharacterCreationManager | Loads the per-category name pools (the name wheel) used by the name-entry screen of character creation. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00982f10` | `CharacterCreationManager::LoadCharacterCreationData` | CharacterCreationManager | Startup loader that fills CharacterCreationManager from the packaged character-creation text tables (character types, customization categories and per-category data, character names, presets, server list, account info, … | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00983500` | `CharacterCreationManager::~CharacterCreationManager` | CharacterCreationManager | Teardown body of CharacterCreationManager, reached from its compiler-generated scalar deleting destructor (008a5c80 ScalarDelete_983500) and from other owners (008b8aa0, 009464c0); the member vtable stores identify the … | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `009836e0` | `CharacterCreationManager::CharacterCreationManager` | CharacterCreationManager | Constructs the character-creation manager and its member containers; member offsets line up with the destructor 00983500 (+0x244 CustomizationData map, +0x280 CharacterCreateDataSource map, +0x340 ServerQueryInterface m… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |

## 🟢 likely (87)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007073a0` | `ClientAccountManagerListNodeAtIndex` | ClientAccountManager | Returns an indexed node from the list traversed by the account-manager run loop. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `0070ca40` | `BuildHairDescriptorPath` |  | Builds an appearance resource path. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070cc70` | `SetAccountManagerState30` |  | Requests account-manager state 0x30 through the shared state setter. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070cfa0` | `SetTemporaryPlayerName` |  | Updates/logs the temporary character name. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070cff0` | `SetAccountManagerState12` |  | Requests account-manager state 0x12 through the shared state setter. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070d000` | `EnterAccountState4` |  | Starts a guarded account-manager transition. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070d040` | `EnterAccountState2` |  | Starts a guarded account-manager transition. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070e7c0` | `LoadWebServiceErrorTable` |  | Loads web-service error mappings. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070e9a0` | `HasChildLinks` |  | The state setter and callback are side effects of this account-manager predicate; its caller context is ClientAccountManager::Run at 00715ec0. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `0070f270` | `SelectCharacterDummyIndex` |  | Caller 00711a60 supplies the race/gender-matched dummy index; this function stores and applies that selection rather than calculating a next index. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `00721c70` | `HashList50_FindKeyAt9c78` |  | Looks up a character-create item-mapping row by its id (the table fed by the CharacterCreateItemMappings.txt loader). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721cb0` | `HashList10_FindKeyAt9d50` |  | Looks up a character-create item-type row by id (table fed by the CharacterCreateItemTypes.txt loader). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721cf0` | `HashList50_FindKeyAt9d88` |  | Looks up a character-create clothing-set row by id (table fed by the CharacterCreateItemClothingSets.txt loader). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721d30` | `HashList50_FindKeyAt9e60` |  | Looks up a character-create starting-set row by id (table fed by the CharacterCreateItemStartingSets.txt loader). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721d70` | `CCNamesList_GetByIndex` |  | Indexed access into the loaded CharacterCreateNames row list. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722850` | `HashList50_RemoveAll_A` |  | Destroys every row of one character-create definition list (50-bucket instantiation). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007228f0` | `HashList10_RemoveAll` |  | Destroys every row of the 10-bucket character-create definition list (item types). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722990` | `HashList50_RemoveAll_B` |  | Destroys every row of a second 50-bucket character-create definition list. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722a30` | `HashList50_RemoveAll_C` |  | Destroys every row of a third 50-bucket character-create definition list. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722cd0` | `CCNamesDefinition_ScalarDeletingDtor` | CCNamesDefinition | MSVC scalar deleting destructor of a CharacterCreateNames row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722e40` | `ConstructCCNamesDefinition` |  | Construction step used when appending a new row to the CharacterCreateNames list. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722ff0` | `CCNamesDefinitionDtorCall` |  | Base-subobject destructor trampoline for an unnamed class deriving from CCNamesDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723320` | `CCItemStartingSetsDefinition_ScalarDeletingDtor` | CCItemStartingSetsDefinition | MSVC scalar deleting destructor of a starting-set row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723340` | `NewCCItemStartingSetsDefinition` |  | Allocating factory used once per data line of CharacterCreateItemStartingSets.txt; the empty shipped table is why no starting set exists for any race/gender. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723410` | `CCNamesList_Append` |  | Appends one row to the loaded CharacterCreateNames list; the row list is indexed later by CCNamesList_GetByIndex. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723660` | `CCNamesDefinition_ScalarDeletingDtor_B` | CCNamesDefinition | Second MSVC scalar deleting destructor variant covering a CCNamesDefinition base subobject. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724150` | `CCNamesList_Remove` |  | Removes one CharacterCreateNames row from the list and destroys it. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724d60` | `CCNamesList_RemoveAll` |  | Empties the loaded CharacterCreateNames list. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725030` | `ClientAccountDataManager_ClearDefinitions` | ClientAccountDataManager | Releases all definition data owned by the account data manager, as part of its teardown or reload. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725520` | `ClientAccountDataManager_ScalarDeletingDtor` | ClientAccountDataManager | MSVC scalar deleting destructor of the account data manager. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007255e0` | `CCItemMappingsDefinition_ParseFields` | CCItemMappingsDefinition | Field parser for one CharacterCreateItemMappings.txt row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725690` | `CCItemTypesDefinition_ParseFields` | CCItemTypesDefinition | Field parser for one CharacterCreateItemTypes.txt row. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725710` | `CCItemClothingSetsDefinition_ParseFields` | CCItemClothingSetsDefinition | Field parser for one CharacterCreateItemClothingSets.txt row (item ids of the four gear slots plus a set id). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007257d0` | `CCItemStartingSetsDefinition_ParseFields` | CCItemStartingSetsDefinition | Field parser for one CharacterCreateItemStartingSets.txt row (race, gender and the starting appearance item ids used at character creation). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725930` | `HashList100_Construct` |  | Initializes the 100-bucket id-to-string hash header used by the mapping-file lookups. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725970` | `HashList100_Unlink` |  | Removes one entry from a 100-bucket id-to-string lookup list. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725a70` | `IdHash_FindValue` |  | Id-to-payload lookup used by the creator code (for example resolving a skin-tone texture alias id to its string). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725d90` | `IdToStringLookup_RemoveAll` |  | Empties an IdToStringLookup mapping table and frees every id-to-string entry. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `0097d670` | `CharacterCreationManager_ReleaseMember208` | CharacterCreationManager | Tear-down path for one of the manager's data sources/objects; the member offset pairs it with the creator that stored it (e.g. +0x360 is filled by 0097f960 with the BaseClient.ServerList data source). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d6c0` | `CharacterCreationManager_ReleaseMember27c` | CharacterCreationManager | Tear-down path for one of the manager's data sources/objects; the member offset pairs it with the creator that stored it (e.g. +0x360 is filled by 0097f960 with the BaseClient.ServerList data source). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d7b0` | `ReleaseTrackedObjectPtr` |  | Generic cleanup for a character-creation object reference held by pointer-to-pointer; duplicated at 0097d800. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d800` | `ReleaseTrackedObjectPtrCopy` |  | Second COMDAT copy of the same pointer release helper. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d850` | `CharacterCreationManager_GetNameWheelNodeCount` | CharacterCreationManager | Accessor for the number of loaded name-wheel categories (count is decremented by the node remover 009828e0 and incremented by the inserter 00982820). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d860` | `CharacterCreationManager_ReleaseMember360` | CharacterCreationManager | Tear-down path for one of the manager's data sources/objects; the member offset pairs it with the creator that stored it (e.g. +0x360 is filled by 0097f960 with the BaseClient.ServerList data source). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d8b0` | `CharacterCreationManager_ReleaseMember364` | CharacterCreationManager | Tear-down path for one of the manager's data sources/objects; the member offset pairs it with the creator that stored it (e.g. +0x360 is filled by 0097f960 with the BaseClient.ServerList data source). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d900` | `CharacterCreationManager_ReleasePresetMembers` | CharacterCreationManager | Tear-down of the preset file scanner (+0x338) and the BaseClient.CharacterPresets data source (+0x33c) that 0097fb80 created. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d970` | `CharacterCreationManager_GetPresetScannerField14` | CharacterCreationManager | Reads a small field (id or state) of the character-preset scan object. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097d9c0` | `CharacterCreationManager_ReleaseMember368` | CharacterCreationManager | Tear-down path for one of the manager's data sources/objects; the member offset pairs it with the creator that stored it (e.g. +0x360 is filled by 0097f960 with the BaseClient.ServerList data source). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e450` | `ReleaseTrackedObjectPtr_EH` |  | Inlined destructor for an object that owns a tracked reference; the unwind funclets that call it identify the owning class, which is outside this shard. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e4a0` | `ReleaseTrackedObjectPtr_EH2` |  | Destructor of another member reference; called from three unwind funclets of neighbouring classes. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e4f0` | `HashList4_FindNodeByString` | CharacterCreationManager | Locates a name-wheel category node (the per-category name lists used by the character-name screen) by its display name. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e590` | `CharacterCreationManager_GetNameCategoryCount` | CharacterCreationManager | Answers how many names exist in a given name category for the name-selection UI. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e700` | `ReleaseTrackedObjectPtr_SEH` |  | Member destructor variant used from EH-protected teardown (callers 0097ecc0 deleting destructor and 00980ef0 container clear). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e780` | `ReleaseTrackedObjectPtr_SEH2` |  | Likely the element destructor of the List<CharacterNamesDataSourceElement> container; callers are 0097ece0 and 00980f80. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e8b0` | `NameElementList_InsertNewTail` |  | Appends a new CharacterNamesDataSourceElement node (0xc-byte nodes) to the list used by the name data-source creation loop 00980c50. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e940` | `NodeCtor_StringFixed32_Key3c` |  | Node constructor for the 256-bucket name table of a name category (key = NAME_ID at +0x3c). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097e9a0` | `TwoStringFixed32_Dtor` |  | Sub-object destructor of a struct holding two short strings (the race/gender pair of a CharacterType node). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097ea10` | `CustomizationCategoryNode_Dtor` |  | Element destructor used by the CustomizationCategory cache removal (caller 00980130) and its scalar deleting destructor 0097f450. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097eab0` | `CustomizationDataNode_Dtor` |  | Element destructor of the per-category customization data list (caller 00980280 removes and destroys nodes with it). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097ecc0` | `RefPtrWrapper_ScalarDeletingDtor` |  | Compiler-emitted deleting destructor for the wrapper whose only member is the tracked reference released by 0097e700. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097ece0` | `RefPtrWrapper_ScalarDeletingDtor2` |  | Deleting destructor for the second tracked-reference wrapper. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097ee40` | `NodeCtor_TwoStringFixed32` |  | Constructor of the CharacterType list node (0x78 bytes) whose race/gender strings are filled by the parser 00980370. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097ef30` | `CustomizationDataNode_Ctor` |  | Allocates the in-object state for one row of the customization data table (icon fields plus Name/Description strings). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f020` | `CharacterTypeNode_CtorWrapper` |  | Node factory step used before the CharacterType insert 0097fd30 fills id/nameId/race/gender. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f070` | `CustomizationCategoryNode_CtorWithKey` |  | Node construction for the category cache; the caller 0097fdc0 passes the categoryId parsed from the table. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f0d0` | `CustomizationDataNode_CtorWrapper` |  | Node factory step of the CustomizationData list insert (caller 0097ffb0). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f3f0` | `ScalarDeletingDtor_TwoFixed32` |  | Deleting destructor of the same two-string struct whose destructor body is 0097e9a0. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f450` | `CustomizationCategoryNode_ScalarDeletingDtor` |  | Deleting destructor entry point for the customization-category node. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f4b0` | `ScalarDeletingDtor_Fixed32` |  | Deleting destructor of a struct whose only member is one StringFixed<32> (matches the dtor thunk 0097eda0). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f510` | `CustomizationDataNode_ScalarDeletingDtor` |  | Deleting destructor of the CustomizationData list node. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f570` | `CharacterCreationManager_FormatCategorySuffix` | CharacterCreationManager | Builds the per-category GDS resource stem 'CharacterCreateCustomization<suffix>' (the suffix is the DataSourceNameSuffix column of the category table). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f5d0` | `CharacterCreationManager_BuildCategoryDataSourceName` | CharacterCreationManager | Produces the 'BaseClient.CharacterCreateCustomization...' name that 0097c810 loads as a data source for the chosen category. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f8a0` | `CharacterCreationManager_GetNameCategoryNameAt` | CharacterCreationManager | Indexed read of a category's name for the name-selection UI (caller 00c2bb70). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f8e0` | `CharacterCreationManager_GetNameInCategory` | CharacterCreationManager | Indexed lookup of 'the i-th name of category X', used by the name screen to populate name suggestions. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097f940` | `CharacterCreationManager_FormatFullDataSourceName` |  | Small helper producing the 'BaseClient.<name>' identity of a name data source (caller 00c2bb70). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097fd30` | `List_CharacterType_InsertNewNode` |  | Insert used by the CharacterTypes table parser 00980370 to add one character type row. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097fdc0` | `HashList9_InsertNewCategoryNode` |  | Insert used by the CustomizationCategories parser 00980540; the key is the categoryId column. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `0097ffb0` | `List_CustomizationData_InsertNewNode` |  | Insert used by the CustomizationCategoryData parser 00980740. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00981020` | `CharacterCreationManager_ClearCharacterCreateDataSources` | CharacterCreationManager | Full teardown of the per-category customization data sources created by 009809f0. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00981080` | `CharacterCreationManager_ClearNameDataSources` | CharacterCreationManager | Teardown counterpart of 00980c50 (name categories and their data sources). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `009825c0` | `CharacterNamesDataSource_Ctor` |  | Sets up the name table object that backs one name category data source (inserts go through 0097fef0, lookups through 0097cbe0). | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00982620` | `CharacterNamesDataSource_Dtor` |  | Tear-down counterpart of 009825c0. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `009826b0` | `CharacterNamesDataSource_DtorThunk` |  | Thunk allowing the data-source destructor to be invoked from unwind/atexit tables. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00982700` | `NameWheelCategoryNode_Ctor` |  | Node construction for the name-category cache; the key at +0x450 is the hash used by 0097e4f0/00982820/009828e0. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `009827c0` | `NameWheelCategoryNode_ScalarDeletingDtor` |  | Deleting destructor entry of the category node. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `00982820` | `HashList4_InsertNewCategoryNode` | CharacterCreationManager | Adds a name-wheel category (with its own 256-name table) to the manager when the CharacterNames parser meets a new category name. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |
| `009828e0` | `HashList4_RemoveCategoryNode` | CharacterCreationManager | Node removal for the name-category cache. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |

## 🟡 speculative (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00705a60` | `InvokeAccountManagerService ? ⚠rename` | ClientAccountManager | Checks account-manager service conditions and logs the supplied service value. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `0070d080` | `EnterAccountState9 ?` |  | Requests account-manager state 9. | [[Native Map/Shards/S0145 0070ca40-00711ff0|S0145]] |
| `007253a0` | `GetField_14 ? → ClientAccountDataManager::GetField_14` |  | Returns the dword field at this+0x14. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007253b0` | `GetField_9c64 ? → ClientAccountDataManager::GetField_9c64` |  | Returns the dword field at this+0x9c64. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007253c0` | `GetField_9c5c ? → ClientAccountDataManager::GetField_9c5c` |  | Returns the dword field at this+0x9c5c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007253d0` | `GetField_9c74 ? → ClientAccountDataManager::GetField_9c74` |  | Returns the dword field at this+0x9c74. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007253e0` | `GetField_9c6c ?` |  | Returns the dword field at this+0x9c6c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007253f0` | `GetField_10 ?` |  | Returns the dword field at this+0x10. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725400` | `GetField_9d4c ? → ClientAccountDataManager::GetField_9d4c` |  | Returns the dword field at this+0x9d4c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725410` | `GetField_9d84 ?` |  | Returns the dword field at this+0x9d84. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725420` | `GetField_9e5c ? → ClientAccountDataManager::GetField_9e5c` |  | Returns the dword field at this+0x9e5c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725430` | `GetField_9e54 ?` |  | Returns the dword field at this+0x9e54. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725440` | `GetField_10_B ?` |  | Returns the dword field at this+0x10; duplicate body of 007253f0. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00725460` | `GetField_28 ? → ClientAccountDataManager::GetField_28` |  | Returns the dword field at this+0x28. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `0097d990` | `CharacterCreationManager_GetPresetListEntry ?` | CharacterCreationManager | Indexes a list kept on the preset scanner object; the same +0x40 chain is used by the StringFixed<32> name list (0097fef0/0097cbe0), so the offset may be overloaded across the two containers. | [[Native Map/Shards/S0262 0097c960-00982e70|S0262]] |

