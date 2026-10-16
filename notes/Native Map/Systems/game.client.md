# System: game.client

65 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0070c8b0` | `ClientAccountManagerSetState` | ClientAccountManager | Account manager state transition helper. | [[Native Map/Shards/S0144 007079d0-0070c9f0|S0144]] |
| `0070c9f0` | `HandleNextPrevBackdoorChord` |  | Handles an input chord; its debug string warns about a character-deletion backdoor, but this body only transitions state and clears a field. | [[Native Map/Shards/S0144 007079d0-0070c9f0|S0144]] |
| `008b8160` | `PlayerEvent::PlayerEvent` | PlayerEvent | Initializes a player event base object. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b82a0` | `PlayerMotionEvent::PlayerMotionEvent` | PlayerMotionEvent | Initializes a player motion event. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b8330` | `PlayerMotionEvent::DtorBody` | PlayerMotionEvent | Performs PlayerMotionEvent base cleanup; the call receives the event object, not the stack-cookie value. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b8400` | `PlayerMotionEvent::ScalarDeletingDtor` | PlayerMotionEvent | Releases a PlayerMotionEvent allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008f0fe0` | `BaseClient::InitializeEnvironment` | BaseClient | Environment and logger setup called by PlayClient::Initialize. | [[Native Map/Shards/S0234 008f0d00-008f50c0|S0234]] |
| `008f12d0` | `BaseClient::OnProviderStatus` | BaseClient | Handles a provider status notification in the shared client implementation. | [[Native Map/Shards/S0234 008f0d00-008f50c0|S0234]] |
| `008f13c0` | `BaseClient::OnPublishStatus` | BaseClient | Shared client handler for a social publish-status event. | [[Native Map/Shards/S0234 008f0d00-008f50c0|S0234]] |
| `008f14c0` | `BaseClient::OnBrowseRequired` | BaseClient | Handles a browse-required notification in the shared client implementation. | [[Native Map/Shards/S0234 008f0d00-008f50c0|S0234]] |
| `00936d00` | `WaitForWorldReady` |  | Coordinates completion and failure handling for the client's world-loading barrier. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `009386f0` | `LoadZone` |  | Begins client zone setup and selects the wait-barrier or Sky.xml load branch from this+0x8f. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `0093b730` | `ReportPlayerRequest` |  | Process a player-report request. Packet delivery is conditional on the gateway; the status callback runs separately. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |

## 🟢 likely (29)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0070c7d0` | `LogAcceptCurrentDummy` |  | The function name follows its diagnostic string; caller context suggests a character-selection path, but the observed body only sets a flag and logs. | [[Native Map/Shards/S0144 007079d0-0070c9f0|S0144]] |
| `00715e30` | `ClientAccountManager::RemoveTrackedActorNode` | ClientAccountManager | ClientAccountManager::Run calls this in state 0x33 after checking a tracked object; it clears the selected-object field after this helper returns. The exact state meaning is unresolved. | [[Native Map/Shards/S0147 007140c0-00715ea0|S0147]] |
| `00934870` | `MoveAndSelectTargetControllerHandler::SelectPlayer` | MoveAndSelectTargetControllerHandler | Handles the controller path that selects a player target; the candidate callback is outside the gateway-send guard. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `0093af90` | `UpdatePositionSnapshot` |  | Coordinate position-related client updates under activity and movement gates. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `0093b340` | `ProcessPositionInput` |  | Process position-bearing input when state and modifier gates permit. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `0093bba0` | `HandleInputEvent` |  | Route eligible pointer events to the current handler and movement path. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00957530` | `ClientLoginManager::BuildAccountCharacterDetails` | ClientLoginManager | ClientLoginManager virtual slot 10 builds account character-detail callback entries from eligible supplied records. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0096aed0` | `RefreshPlayerInfoAndInventoryData` |  | Refreshes the BaseClient.PlayerInfo entry and each BaseClient.Inventory.<category>Data entry when the owner link at this+0x738 points back to this. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00a13df0` | `ClientMap_FindNodeByKey` |  | Primary keyed lookup on the large client object; the same map is read by 00a14040/00a14090/00a140e0/00a153f0 in this shard and by 008c50f0 outside it. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14040` | `ClientMap_FindNodeAndStoreField14` |  | Field setter on one record of the client's keyed table; in the packet dispatcher 009392c0 this is called on a record obtained from the same table right next to the JobLevelUp notification strings. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14090` | `ClientMap_NodeEligibleNoLimit` |  | Eligibility predicate over one record of the client's keyed table (flag byte, a blocking dword and an extra byte gated by a client-wide flag); the in-shard predicate 00a140e0 calls it for its alternative branch. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a140e0` | `ClientMap_NodeEligibleWithBudget` |  | Second eligibility predicate on the same records, adding an accumulated-budget comparison (24 + 1c versus 44) on top of the first one. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14170` | `ClientMap_LookupNestedThenGetValue` |  | Two-level keyed lookup, proving the +0x4a8 table stores records that each own a nested 4-bucket map. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14560` | `ClientList_FindArrayElementByID` |  | Two-level search over the client's element list and the per-element 0x28-byte records, returning both the record and its owner. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a147f0` | `ClientList_DestroyAllNodes` |  | Whole-list teardown of the container whose elements are ~0x3a8 bytes, used by the owner destructor at 00a1d6d0. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14b50` | `ClientMap_RemoveHeadAndClearCache` |  | Drop-the-oldest-element step of the same container whose erase routine keeps the +0x70 cache in 00a146e0 and 00a18990. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a153f0` | `ClientMap_NodeEligibleComposite` |  | Composite eligibility test used by the accepted-label caller Call00a153f0If3E0SetDefault1 (008a6970), which treats a null client pointer as the default answer 1. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a154d0` | `ClientMap_NestedLookupThenTrack` |  | Two-level lookup that then consults the 100-bucket table at this+0x358fc (via 008becc0) for the inner record; used by 00a15530/00a15590 when matching a client entry. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a15530` | `ClientMap_EntryMatchesCurrentOwner` |  | Identity test used while scanning client entries (caller 00a76a60) to decide whether an entry belongs to the currently active record. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16670` | `KeyedMap_InsertNodeAfter` |  | Re-insert path used by the update-or-insert routine 00a18990 after it removes an element with the same key. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a197e0` | `ClientPcProfileManager_Ctor` | ProfileManager<ClientPcProfile> | Initializes a client profile manager. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19870` | `ClientPcProfileManager_Dtor` | ProfileManager<ClientPcProfile> | Tears down the profile manager's list and map members. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19bf0` | `ClientPcProfileManager_ScalarDeletingDtor` | ProfileManager<ClientPcProfile> | Scalar deleting destructor for the client profile manager. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19f40` | `FindCachedKeyOrResolve` |  | Client-side keyed cache lookup. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a1d6d0` | `ClientPcData::Dtor` | ClientPcData | Tears down the aggregate per-client player-data object. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a1db70` | `ClientPcData::GetField_0x46c` | ClientPcData | Getter for an unresolved ClientPcData field. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00ce1cb0` | `GameProcessor::GetByte8` | GameProcessor | Flag getter shared by GameProcessor and VehicleRaceProcessor; game meaning not established. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1e10` | `GameProcessor::ReleaseOwnedMember` | GameProcessor | Releases the owned member at +0x14 of the game processor; called from the GameProcessor destructor path (00b9f050). Also slot [3] of VehicleRaceProcessor. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce2810` | `GameProcessor::LoadPreloadForKey` | GameProcessor | Starts the preload-file load for a keyed mini game entry and marks the processor initialised. Also slot [2] of VehicleRaceProcessor (inherited shared implementation, named for the base). | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## 🟡 speculative (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0070c9d0` | `SetAccountStateFD ?` |  | Convenience transition to account-manager state 0xfd. | [[Native Map/Shards/S0144 007079d0-0070c9f0|S0144]] |
| `0070c9e0` | `SetAccountStateFE ?` |  | Convenience transition to account-manager state 0xfe. | [[Native Map/Shards/S0144 007079d0-0070c9f0|S0144]] |
| `009552e0` | `ClientLoginManager::Virtual08 ?` | ClientLoginManager | Client login-manager virtual operation; exact semantic role is unresolved. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00955460` | `ClientLoginManager::BuildEntriesFromLinkedList ?` | ClientLoginManager | Builds a name-suggestion list supplied to the login manager. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00a18990` | `KeyedMap_UpdateOrInsert ? → KeyedMap_ReplaceOrInsert ⚠rename` |  | Replace-or-add primitive of the client's keyed container, keeping the recency order that the cached key at +0x70 tracks. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a19290` | `LookupOrRemoveClientCacheEntry ? ⚠rename` |  | Client-side cache lookup/update path; the specific cached feature is unknown. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a193c0` | `InsertClientCacheItemAndRefresh ?` |  | Client cache update path; the item domain is unknown from this body. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19450` | `InsertClientCacheSentinelAndRefresh ?` |  | Client cache update path associated with the caller; exact data meaning remains unknown. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19510` | `SetCachedState0 ?` |  | Updates one state variant in a client-side keyed table. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19570` | `SetCachedState1 ?` |  | Updates the alternate state variant in a client-side keyed table. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a195e0` | `DispatchCachedStateUpdate ?` |  | ClientUpdateStat packet handling cache helper, based on the sole caller. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19da0` | `UpdateMapAndNotifyThreeObservers ?` |  | Client state update with three observer notifications. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00c23da0` | `SetClientStatusFlag ? ⚠rename` |  | Changes a client state through the two status helpers. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## ⚪ unknown (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0070a980` | `— ⚠reject` |  | Its caller is ClientAccountManager::Run and the function references a character-login log string, but the two helper-call semantics remain uncertain. | [[Native Map/Shards/S0144 007079d0-0070c9f0|S0144]] |
| `008b8390` | `— ⚠reject` | PlayerMotionEvent | Handles the PlayerMotionEvent virtual operation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008ec2a0` | `— ⚠reject` |  | Refreshes the client actor node from the client state (position and heading inputs) each time the owning client updates. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008edd00` | `— ⚠reject` |  | Delivers a speaker-to-listener event from a base player update as a system message or world say line. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008f0d00` | `— ⚠reject` | BaseClient | Shared PlayClient/BaseClient virtual reports a low-space condition to the client UI. | [[Native Map/Shards/S0234 008f0d00-008f50c0|S0234]] |
| `009161d0` | `— ⚠reject` | BaseClient | Loads the country code, member loading-screen flag, special loading-screen id and the Steam-enabled flag into the client object. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00a15260` | `— ⚠reject` |  | Packet-dispatcher helper that drops a client record and notifies the three bound UI widgets plus one further control; called from the update dispatcher 009392c0. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a15640` | `— ⚠reject` |  | Derives a remaining/available count from three counters of the object, decremented while an entry is considered occupied. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16610` | `— ⚠reject` |  | Insertion into the same keyed table that 00a13df0/00a14040 search (node key +0xd8, chain +0xd4) and that 00a138e0 links for the after-a-node case. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a19e10` | `— ⚠reject` |  | Client-state side effects gated by a predicate. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00936d00` | `WaitForWorldReady` | send | PacketClientFinishedLoading |  |  |
| `0093b730` | `ReportPlayerRequest` | send | PacketTunneledClientPacket |  | Hypothesis: the endpoint receives and processes the player-report payload; this depends on ReportPacketReportPlayer construction and the conditional tunneled send, with original server behavior unavailable. |
| `00934870` | `MoveAndSelectTargetControllerHandler::SelectPlayer` | send | CommandPacketSelectPlayer |  |  |
