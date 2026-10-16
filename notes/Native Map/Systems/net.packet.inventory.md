# System: net.packet.inventory

33 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0098ea30` | `BaseInventoryPacket::Constructor` | BaseInventoryPacket | Initializes the common base for one inventory packet object. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098ea90` | `BaseInventoryPacket::Destructor` | BaseInventoryPacket | Resets the packet object's vtable state during destruction. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098eae0` | `BaseInventoryPacket::ScalarDeletingDestructor` | BaseInventoryPacket | Provides scalar deletion for the base inventory packet. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `00bd58c0` | `InventoryPacketPreviewStyleCard::InventoryPacketPreviewStyleCard` | InventoryPacketPreviewStyleCard | Constructs the BaseInventoryPacket sub-type 11 packet InventoryPacketPreviewStyleCard (community row 42/11). | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd5920` | `BaseInventoryPacket::~BaseInventoryPacket` | BaseInventoryPacket | Destructor body of the BaseInventoryPacket family (community row 42). Its callers are two EH unwind funclets and the senders 00bda400, 00bdc060 and 00bdc420. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd5c10` | `InventoryPacketPreviewStyleCard::ScalarDeletingDtor` | InventoryPacketPreviewStyleCard | Heap-delete entry of InventoryPacketPreviewStyleCard, matching the scalar deleting shape with base vftables restored. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00c09000` | `BaseInventoryPacket_DtorBody` | BaseInventoryPacket | Common teardown of the inventory packet family objects. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c096b0` | `InventoryPacketItemRequirementRequest_ScalarDeletingDtor` | InventoryPacketItemRequirementRequest | MSVC scalar deleting destructor for the item-requirement request packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c10890` | `InventoryPacketUseStyleCard::InventoryPacketUseStyleCard` | InventoryPacketUseStyleCard | Builds the client request that uses a style card (catalogue InventoryPacketUseStyleCard, inventory channel 42 sub-opcode 10); constructed by the Lua command 00c11b10 (EquipScreen.UseStyleCard). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10940` | `InventoryPacketUseImmediateActivationItem::InventoryPacketUseImmediateActivatio…` | InventoryPacketUseImmediateActivationIt… | Builds the client request that consumes an immediately-activating item (food/potion); constructed by the Lua command 00c11bc0 (EquipScreen.UseImmediateActivationItem). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11240` | `InventoryPacketUseStyleCard::ScalarDeletingDtor` | InventoryPacketUseStyleCard | MSVC scalar deleting destructor of InventoryPacketUseStyleCard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c112b0` | `InventoryPacketUseImmediateActivationItem::ScalarDeletingDtor` | InventoryPacketUseImmediateActivationIt… | MSVC scalar deleting destructor of InventoryPacketUseImmediateActivationItem. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11320` | `InventoryPacketCheckImmediateActivationItem::ScalarDeletingDtor` | InventoryPacketCheckImmediateActivation… | MSVC scalar deleting destructor of InventoryPacketCheckImmediateActivationItem. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14720` | `InventoryPacketItemActionBarAssign::InventoryPacketItemActionBarAssign` | InventoryPacketItemActionBarAssign | Builds the request that assigns an item instance to an action-bar slot; constructed by the AssignToItemActionBar command 00c23ed0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c147e0` | `InventoryPacketMoveItemToStorage::~InventoryPacketMoveItemToStorage` | InventoryPacketMoveItemToStorage | Cleanup of the move-item-to-storage packet; its callers are the command 00c24270 and its unwind funulet 015f7d40. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14840` | `InventoryPacketMoveItemToStorage::ScalarDeletingDtor` | InventoryPacketMoveItemToStorage | MSVC scalar deleting destructor of InventoryPacketMoveItemToStorage. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c1a850` | `InventoryPacketItemActionBarAssign::ScalarDeletingDtor` | InventoryPacketItemActionBarAssign | Performs scalar deleting destruction for the inventory action-bar assignment packet. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |
| `00c20a30` | `SerializeItemActionBarAssign` |  | Serializes the InventoryPacketItemActionBarAssign payload used by the adjacent send path. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c108f0` | `InventoryPacketUseStyleCard::~InventoryPacketUseStyleCard` | InventoryPacketUseStyleCard | Cleanup of the UseStyleCard packet; its only caller is the unwind funulet 015f5d70 that guards the Lua command 00c11b10 in the same exception region. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c109a0` | `InventoryPacketUseImmediateActivationItem::~InventoryPacketUseImmediateActivati…` | InventoryPacketUseImmediateActivationIt… | Cleanup of the immediate-activation packet; only caller is the unwind funulet 015f5db0 that guards the Lua command 00c11bc0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10a50` | `InventoryPacketCheckImmediateActivationItem::~InventoryPacketCheckImmediateActi…` | InventoryPacketCheckImmediateActivation… | Cleanup of the check-immediate-activation packet; only caller is the unwind funulet 015f5df0 that guards the Lua command 00c11cb0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11600` | `WriteUseStyleCardPacketBody` |  | Serialises the one-id payload of InventoryPacketUseStyleCard; called by the send wrapper 00c11780. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11660` | `WriteUseImmediateActivationItemPacketBody` |  | Serialises the one-id payload of InventoryPacketUseImmediateActivationItem; called by the send wrapper 00c11890. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c116c0` | `WriteCheckImmediateActivationItemPacketBody` |  | Serialises the one-id payload of InventoryPacketCheckImmediateActivationItem; called by the send wrapper 00c119a0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11780` | `SendUseStyleCard` |  | Wire encoder/sender for the style-card use request; called by the EquipScreen.UseStyleCard command 00c11b10. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11890` | `SendUseImmediateActivationItem` |  | Wire encoder/sender for the item-use request; called by the EquipScreen.UseImmediateActivationItem command 00c11bc0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c119a0` | `SendCheckImmediateActivationItem` |  | Wire encoder/sender for the item-availability check; called by the EquipScreen.CheckImmediateActivationItem command 00c11cb0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14790` | `InventoryPacketItemActionBarAssign::~InventoryPacketItemActionBarAssign` | InventoryPacketItemActionBarAssign | Cleanup of the action-bar assignment packet; its callers are the command 00c23ed0 (constructor 00c14720) and its unwind funulet 015f7c40. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c08fa0` | `— ⚠reject` | InventoryPacketItemRequirementRequest | Builds the client request that asks the server to evaluate an item's requirements (used by the InventoryScreen handler 00c09d30). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09b80` | `— ⚠reject` |  | Serialises the item-requirement request body for the ClientGateway template 00c09c00. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09c00` | `— ⚠reject` |  | Sends InventoryPacketItemRequirementRequest built by the InventoryScreen handler 00c09d30 through the gateway tunnel. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c109f0` | `— ⚠reject` | InventoryPacketCheckImmediateActivation… | Builds the client request that asks whether an immediate-activation item can be used; constructed by the Lua command 00c11cb0 (EquipScreen.CheckImmediateActivationItem). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c1a8c0` | `— ⚠reject` | InventoryPacketMoveItemToStorage | Builds the inventory move-item-to-storage packet. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c10890` | `InventoryPacketUseStyleCard::InventoryPacketUseStyleCard` | send | InventoryPacketUseStyleCard | 10 | Hypothesis: the server applied the style-card item to the player's appearance and replied with the updated equipment/style state; the client request shape (42/10 with one id word serialised by 00c11780) is proven, the reply is not. |
| `00c10940` | `InventoryPacketUseImmediateActivationItem::InventoryPacketUseImmediateActivatio…` | send |  | 15 | Hypothesis: the server consumed the item, applied its effect to the player and updated the inventory; this packet's sub-opcode (15) is not among the catalogue rows (which stop at 12), so the wire packet name stays null. |
| `00c14720` | `InventoryPacketItemActionBarAssign::InventoryPacketItemActionBarAssign` | send | InventoryPacketItemActionBarAssign | 6 | Hypothesis: the original server stored the slot assignment and persisted it with the character, answering only on error; this is consistent with the client sending an absolute slot index plus the item instance word. |
| `00c20a30` | `SerializeItemActionBarAssign` | send | InventoryPacketItemActionBarAssign |  | Hypothesis: the original server applies the owned item instance to the requested zero-based action-bar slot. Depends on the packet class catalogue and the cited native request mapping; it does not establish server validation or response behavior. |
| `00c11600` | `WriteUseStyleCardPacketBody` | send | InventoryPacketUseStyleCard |  |  |
| `00c11780` | `SendUseStyleCard` | send | InventoryPacketUseStyleCard |  | Hypothesis: the original server validated and applied the style card, then updated the player's appearance through the player-update channel; the client proves the request envelope (gateway opcode 5 + inventory channel 42 sub-opcode 10) only. |
| `00c08fa0` | `— ⚠reject` | send |  | 4 | Hypothesis: the original server evaluated the character's level/membership/quest state against the item and answered with a requirement-status reply; the inference depends on the send path 00c09d30 -> 00c09c00 and the body writer 00c09b80. The community catalogue documents BaseInventoryPacket subtypes 2,3,6,8,10,11,12 but not this subtype, so the reply is unrecovered. |
| `00c109f0` | `— ⚠reject` | send |  | 16 | Hypothesis: the server answered with a yes/no availability result for the item (the client command returns 1 to Lua after queueing the request); no catalogue row covers sub-opcode 16 so the packet name stays null. |
| `00c1a8c0` | `— ⚠reject` | send |  | 17 | Hypothesis: server receives a request to move an item into storage; inferred from packet class and constructor fields only, without original server logic. |
