# System: game.actionbar

30 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a7e40` | `RunActionBarMode` |  | Mode runner of the action-bar activation path: mode 1 -> 009cf430 (job toolbar activation, per Native Combat Controls), mode 2 -> 00a12840 (bar-2 lookup, per Original Inventory Food and Potion Use), mode 3 -> 009aaeb0 (… | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a7eb0` | `DispatchActionBarSlot` |  | Native action-bar activation dispatcher: ActionBarSlot1 dispatches through 008a7eb0 (per Native Combat Controls), choosing mode 4 for flash minigame types via 008a5850, mode 3 for other active minigames through FUN_009a… | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `009cf430` | `DispatchActionBarCastOrFailureReason` |  | Handles action-bar ability dispatch failures and reports their reason to Lua. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a16d50` | `Client_GetOrCreateActionBar` |  | Lazy factory of the two action bars the client owns (ability bar and item bar), keyed by bar type, with an explicit invalid-type fallback object. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00c44160` | `ClientActionBar_ActivateSlot` |  | The ability slot gate on the action bars: explain why a slot cannot be used (status effects, mana/level, recast timer, disabled) and otherwise start the ability; this is the function the vault notes call the slot gating… | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c453d0` | `BaseActionBar_Destruct` | BaseActionBar | Tear down the action bar's slot storage. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c454b0` | `BaseActionBar_Construct` | BaseActionBar | Build the base action bar subobject of ClientActionBar. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45510` | `ClientActionBar_Construct` | ClientActionBar | Build the client action bar with its data source; called by the minigame-manager constructor 009bc410 and by 00a16d50. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c455f0` | `ClientActionBar_Destruct` | ClientActionBar | Tear down the client action bar and its data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## 🟢 likely (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ab6d0` | `ActionBarSlotInitAndStore60` |  | Uses the ActionBarSlot initializer 0101cd80, then stores an incoming dword at +0x60. The wrapper does not establish the owner type or field meaning. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `00c43a90` | `FieldGetter_0x3ac` |  | Accessor used by the action-bar data source path 00cf3bc0 to read which slot is selected. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43c70` | `SlotCheck_ForwardToGameVtable` |  | Ask the game/player-side interface whether a given action-bar slot entry is currently valid (used by the action-bar data source path 00cf3bc0). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43d20` | `Player_CanCastStatusCheck` |  | Status-effect gate used by the activation path 00c44160 and by 00c43fd0 to decide whether the player may cast an ability. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43d80` | `ClientActionBar_SelectSlot` |  | Selection change on an action bar (used by the input paths 009aaed0 and 00a128e0). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43e00` | `ClientActionBar_ClearSelection` |  | Deselect the current action-bar slot (companion of ClientActionBar_SelectSlot). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43fa0` | `ActionBar_OnOwnerStateReset` |  | Bulk slot-state reset performed when the owning window refreshes; driven from the minigame event paths 009bef50/009bf190. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43fd0` | `ActionBarSlot_IsUsable` |  | Combined usability predicate applied by the activation gate 00c44160 and by the action-bar data source 00cf3bc0. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c440d0` | `ActionBar_NotifyAllSlots` |  | Broadcast one notification code to all slots of an action bar (used by callers that repaint every slot). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c44440` | `ActionBar_NotifySlotsCode3_A` |  | Notify every slot of a code-3 change (slot data changed) on an action bar; one of two byte-identical twins in this shard. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c44470` | `ActionBar_NotifySlotsCode3_B` |  | Second entry point of the same code-3 slot broadcast (likely two separate forwarding slots); kept as a separate identical function in the binary. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45490` | `BaseActionBar_ScalarDeletingDtor` | BaseActionBar | Scalar deleting destructor of BaseActionBar. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45680` | `ClientActionBar_ScalarDeletingDtor` | ClientActionBar | Scalar deleting destructor of ClientActionBar. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c44070` | `ActionBar_NotifySlotAvailability ? ⚠downgrade` |  | Refresh the presentation (code 4) of the affected action-bar slot after a timer/quantity change; driven from the input paths 009aaec0/00a12890. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d08f0` | `— ⚠reject` |  | Reconciles action-bar slot availability with current combat status and mana. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a326a0` | `— ⚠reject` |  | Sends the HandlerActionBar:setTimer call carrying a 64-bit timer value. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00c43bf0` | `— ⚠reject` |  | Copy-assign one ActionBarSlot payload onto a slot node; invoked by the BaseActionBar/ClientActionBar set-slot methods 00c445a0 and 00c44600. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43cd0` | `— ⚠reject` |  | Level/requirement gate applied before an action-bar slot can be activated (called from 00c44160 and 00c43fd0). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c445a0` | `— ⚠reject` | BaseActionBar | Receiver-side update of one action-bar slot for the base action bar (server action-bar data lands here). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c44600` | `— ⚠reject` | ClientActionBar | Apply one action-bar slot update and repaint only the aspects that changed. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45450` | `— ⚠reject` |  | Advance or re-base the per-slot 64-bit timers of an action bar (e.g. on a time delta). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

