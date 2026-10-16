# System: ui.pets

101 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (59)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a782d0` | `PetTooltip_BuildRows` |  | Fills the PetTooltip row table for a pet: name, and the hunger, hygiene and related stat values. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a784e0` | `PetTrickTooltip_BuildRows` |  | Fills the PetTrickTooltip rows (trick name and value, plus the Skill value) for a pet trick. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00b53450` | `PetsHandler::PopulatePetInventory` | PetsHandler | PopulatePetInventory handler of the client pet UI: rebuild the pet list/inventory views and then hand control to the Lua controller. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b53530` | `PetsHandler::PetCanDraw` | PetsHandler | PetCanDraw handler of the client pet UI: gate a draw/visibility request on the handler's own actor guid. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00cca5c0` | `PetListDataSource::GetColumnName` | PetListDataSource | Supplies column names to pet-list UI data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca690` | `PetActionDataSource::GetColumnName` | PetActionDataSource | Provides field names for pet action rows. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca6f0` | `PetTrickDataSource::GetColumnName` | PetTrickDataSource | Provides column names for pet trick rows. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca7b0` | `PetTrickFavoriteDataSource::GetColumnName` | PetTrickFavoriteDataSource | Provides field names for favorite-trick rows. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca820` | `PetNameDataSource::GetColumnName` | PetNameDataSource | Provides column names for pet-name entries. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca880` | `PetEquipScreenDataSourceCtor` | PetEquipScreenDataSource | Constructs the pet equipment screen data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca910` | `PetEquipScreenDataSourceDtor` | PetEquipScreenDataSource | Destroys PetEquipScreenDataSource base-owned data. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca970` | `PetUtilityScreenDataSourceCtor` | PetUtilityScreenDataSource | Constructs the pet utility screen data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccaa10` | `PetUtilityScreenDataSourceDtor` | PetUtilityScreenDataSource | Destroys PetUtilityScreenDataSource base-owned data. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccaab0` | `PetEquippedWearablesDataSourceCtor` | PetEquippedWearablesDataSource | Constructs the pet equipped-wearables data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccab50` | `PetEquippedWearablesDataSourceDtor` | PetEquippedWearablesDataSource | Destroys PetEquippedWearablesDataSource base-owned data. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccabc0` | `PetEquippedUtilityDataSourceCtor` | PetEquippedUtilityDataSource | Constructs equipped-pet-utility data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccac60` | `PetEquippedUtilityDataSourceDtor` | PetEquippedUtilityDataSource | Destroys PetEquippedUtilityDataSource base-owned data. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccacc0` | `BasePetTrickDataSource::GetColumnName` | BasePetTrickDataSource | Supplies shared item columns for three pet-trick data sources. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccadf0` | `PetGestureDataSource::GetColumnName` | PetGestureDataSource | Supplies gesture waypoint fields to its UI data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb1d0` | `PetEquippedUtilityDataSource::DtorAdjustorThunk` | PetEquippedUtilityDataSource | Routes an embedded utility-data-source view to the scalar deleting destructor. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb1e0` | `PetUtilityScreenDataSource::DtorAdjustorThunk` | PetUtilityScreenDataSource | Routes embedded utility-screen view to its scalar deleting destructor. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb1f0` | `PetEquippedWearablesDataSource::DtorAdjustorThunk` | PetEquippedWearablesDataSource | Routes embedded wearable-data-source view to its scalar deleting destructor. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb200` | `PetEquipScreenDataSource::DtorAdjustorThunk` | PetEquipScreenDataSource | Routes embedded equipment-data-source view to its scalar deleting destructor. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb210` | `PetActiveDataSource::RefreshActivePetRows` | PetActiveDataSource | Updates active-pet rows for the currently selected pet. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb380` | `PetEquipScreenDataSourceScalarDeletingDtor` | PetEquipScreenDataSource | MSVC scalar deleting destructor for pet equipment data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb3a0` | `PetEquipScreenDataSource::CanEquipItem` | PetEquipScreenDataSource | Filters inventory candidates for the pet equipment screen. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb450` | `PetUtilityScreenDataSourceScalarDeletingDtor` | PetUtilityScreenDataSource | MSVC scalar deleting destructor for pet utility screen data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb470` | `PetEquippedWearablesDataSourceScalarDeletingDtor` | PetEquippedWearablesDataSource | MSVC scalar deleting destructor for equipped-wearables pet data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb490` | `PetEquippedUtilityDataSourceScalarDeletingDtor` | PetEquippedUtilityDataSource | MSVC scalar deleting destructor for equipped-utility pet data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb4b0` | `PetEquippedUtilityDataSource::MatchesSlot` | PetEquippedUtilityDataSource | Filters for an item in one of the active pet's utility slots. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb530` | `PetTricksDataSource::GetActivePetField44` | PetTricksDataSource | Supplies one active-pet value to the pet tricks data source. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccba50` | `PetGestureDataSource::GetRowCount` | PetGestureDataSource | Reports waypoint row count for selected trick gesture. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccc0e0` | `PetNameDataSource::GetField` | PetNameDataSource | Provides a pet-name data source field value. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccc200` | `PetListDataSource::Refresh` | PetListDataSource | Refreshes pet-list rows after pet data changes. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccc420` | `PetTrickDataSource::GetCurrentPetField` | PetTrickDataSource | Exposes a current-pet value to the trick data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccc480` | `PetEquipScreenDataSource::HandleEquipSelection` | PetEquipScreenDataSource | Handles pet equipment selection in the pet equipment screen. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccca10` | `PetTrickDataSource::GetField` | PetTrickDataSource | Provides current-pet trick row values to the pet UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd120` | `PetTricksDataSource::GetField` | PetTricksDataSource | Provides the owned pet's trick list to the UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd1a0` | `PetTricksEquippedDataSource::GetField` | PetTricksEquippedDataSource | Provides equipped pet trick rows to the UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd350` | `PetListDataSource::Dtor` | PetListDataSource | Cleans up a pet list data source and its inherited GUI table state. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd3d0` | `PetActionDataSource::Dtor` | PetActionDataSource | Cleans up a pet action data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd450` | `PetTrickDataSource::Dtor` | PetTrickDataSource | Cleans up a pet trick data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd4c0` | `PetTrickFavoriteDataSource::Dtor` | PetTrickFavoriteDataSource | Cleans up a favorite-trick data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd530` | `PetNameDataSource::Dtor` | PetNameDataSource | Cleans up the pet-name data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd5b0` | `PetGestureDataSource::Dtor` | PetGestureDataSource | Cleans up the pet-gesture data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd620` | `PetListDataSource::Ctor` | PetListDataSource | Constructs a pet list data source over a GUI table. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd6b0` | `PetListDataSource::ScalarDeletingDtor` | PetListDataSource | MSVC scalar deleting destructor for the pet list data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd6d0` | `PetActiveDataSource::Ctor` | PetActiveDataSource | Creates the active-pet list data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd7a0` | `PetActiveDataSource::ScalarDeletingDtor` | PetActiveDataSource | MSVC scalar deleting destructor for the active-pet data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd800` | `PetActionDataSource::Ctor` | PetActionDataSource | Constructs the pet action table data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd890` | `PetActionDataSource::ScalarDeletingDtor` | PetActionDataSource | MSVC scalar deleting destructor for PetActionDataSource. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd8b0` | `PetTrickDataSource::Ctor` | PetTrickDataSource | Constructs the pet trick table data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd940` | `PetTrickDataSource::ScalarDeletingDtor` | PetTrickDataSource | MSVC scalar deleting destructor for PetTrickDataSource. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd960` | `PetTrickFavoriteDataSource::Ctor` | PetTrickFavoriteDataSource | Constructs the favorite pet trick table source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd9f0` | `PetTrickFavoriteDataSource::ScalarDeletingDtor` | PetTrickFavoriteDataSource | MSVC scalar deleting destructor for PetTrickFavoriteDataSource. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccda10` | `PetNameDataSource::Ctor` | PetNameDataSource | Constructs a pet-name table data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccdaa0` | `PetNameDataSource::ScalarDeletingDtor` | PetNameDataSource | MSVC scalar deleting destructor for PetNameDataSource. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccdac0` | `PetGestureDataSource::Ctor` | PetGestureDataSource | Constructs the pet gesture table data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccdb50` | `PetGestureDataSource::ScalarDeletingDtor` | PetGestureDataSource | MSVC scalar deleting destructor for PetGestureDataSource. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## 🟢 likely (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b53560` | `PetsHandler::ClearActivePetGuid` | PetsHandler | Clears the cached active-pet guid on the pet handler; 00a8e540 resolves the pair through the guid lookup 00a88f90 and drives its MiniMap:KillIndicator path for the resolved entry. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b535b0` | `PetsHandler::ResetPetPanel` | PetsHandler | Panel reset of the pet UI, run when the pet controller is closed (caller 00c07420). | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b535d0` | `PetsHandler::SetPendingValue` | PetsHandler | Stores a pending value on the pet panel sub-object (caller 00c06a60) before refreshing the UI. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b53600` | `PetsHandler::RefreshInventoryUi` | PetsHandler | Lighter inventory-UI refresh of the pet handler, run by the pet dispatcher 00b5ce60 without the panel rebuild that PopulatePetInventory performs. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b53710` | `PetsHandler::SetProxiedActorFlag` | PetsHandler | Sets a per-proxied-character flag on the pet handler and refreshes the pet UI; the node layout is the same hash used by FindPetInventoryItemByType (00b54930). | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b54080` | `PetsHandler::SetActivePet` | PetsHandler | Activates a pet by guid on the pet handler: caches the pair, places the map indicator and drives the pet controller (callers 00a12660, 00b589c0, 00b58bd0, 00b5ce60). | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b549f0` | `PetsHandlerOnSummonFailed` |  | Client-side check on the pet summon path (caller 00b60080, the summon sender of note 091): decides whether the summon must be reported as failed to the Lua pet controller. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00c066f0` | `Pet_TutorialEnd` |  | Handler registered as 'TutorialEnd' in the Pet module (0x01829f60): closes the pet tutorial sequence. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c06a60` | `Pet_SetGestureData` |  | Handler registered as 'SetGestureData' in the Pet module (0x01829f60): feeds the pet gesture minigame data into PetsHandler::SetPendingValue. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c06ac0` | `RegisterPetModule` |  | Registers the client 'Pet' Lua module whose entries include SetName, Summon, Dismiss, WasGood/WasBad, GetActivePetId, EquipItem/UnequipItem, PerformTrick, StartFeed, Delete - the API PetsHandler and the pet controller i… | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07990` | `Pet_EquipItem` |  | Handler registered as 'EquipItem' in the Pet module (0x01829f60): equips a cosmetic/gear item on the pet (utility items are routed to 00c077a0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00ccaa70` | `PetUtilityScreenDataSource::MatchesUtilityId ⚠downgrade` | PetUtilityScreenDataSource | Tests whether a utility item matches the data-source filter value. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccc560` | `PetEquippedWearablesDataSource::FindByWearable ⚠downgrade` | PetEquippedWearablesDataSource | Provides pet-equipped wearable rows for the pet UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0096c120` | `ShowThoughtBubbleAndTrack ? ⚠rename` |  | Shows the PetsHandler thought bubble when the tracked object matches and records the pending bubble entry. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00b53690` | `PetsHandler::RefreshCachedPetEntity ? ⚠rename` | PetsHandler | Refreshes and then releases the cached pet entity of the pet handler (callers 00bda800 and 00bdc3c0). | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b536e0` | `PetsHandler::ResolveOwnActor ?` | PetsHandler | Cheap own-actor lookup recorded from callers 0094caa0 and 00afd810; the name only states what is resolved, not why the result is dropped. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00ccb2c0` | `PetActionDataSource::RefreshAllRows ?` | PetActionDataSource | Refreshes all pet action rows. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb9d0` | `PetEquippedUtilityDataSource::RefreshRows ? ⚠rename` | PetEquippedUtilityDataSource | Builds equipped pet-utility rows from matching inventory entries. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |

## ⚪ unknown (24)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b53640` | `— ⚠reject` | PetsHandler | Clears the cached pet-entity guid; 009855d0 resolves the pair through the guid lookup 00984960 and calls 00985550 on the found entity, so the reset also routes the entity the pair pointed at. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00c066a0` | `— ⚠reject` |  | Handler registered as 'Summon' in the Pet module (entry table 0x01829f60): the client-side pet summon request used by the pet browser/dock. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c06a00` | `— ⚠reject` |  | Handler registered as 'ModeSwitch' in the Pet module (0x01829f60): switches the pet UI/util mode. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07420` | `— ⚠reject` |  | Handler registered as 'Dismiss' in the Pet module (0x01829f60); note 091 records this function as the action-0 (recall) sender of the PetSummonRecallPacket family. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c074c0` | `— ⚠reject` |  | Handler registered as 'WasGood' in the Pet module (0x01829f60): reports a successful pet interaction (trick/feed result) to the server. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07630` | `— ⚠reject` |  | Handler registered as 'WasBad' in the Pet module (0x01829f60): reports a failed pet interaction to the server. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c077a0` | `— ⚠reject` |  | Handler registered as 'EquipUtility' in the Pet module (0x01829f60): attaches a food/groom utility item to the pet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c078a0` | `— ⚠reject` |  | Handler registered as 'UnequipUtility' in the Pet module (0x01829f60): removes the pet's utility item. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07ae0` | `— ⚠reject` |  | Handler registered as 'UnequipItem' in the Pet module (0x01829f60). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07bc0` | `— ⚠reject` |  | Handler registered as 'Delete' in the Pet module (0x01829f60): deletes a pet (or a pending pet creation) by name. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cca5a0` | `— ⚠reject` | PetListDataSource | Refreshes pet-list data source when marked clean. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb300` | `— ⚠reject` | PetTrickFavoriteDataSource | Refreshes the four favorite-trick rows shown by pet UI. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb580` | `— ⚠reject` | PetGestureDataSource | Populates gesture waypoint rows from selected trick points. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb980` | `— ⚠reject` | PetActionDataSource | Reports pet action row count to UI framework. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccb9b0` | `— ⚠reject` | PetTrickFavoriteDataSource | Reports the four favorite-trick slots to UI framework. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccc100` | `— ⚠reject` | PetEquippedUtilityDataSource | Supplies a pet-equipped utility row to the pet UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccc300` | `— ⚠reject` | PetTrickDataSource | Refreshes the displayed trick rows for the current pet. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccc680` | `— ⚠reject` | PetListDataSource | Writes editable fields in the pet list and active-pet data sources. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccc940` | `— ⚠reject` | PetActionDataSource | Provides pet action data to the UI table. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cccc10` | `— ⚠reject` | PetTrickFavoriteDataSource | Supplies favorite trick rows to the pet UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cccdb0` | `— ⚠reject` | PetNameDataSource | Provides pet name choices and generated name entries to the pet UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cccfe0` | `— ⚠reject` |  | Supplies values for a pet trick row; the owner class name is not present in this function. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd220` | `— ⚠reject` | PetGestureDataSource | Provides gesture choices to the pet UI. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccd750` | `— ⚠reject` | PetListDataSource | Compiler-generated cleanup wrapper for a pet list data source. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c07420` | `— ⚠reject` | send |  | 0 | Hypothesis: the original server recalled/despawned the character's active pet on receiving a PetSummonRecallPacket whose action byte is 0 and confirmed with the pet status update; the inference depends on the caller path (Pet.Dismiss -> 00b5c5c0) and on the family base opcode 0x35. No pet datagram was captured, so the reply is unreconstructed. |
