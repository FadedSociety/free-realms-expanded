# System: ui.actionbar

37 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (22)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ee3d0` | `BaseClient::RefreshActionBarSlots` | BaseClient | Refreshes every action-bar slot from the client's bar data: slot text, button data and item counts. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `009cf6a0` | `InitializeActionBarLuaHandler` |  | Initializes the action-bar Flash/Lua handler. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cf7d0` | `SetActionBarVisible` |  | Shows or hides the action-bar interface through its UI handler. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cf920` | `SetActionBarButtonData` |  | Sends action slot data to the action-bar UI handler. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cfb80` | `SetActionBarItemCount` |  | Updates the number of displayed action-bar items. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cfd60` | `SetActionBarTimer` |  | Updates a timed action-bar slot and its UI countdown. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d0030` | `EnableActionBarSlot` |  | Sends the action slot enabled-state update to the UI. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d01f0` | `DisableActionBarSlot` |  | Sends the action slot disabled-state update to the UI. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d0570` | `SetActionBarTargetingOn` |  | Enables targeting mode for an action slot and notifies the UI. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d0730` | `CancelActionBarTargeting` |  | Cancels targeting mode for an action slot and notifies the UI. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d0a70` | `SetTargetingFlagForAllSlots` |  | Selects an action slot for targeting and synchronizes the action-bar UI. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a12840` | `ItemActionBar_LookupAndGate` |  | Bar lookup used by the dock/quick-item activation path; Research/Original Inventory Food and Potion Use.md cites this address as the 'bar2 lookup' that precedes slot gating. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00c48f60` | `ActionBarDataSource_GetColumnName` | ActionBarDataSource | Names the eight ActionBar data-source columns; paired with the row accessor 00c49030, these describe the nine action-bar slots exposed to the UI. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c49030` | `ActionBarDataSource_GetValue` | ActionBarDataSource | Supplies values for one of nine indexed slots. The code does not establish that the hash key is a player GUID or that the accessor is resolving a local-player entity. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c492a0` | `ActionBarDataSource::~ActionBarDataSource` | ActionBarDataSource | Non-deleting destructor of the action-bar data source. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c49420` | `ActionBarDataSource::ScalarDeletingDtor` | ActionBarDataSource | Polymorphic deletion entry for ActionBarDataSource. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00cf3b20` | `ClientActionBarDataSource::GetFieldName` | ClientActionBarDataSource | Supplies field names for action-bar datasource rows. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3bb0` | `ClientActionBarDataSource::GetCount` | ClientActionBarDataSource | Exposes the action-bar datasource item count. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3bc0` | `ClientActionBarDataSource::GetFieldValue` | ClientActionBarDataSource | Supplies per-slot values to the action-bar GUI datasource. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3d60` | `ClientActionBarDataSource::Dtor` | ClientActionBarDataSource | Destroys the action-bar datasource and its embedded base. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3dd0` | `ClientActionBarDataSource::Ctor` | ClientActionBarDataSource | Initializes the GUI datasource that reads the current action bar. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3e50` | `ClientActionBarDataSource::ScalarDeletingDtor` | ClientActionBarDataSource | Implements the MSVC scalar deleting destructor for this datasource. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d09f0` | `CancelTargetingForAllSlots` |  | Clears active targeting-cancel state across the action slots. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d0a20` | `StartTimersForOtherActionSlots` |  | Applies a timer to the other currently untimed action slots. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a125d0` | `ResolveItemStateForBar` |  | State resolution feeding the action-bar/ability path: the returned 0/5/6 codes are refusal or proceed states. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12660` | `SyncAbilitySlotWithPlayer` |  | Equality-gated ability-slot synchronisation driven by a per-frame caller. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12890` | `ItemActionBar_LookupAndDispatch` |  | Action-bar lookup variant used from the targeting path (caller 00a33760 is target-related). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a128e0` | `ItemActionBar_LookupAndActivate` |  | Action-bar lookup variant used by the activation paths (three callers in the ability/cast machinery). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12930` | `ItemActionBar_ResetSelection` |  | Action-bar lookup followed by the selected-slot reset helper; caller is state cleanup/unlink code. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12980` | `ItemActionBar_LookupAndVirtualCall` |  | Action-bar lookup variant that dispatches into the bar's own virtual interface. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a129e0` | `ItemActionBar_FindSlot` |  | Inner slot lookup used by the cast path (callers 00a35cc0/00a36a70 are the ability state machine and its target resolver). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00c48fe0` | `ActionBarDataSource_NotifyOwner` | ActionBarDataSource | Change notification hook of the embedded ActionBar subobject. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c49000` | `SetField278AndNotify` |  | Small state setter with change notification; the field it writes is the one initialised to zero by the ActionBar data source constructor 00c49360, so it belongs to that data source's slot state. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c49350` | `ActionBarDataSource_SecondaryScalarDeletingDtor` | ActionBarDataSource | Deleting-destructor thunk of the ActionBarDataSource base subobject at +0x254. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00958740` | `HideActionProgress ?` |  | Hides the action-progress display and performs a gated callback. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d03b0` | `— ⚠reject` |  | Cancels targeting mode for the selected action slot and notifies the UI. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00c49360` | `— ⚠reject` | ActionBarDataSource | Creates the action-bar data source and binds it to the inventory data source used for its item backing. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |

