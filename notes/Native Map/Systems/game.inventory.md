# System: game.inventory

31 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0098f7e0` | `ClientItemRootDataSource::ScalarDeletingDtor` | ClientItemRootDataSource | Scalar deleting destructor for the RTTI class. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `0098fb10` | `PlayerItemManagerCtor` | PlayerItemManager<ClientItem> | Initializes the generic player item manager base used by the client inventory manager. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009912f0` | `ClientPlayerItemManager::RefreshDerivedState` | ClientPlayerItemManager | Refreshes client-facing inventory-derived properties after item state changes. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00991430` | `ClientPlayerItemManager::Refresh` | ClientPlayerItemManager | Virtual slot 3 forwards to the manager refresh routine at 009912f0. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00991690` | `RemoveItemAndRefreshNamePermission` |  | Removes an inventory item and updates the name-change permission derived property. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00991860` | `PlayerItemManagerDtor` | PlayerItemManager<ClientItem> | Destructor for the generic player item manager base. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009918b0` | `PlayerItemManagerScalarDeletingDtor` | PlayerItemManager<ClientItem> | Scalar deleting destructor for the player item manager base. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009918d0` | `ClientPlayerItemManager::Ctor` | ClientPlayerItemManager | Constructs the client inventory item manager and its UI data-source graph. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009921a0` | `ClientPlayerItemManager::Refresh` | ClientPlayerItemManager | Virtual slot 1 forwards to the same manager refresh routine at 009912f0 as slot 3 at 00991430. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009921c0` | `ClientPlayerItemManagerDtor` | ClientPlayerItemManager | Destroys the client inventory manager and its owned data-source graph. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009924e0` | `ClientPlayerItemManagerScalarDeletingDtor` | ClientPlayerItemManager | Scalar deleting destructor for the client player item manager. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00c23ed0` | `AssignToItemActionBar` |  | Assigns an owned item instance to an action-bar slot. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdaae0` | `NodeDefinitionUsabilityResultByGuid` |  | Looks up a node by guid and returns the result produced by 00a15690; the state values and their meaning are not established here. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdab30` | `NodeFlagAndKindPredicate` |  | A three-result node flag/kind predicate called by equip and item-use checks; the meaning of the kind values is unresolved. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdac10` | `InventoryNodeRemainingCount` |  | Computes how many copies of an item the player still has available, used by 00bdac80 to take the minimum over a group. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdae10` | `InventoryNodePlayerEligibilityPredicate` |  | Checks a node field against a player field and then applies the node predicate; callers include equip and preview/use checks. The compared player field is not identified as a tier in this function. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdaf00` | `ItemGroupUsabilityAll` |  | All-items-usable gate that decides whether an item-set action may proceed; used by 00bdc420 and the UI entry points. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdbe30` | `EquipLightsaber_ByGuid` |  | Equip a lightsaber colour/blade choice from the UI; caller 00c10b60 passes the item guid and a second id read from the Lua arguments. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdbf90` | `EquipItemGroup_ById` |  | Equip an item group chosen in the UI; the Lua wrapper 00c10bb0 is the only caller. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdc060` | `UseItemSelectedInUi` |  | Item-use entry point driven from the UI: it decides between the generic use, the style-card preview and the mount activation paths. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00c24270` | `MoveItemToStorage` |  | Requests moving an item into storage. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c24370` | `RequestStorageCheck` |  | Requests a storage check. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdc3c0` | `ClearPreviewAndFlushMount ?` |  | Clears a preview and flushes a pending mount after several helper calls. The player-byte gate and the type-0x15 branch are observed, but their broader presentation meaning is unknown. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdf150` | `ItemProducerVisible ? ⚠rename` |  | Visibility/eligibility predicate used by 00cea650 and 00cea720 when they rebuild the activity lists. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00991440` | `— ⚠reject` | ClientPlayerItemManager | Handles item-manager changes that enable name-change permissions. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00bdac80` | `— ⚠reject` |  | Answers how many copies of the whole item set the player can still use, by taking the tightest single-item availability. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdad00` | `— ⚠reject` |  | Queries each listed node through 00a15690 and returns 4 on a missing node or a queried result of 4; the caller uses this as a group-level result. The state values are not named here. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdadb0` | `— ⚠reject` |  | Tests whether any item of a set is currently usable; feeds the equip/use gates. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdbc50` | `— ⚠reject` |  | Implements the equip-screen selection: it resolves the requested item definition, applies the member-only rules and sends either the equip-by-guid or the unequip request. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdc420` | `— ⚠reject` |  | Applies a whole item group chosen in the UI (for example a style-card group previewed item by item, or a mount group that queues a spawn). | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdc7f0` | `— ⚠reject` |  | Small wrapper that resets the item UI state; no caller is recorded in the map. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c23ed0` | `AssignToItemActionBar` | send | InventoryPacketItemActionBarAssign |  | Hypothesis: the server records the item-instance ID in the requested action-bar slot. Depends on the native binding and packet mapping in the cited note; validation and response semantics remain unknown. |
| `00bdbe30` | `EquipLightsaber_ByGuid` | send | BaseInventoryPacket |  | Hypothesis: a server might apply the lightsaber equip request after checking its own state. This depends on the outgoing BaseInventoryPacket path carrying the guid, profile id and selected value; server validation is unknown. |
| `00bdbf90` | `EquipItemGroup_ById` | send | BaseInventoryPacket |  | Hypothesis: a server might interpret the item-group equip request and apply its group. This depends on the outgoing BaseInventoryPacket path; server expansion and validation are unknown. |
| `00bdc060` | `UseItemSelectedInUi` | send | InventoryPacketPreviewStyleCard |  | Hypothesis: a server might process an item-use request and return resulting state. This depends on the outgoing packet path identified by callers; behavior for other item kinds is not established by this function. |
| `00bdbc50` | `— ⚠reject` | send | InventoryPacketEquipByGuid |  | Hypothesis: a server might apply the equip-by-guid or equipped-remove request after its own validation. This depends on the two outgoing packet paths in this function; client warnings do not prove server-side revalidation or member rules. |
| `00bdbc50` | `— ⚠reject` | send | InventoryPacketEquippedRemove |  | Hypothesis: a server might apply the equip-by-guid or equipped-remove request after its own validation. This depends on the two outgoing packet paths in this function; client warnings do not prove server-side revalidation or member rules. |
| `00bdc420` | `— ⚠reject` | send | InventoryPacketPreviewStyleCard |  | Hypothesis: a server might process preview requests and the selected group action after its own validation. This depends on outgoing request paths from this function; group effects and validation are not established. |
