# System: ui.collection

102 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (82)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fce30` | `CollectionCategoryDataSource::ColumnName` | CollectionCategoryManager::CollectionCa… | Supplies column names for the collection-category data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd050` | `FindCollectionCategoryByKey` |  | Looks up a collection category node by identifier. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd0b0` | `CollectionCategoryDataSource::GetValue` | CollectionCategoryManager::CollectionCa… | Supplies a category data-source value from its nested manager object. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd4e0` | `CollectionCategoryManager::LoadCategories` | CollectionCategoryManager | Loads the client collection-category definitions into the UI manager. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd7a0` | `CollectionCategoryDataSource::SetValue` | CollectionCategoryManager::CollectionCa… | Supplies values for collection-category UI rows. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd850` | `CollectionCategoryManager::Dtor` | CollectionCategoryManager | Tears down collection category registration and stored rows. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd8d0` | `CollectionCategoryManager::CollectionCategoryDataSource::Dtor` | CollectionCategoryManager::CollectionCa… | Cleans up the collection-category data source and its GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd930` | `CollectionCategoryManager::CollectionCategoryDataSource::Ctor` | CollectionCategoryManager::CollectionCa… | Initializes the collection-category data source and embedded GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd9b0` | `CollectionCategoryManager::CollectionCategoryDataSource::ScalarDeletingDtor` | CollectionCategoryManager::CollectionCa… | Scalar deleting destructor for the collection category data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00a78040` | `CollectionTooltip_BuildRows` |  | Fills the CollectionTooltip row table for a collection: started and total entry counts, completion flag and image set. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a93c80` | `NotifyCollectionStarted` |  | Client-side collection-start presentation and state update; it does not establish collection completion rules. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a93f40` | `NotifyCollectionUpdated` |  | Refreshes the client collection notification after a collection-state change. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a94380` | `NotifyCollectionCompleted` |  | Presents a completed collection to the client; reward or completion authority is not in this body. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00c0a5f0` | `RegisterCollectionScreenModule` |  | Registers the client 'CollectionScreen' Lua module (its single handler SelectCollection lives at 00c0a5b0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `01276bc0` | `Engine::CollectionRedeemItem::ActionName` | Engine::CollectionRedeemItem | Returns the bare action name RedeemItem for this Scaleform action class. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276be0` | `Engine::CollectionRedeemItem::DebugName` | Engine::CollectionRedeemItem | Debug label of the RedeemItem action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276bf0` | `Engine::CollectionRedeemItem::ScalarDeletingDtor` | Engine::CollectionRedeemItem | Scalar deleting destructor of the RedeemItem Scaleform action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276d10` | `Engine::CollectionProcessFulfillment::ActionName` | Engine::CollectionProcessFulfillment | Returns the bare action name ProcessFulfillment for this Scaleform action class. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276d30` | `Engine::CollectionProcessFulfillment::DebugName` | Engine::CollectionProcessFulfillment | Debug label of the ProcessFulfillment action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276d40` | `Engine::CollectionProcessFulfillment::ScalarDeletingDtor` | Engine::CollectionProcessFulfillment | Scalar deleting destructor of the ProcessFulfillment Scaleform action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276e60` | `Engine::CollectionRequestCollection::ActionName` | Engine::CollectionRequestCollection | Returns the bare action name RequestCollection for this Scaleform action class. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276e80` | `Engine::CollectionRequestCollection::DebugName` | Engine::CollectionRequestCollection | Debug label of the RequestCollection action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276e90` | `Engine::CollectionRequestCollection::ScalarDeletingDtor` | Engine::CollectionRequestCollection | Scalar deleting destructor of the RequestCollection Scaleform action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276fb0` | `Engine::CollectionRequestPacks::ActionName` | Engine::CollectionRequestPacks | Returns the bare action name RequestPacks for this Scaleform action class. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276fd0` | `Engine::CollectionRequestPacks::DebugName` | Engine::CollectionRequestPacks | Debug label of the RequestPacks action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276fe0` | `Engine::CollectionRequestPacks::ScalarDeletingDtor` | Engine::CollectionRequestPacks | Scalar deleting destructor of the RequestPacks Scaleform action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277100` | `Engine::CollectionRequestProductTypes::ActionName` | Engine::CollectionRequestProductTypes | Returns the bare action name RequestProductTypes for this Scaleform action class. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277120` | `Engine::CollectionRequestProductTypes::DebugName` | Engine::CollectionRequestProductTypes | Debug label of the RequestProductTypes action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277130` | `Engine::CollectionRequestProductTypes::ScalarDeletingDtor` | Engine::CollectionRequestProductTypes | Scalar deleting destructor of the RequestProductTypes Scaleform action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277250` | `Engine::CollectionRequestTickets::ActionName` | Engine::CollectionRequestTickets | Returns the bare action name RequestTickets for this Scaleform action class. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277270` | `Engine::CollectionRequestTickets::DebugName` | Engine::CollectionRequestTickets | Debug label of the RequestTickets action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277280` | `Engine::CollectionRequestTickets::ScalarDeletingDtor` | Engine::CollectionRequestTickets | Scalar deleting destructor of the RequestTickets Scaleform action. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `012773d0` | `Engine::CollectionRespondTickets::DebugName` | Engine::CollectionRespondTickets | Debug/registration tag of the respond-tickets display action, with its source file and line. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012773e0` | `Engine::CollectionRespondTickets::ScalarDeletingDtor` | Engine::CollectionRespondTickets | Heap-delete path of the respond-tickets display action object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277490` | `Engine::CollectionRequestCardData::ctor` | Engine::CollectionRequestCardData | Constructs the RequestCardData collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277500` | `Engine::CollectionRequestCardData::ActionName` | Engine::CollectionRequestCardData | Registered action name of the RequestCardData handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277520` | `Engine::CollectionRequestCardData::DebugName` | Engine::CollectionRequestCardData | Debug/registration tag with source file and line for the RequestCardData handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277530` | `Engine::CollectionRequestCardData::ScalarDeletingDtor` | Engine::CollectionRequestCardData | Heap-delete path of the RequestCardData handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012775e0` | `Engine::CollectionQueryMoz::ctor` | Engine::CollectionQueryMoz | Constructs the QueryMoz collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277650` | `Engine::CollectionQueryMoz::ActionName` | Engine::CollectionQueryMoz | Registered action name of the QueryMoz handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277670` | `Engine::CollectionQueryMoz::DebugName` | Engine::CollectionQueryMoz | Debug/registration tag with source file and line for the QueryMoz handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277680` | `Engine::CollectionQueryMoz::ScalarDeletingDtor` | Engine::CollectionQueryMoz | Heap-delete path of the QueryMoz handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277730` | `Engine::CollectionAddFilter::ctor` | Engine::CollectionAddFilter | Constructs the AddFilter collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012777a0` | `Engine::CollectionAddFilter::ActionName` | Engine::CollectionAddFilter | Registered action name of the AddFilter handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012777c0` | `Engine::CollectionAddFilter::DebugName` | Engine::CollectionAddFilter | Debug/registration tag with source file and line for the AddFilter handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012777d0` | `Engine::CollectionAddFilter::ScalarDeletingDtor` | Engine::CollectionAddFilter | Heap-delete path of the AddFilter handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277890` | `Engine::CollectionRemoveFilter::ctor` | Engine::CollectionRemoveFilter | Constructs the RemoveFilter collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277900` | `Engine::CollectionRemoveFilter::ActionName` | Engine::CollectionRemoveFilter | Registered action name of the RemoveFilter handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277920` | `Engine::CollectionRemoveFilter::DebugName` | Engine::CollectionRemoveFilter | Debug/registration tag with source file and line for the RemoveFilter handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277930` | `Engine::CollectionRemoveFilter::ScalarDeletingDtor` | Engine::CollectionRemoveFilter | Heap-delete path of the RemoveFilter handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012779f0` | `Engine::CollectionSetSearch::ctor` | Engine::CollectionSetSearch | Constructs the SetSearch collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277a60` | `Engine::CollectionSetSearch::ActionName` | Engine::CollectionSetSearch | Registered action name of the SetSearch handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277a80` | `Engine::CollectionSetSearch::DebugName` | Engine::CollectionSetSearch | Debug/registration tag with source file and line for the SetSearch handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277a90` | `Engine::CollectionSetSearch::ScalarDeletingDtor` | Engine::CollectionSetSearch | Heap-delete path of the SetSearch handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277b50` | `Engine::CollectionResetFilter::ctor` | Engine::CollectionResetFilter | Constructs the ResetFilter collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277bc0` | `Engine::CollectionResetFilter::ActionName` | Engine::CollectionResetFilter | Registered action name of the ResetFilter handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277be0` | `Engine::CollectionResetFilter::DebugName` | Engine::CollectionResetFilter | Debug/registration tag with source file and line for the ResetFilter handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277bf0` | `Engine::CollectionResetFilter::ScalarDeletingDtor` | Engine::CollectionResetFilter | Heap-delete path of the ResetFilter handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277cb0` | `Engine::CollectionOpenCollectionManager::ctor` | Engine::CollectionOpenCollectionManager | Constructs the OpenCollectionManager collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277d20` | `Engine::CollectionOpenCollectionManager::ActionName` | Engine::CollectionOpenCollectionManager | Registered action name of the OpenCollectionManager handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277d40` | `Engine::CollectionOpenCollectionManager::DebugName` | Engine::CollectionOpenCollectionManager | Debug/registration tag with source file and line for the OpenCollectionManager handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277d50` | `Engine::CollectionOpenCollectionManager::ScalarDeletingDtor` | Engine::CollectionOpenCollectionManager | Heap-delete path of the OpenCollectionManager handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277e10` | `Engine::CollectionCloseCollectionManager::ctor` | Engine::CollectionCloseCollectionManager | Constructs the CloseCollectionManager collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277e80` | `Engine::CollectionCloseCollectionManager::ActionName` | Engine::CollectionCloseCollectionManager | Registered action name of the CloseCollectionManager handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277ea0` | `Engine::CollectionCloseCollectionManager::DebugName` | Engine::CollectionCloseCollectionManager | Debug/registration tag with source file and line for the CloseCollectionManager handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277eb0` | `Engine::CollectionCloseCollectionManager::ScalarDeletingDtor` | Engine::CollectionCloseCollectionManager | Heap-delete path of the CloseCollectionManager handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277f30` | `Engine::CollectionSynchronizationStartInstancesDisplayAction::ctor` | Engine::CollectionSynchronizationStartI… | Constructs the display action that starts a collection instance synchronization. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277fc0` | `Engine::CollectionSynchronizationStartInstancesDisplayAction::DebugName` | Engine::CollectionSynchronizationStartI… | Debug/registration tag with source file and line for the synchronization-start display action. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01277fd0` | `Engine::CollectionSynchronizationStartInstancesDisplayAction::ScalarDeletingDtor` | Engine::CollectionSynchronizationStartI… | Heap-delete path of the synchronization-start display action object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278030` | `Engine::CollectionOpenPacks::ctor` | Engine::CollectionOpenPacks | Constructs the OpenPacks collection action handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012780a0` | `Engine::CollectionOpenPacks::ActionName` | Engine::CollectionOpenPacks | Registered action name of the OpenPacks handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012780c0` | `Engine::CollectionOpenPacks::DebugName` | Engine::CollectionOpenPacks | Debug/registration tag with source file and line for the OpenPacks handler. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012780d0` | `Engine::CollectionOpenPacks::ScalarDeletingDtor` | Engine::CollectionOpenPacks | Heap-delete path of the OpenPacks handler object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278130` | `Engine::EngineCollectionSynchronizationCompleteInstancesDisplayAction::Handle` | Engine::EngineCollectionSynchronization… | Completion handler of the engine-side collection synchronization display action; it notifies a service object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278170` | `Engine::CollectionOpenPacks::Handle` | Engine::CollectionOpenPacks | Opens packs by forwarding an ArchetypeID and Quantity pair (flag 1) to the collection service object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012781c0` | `Engine::CollectionRedeemItem::Handle` | Engine::CollectionRedeemItem | Redeems a collection item by forwarding the archetype choice and conversion/tradable flags to the collection service object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278250` | `Engine::CollectionProcessFulfillment::Handle` | Engine::CollectionProcessFulfillment | Processes a collection fulfillment by forwarding ArchetypeID, Quantity and IsTradable to the collection service object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278510` | `Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::Handle` | Engine::UICollectionSynchronizationComp… | On collection synchronization completion, closes the progress dialog and queues a RequestCollection command. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278680` | `Engine::CollectionSynchronizationStartDeliveryDisplayAction::Handle` | Engine::CollectionSynchronizationStartD… | Starts a product-delivery synchronization: forwards its seven arguments to the collection service and opens a progress dialog with MaxCount when the flag is clear. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278aa0` | `Engine::CollectionReloadCollectionDisplayAction::Handle` | Engine::CollectionReloadCollectionDispl… | Queues an UpdateCollection command when the collection display is reloaded. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278b50` | `Engine::CollectionRequestCollection::Handle` | Engine::CollectionRequestCollection | Initialises the collection view with the ShowUnowned option once the service reports ready. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278db0` | `Engine::CollectionRespondTickets::Handle` | Engine::CollectionRespondTickets | Replies to a tickets request with the ticket count taken from the argument. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fd2a0` | `InitializeCollectionCategoryNode` |  | Initializes a collection-category row node; this feature attribution is supported by its resource-loader caller 009fd4e0. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd390` | `ScalarDeletingDtor_009fd390` |  | Scalar deleting destructor for a collection category node. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fd9d0` | `CollectionCategoryManager::Ctor` | CollectionCategoryManager | Initializes the collection-category manager and its data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00c32a70` | `SendCollectionSelectionEvent` |  | Notifies the collection screen script of a selected collection. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c32c20` | `SendCollectionEntrySelectionEvent` |  | Notifies the collection screen script of a selected collection entry. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fd0c0` | `PopulateCollectionCategoryRows ?` |  | Refreshes rows for the collection-category data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |

## ⚪ unknown (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fd3e0` | `— ⚠reject` |  | Removes the current collection-category head node; the feature attribution comes from its caller 009fd850. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `01276b50` | `— ⚠reject` | Engine::CollectionRedeemItem | Constructor of the RedeemItem Scaleform action object. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276ca0` | `— ⚠reject` | Engine::CollectionProcessFulfillment | Constructor of the ProcessFulfillment Scaleform action object. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276df0` | `— ⚠reject` | Engine::CollectionRequestCollection | Constructor of the RequestCollection Scaleform action object. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01276f40` | `— ⚠reject` | Engine::CollectionRequestPacks | Constructor of the RequestPacks Scaleform action object. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `01277090` | `— ⚠reject` | Engine::CollectionRequestProductTypes | Constructor of the RequestProductTypes Scaleform action object. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `012771e0` | `— ⚠reject` | Engine::CollectionRequestTickets | Constructor of the RequestTickets Scaleform action object. | [[Native Map/Shards/S0665 01271690-01277340|S0665]] |
| `012782c0` | `— ⚠reject` | Engine::CollectionSynchronizationStartI… | Starts a collection synchronization by opening the progress dialog with MaxProgress set from the argument. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012783f0` | `— ⚠reject` | Engine::CollectionSynchronizationReceiv… | Updates the collection synchronization progress dialog with the received progress value. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `012789b0` | `— ⚠reject` | Engine::CollectionSynchronizationRemove… | After a collection instance removal, queues a RequestCollection command. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278c90` | `— ⚠reject` | Engine::CollectionRequestPacks | Sends a RespondPacks command reply for the packs request. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278eb0` | `— ⚠reject` | Engine::CollectionRequestCardData | Requests card data for an archetype by posting a SendCardData command. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01278ff0` | `— ⚠reject` | Engine::CollectionQueryMoz | Issues a QueryMoz command for an archetype. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01279130` | `— ⚠reject` | Engine::CollectionRequestProductTypes | Requests product types by posting a named command with a list of integer arguments. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

