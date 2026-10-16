# System: game.gaq

66 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a090d0` | `GaqItemGroupInvite_Respond0` | GaqItemGroupInvite | First responder of the group-invite game action; status 0 selects the rejected branch (00a983b0 logs GroupInviteResponseRejected for 0 and GroupInviteResponseAccepted for 1). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a090e0` | `GaqItemGroupInvite_Respond1` | GaqItemGroupInvite | Second responder of the group-invite game action; status 1 is the accepted branch of the same response builder. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a090f0` | `GaqItemGroupInvite_ForwardToSlot0` | GaqItemGroupInvite | Slot 2 of the group-invite item vtable, an alias for the item's first action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09140` | `GaqItemTradeInvite_ForwardToSlot0` | GaqItemTradeInvite | Slot 2 of the trade-invite item vtable, an alias for the item's first action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a091e0` | `HashListMap_GaqItem64_Construct` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Initialises the 64-bucket map that holds the pending game-action items by id. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a094d0` | `GaqItemClaimItem_Respond1` | GaqItemClaimItem | Claim action of a reward game item; the callee builds a claim-items request packet, so this is the branch that asks the server to grant the item. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09550` | `GaqItemHouseInvite_Respond0` | GaqItemHouseInvite | First responder of the house-invite game action; the callee 00ac4e00 builds a BaseHousingPacket, i.e. this branch answers the housing invite. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09580` | `GaqItemHouseInvite_Respond1` | GaqItemHouseInvite | Second responder of the house-invite game action; the mirrored housing-packet builder covers the other answer. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09610` | `GaqItemGroupAnnounceEncounter_ForwardToSlot0` | GaqItemGroupAnnounceEncounter | Slot 2 of the encounter-announcement item vtable, an alias for the item's first action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09650` | `GaqItemGroupAnnounceEncounter_Respond0` | GaqItemGroupAnnounceEncounter | First responder of the encounter-announcement action; the trailing 0 is the rejected status used by the encounter join/leave path 00a979a0. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a096a0` | `GaqItemGroupAnnounceEncounter_Respond1` | GaqItemGroupAnnounceEncounter | Second responder of the encounter-announcement action (accepted status). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09710` | `GaqItemDuelRequest_ForwardToSlot0` | GaqItemDuelRequest | Slot 2 of the duel-request item vtable, an alias for the item's first action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09730` | `GaqItemDuelRequest_Respond0` | GaqItemDuelRequest | First responder of the duel-request action; the flag selects the encounter response byte written into the packet. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09750` | `GaqItemDuelRequest_Respond1` | GaqItemDuelRequest | Second responder of the duel-request action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a097b0` | `GaqItemInviteAndStartMiniGame_ForwardToSlot0` | GaqItemInviteAndStartMiniGame | Slot 2 of the mini-game invite item vtable, an alias for the item's first action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a098c0` | `HashListMap_GaqItem64_NewNode` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Virtual slot 1 of the GAQ map. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09900` | `HashListMap_GaqItem64_FreeNode` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Virtual slot 2 of the GAQ map. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09b00` | `GaqItemGuildInvite_Construct` | GaqItemGuildInvite | Creates the pending guild-invite item shown to the player; the guild-invite creator 00a0b5b0 builds it through this constructor. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a160` | `HashListMap_GaqItem64_Destruct` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Destructor of the GAQ item map, reached from exception-unwind funclets. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a190` | `HashListMap_GaqItem64_ScalarDeletingDtor` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Slot 0 of the GAQ item map vtable. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a1f0` | `HashListMap_GaqItem64_Destruct_B` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Destructor used by 00a0a3b0 (the HashList scalar delete) and by the manager teardown 00a0a600. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a330` | `HashListMap_GaqItem64_Construct_B` | SoeUtil::HashListMap<int,GaqItem*,64,-1> | Builds the GAQ item map inside the map-owner object constructed by 00a0a5a0. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a3b0` | `HashList_GaqItem_ScalarDeletingDtor` | SoeUtil::HashList<GaqItem*,64,-1> | Slot 0 of the HashList<GaqItem*,64,-1> vtable. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a410` | `GaqItemFriendInvite_Construct` | GaqItemFriendInvite | Creates the pending friend-invite item; the friend-invite creator 00a0ba90 builds it through this constructor. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0c170` | `GaqManager_AddGroupInvite` |  | Presents a group invitation as an accept/decline game action; its auto-refuse branch invokes slot 0, which forwards status 0 to the group-invite response routine. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## 🟢 likely (33)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a09120` | `GaqItemTradeInvite_Respond0 ⚠downgrade` | GaqItemTradeInvite | First responder of the trade-invite game action; the 0/1 status mirrors the reject/accept convention of the other invite items. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09130` | `GaqItemTradeInvite_Respond1 ⚠downgrade` | GaqItemTradeInvite | Second responder of the trade-invite game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a097d0` | `GaqItemInviteAndStartMiniGame_Respond0 ⚠downgrade` | GaqItemInviteAndStartMiniGame | First responder of the friend-invite-to-game action, which forwards the mini-game invitation parameters with status 0. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09800` | `GaqItemInviteAndStartMiniGame_Respond1 ⚠downgrade` | GaqItemInviteAndStartMiniGame | Second responder of the friend-invite-to-game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09840` | `HashListMap_GaqItem64_RemoveNode` |  | Removal routine used by the GAQ map destructors and the manager teardown 00a0a600. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a09ad0` | `GaqItemMap_Clear` |  | Clear routine for the GAQ item map. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a080` | `GaqItem_DestructMembers` |  | Destructor body of the object stored in the GAQ item map (its scalar deleting destructor 00a0a260 calls this and then frees); the class binding comes from that caller chain. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a260` | `ScalarDelete_a0a260` |  | Deleting destructor for the GAQ item payload stored in the item map. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a5a0` | `GaqItemMapOwner_Construct` |  | Initialises the small owner object that wraps the GAQ item map (allocated as 0x118 bytes by 00a0a680). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a680` | `GaqItemMapOwner_Create` |  | Creates the GAQ item-map owner during client bootstrap (called from 0092b170). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a6f0` | `ScalarDelete_a0a6f0` |  | Deleting destructor of the GAQ owner object. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a960` | `GaqItemMap_InsertGeneric` |  | Insert emission used by the generic game-action creator 00a0b1f0; the twelve copies in this range differ only by their caller. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0aa00` | `GaqItemMap_InsertGuildRenameRequest` |  | Insert used by the guild-rename game-action creator 00a0b380. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0aaa0` | `GaqItemMap_InsertGuildInvite` |  | Insert used by the guild-invite creator 00a0b5b0 (its own dedicated copy of the insert). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0ab40` | `GaqItemMap_InsertHouseInvite` |  | Insert used by the house-invite creator 00a0b830. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0abe0` | `GaqItemMap_InsertFriendInvite` |  | Insert used by the friend-invite creator 00a0ba90. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0ac80` | `GaqItemMap_InsertClaimItem` |  | Insert used by the claim-item creator 00a0e090 (outside this shard), whose item class is GaqItemClaimItem. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0ad20` | `GaqItemMap_InsertGenericNoResponse` |  | Shared insert used by all the notification-style creators that build a GaqGenericNoResponse item (00a0bdc0, 00a0bf80, 00a0c3a0 and further callers outside this shard). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0adc0` | `GaqItemMap_InsertGroupInvite` |  | Insert used by the group-invite creator 00a0c170. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0ae60` | `GaqItemMap_InsertTradeInvite` |  | Insert used by the trade-invite creator 00a0d320 (GAQTradeInvite handler, item class GaqItemTradeInvite). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0af00` | `GaqItemMap_InsertEncounterInvite` |  | Insert used by the encounter-invite creator 00a0d550, whose strings name the GAQEncounterInvite handler and the GaqItemGroupAnnounceEncounter item class. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0afa0` | `GaqItemMap_InsertDuelRequest` |  | Insert used by the duel-request creator 00a0d840 (GAQDuelInvite handler, item class GaqItemDuelRequest). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0b040` | `GaqItemMap_InsertInviteAndStartMiniGame` |  | Insert used by the mini-game invite creator 00a0dc20 (GAQInviteAndStartMiniGame handler). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0b180` | `GaqItemMapOwner_Destroy` |  | Class teardown body called by the compiler-generated scalar deleting destructor 008a6000; releases the GAQ owner object and its item map. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0b5b0` | `GaqManager_AddGuildInvite ⚠downgrade` |  | Presents an incoming guild invitation as a game action; the auto-refuse branch invokes the new item virtual slot 0 before destroying and freeing it. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bd50` | `GaqManager_AddFriendInviteToGame_Entry` |  | Entry thunk called by the FriendInviteToGame network handler; it forwards to the generic action creator and does not register an action by itself. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bd60` | `Wrapper_00a0bd60 → GaqManager_AddGuildRenameRequest_Entry` |  | Entry point used by the caller 00b1ee10 to create the guild-rename game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bd70` | `Wrapper_00a0bd70 → GaqManager_AddGuildInvite_Entry` |  | Entry point used by the caller 00b1e7e0 to create the guild-invite game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bd80` | `Wrapper_00a0bd80 → GaqManager_AddFriendInvite_Entry` |  | Entry point used by the caller 00aa2560 to create the friend-invite game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bdb0` | `Wrapper_00a0bdb0 → GaqManager_AddHouseInvite_Entry` |  | Entry point used by the caller 00abd510 to create the house-invite game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bf80` | `GaqManager_AddGenericNotification ⚠downgrade` |  | Generic notification creator used by the caller 00a0e020 for popups that carry two text parameters. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0c390` | `Wrapper_00a0c390 → GaqManager_AddGroupInvite_Entry` |  | Entry point used by the caller 00a87050 to create the group-invite game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00b20420` | `AdjustThis8TailJump_0099f000` |  | This-adjusting thunk used by caller 00a42470 while matching nudge records. Feature attribution comes from the caller; this function only forwards. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a09a80` | `— ⚠reject` |  | Sweep over the pending game-action items that stops at the first item matching one of the guard predicates (used by the frame-step entry 00a0a070). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a070` | `— ⚠reject` |  | Frame-step entry point used by 0092efe0 to run the game-action guard sweep. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a600` | `— ⚠reject` |  | Destructor body of the GAQ owner object; the scalar deleting destructor 00a0a6f0 and the manager destroy 00a0b180 drive it. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0b1f0` | `— ⚠reject` |  | Shared creator behind the simple game-action popups; 00a0bd50 forwards the incoming network arguments to it. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0b380` | `— ⚠reject` |  | Shows the pending guild-rename request popup and wires its rejected-notification path. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0b830` | `— ⚠reject` |  | Presents an incoming house invitation as an accept/decline game action. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0ba90` | `— ⚠reject` |  | Presents an incoming friend invitation as a game action with its extra context values. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0bdc0` | `— ⚠reject` |  | Shows a text popup in the game-action queue without any accept/decline response (the item class has no responder bodies). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a09550` | `GaqItemHouseInvite_Respond0` | send | BaseHousingPacket |  | Hypothesis: the original server received one of the two housing-packet answers (family 127, catalogue BaseHousingPacket) that the responder pair builds through 00ac4e00/00ac3fe0; which slot sends accept is not proven here. |
| `00a09580` | `GaqItemHouseInvite_Respond1` | send | BaseHousingPacket |  | Hypothesis: 00ac3fe0 builds the complementary housing answer (family 127, catalogue BaseHousingPacket) to 00ac4e00; the accept/decline mapping rests on that callee pair and is not proven here. |
| `00a09730` | `GaqItemDuelRequest_Respond0` | send | BaseEncounterPacket |  | Hypothesis: the original server received one of the two encounter answers (family 41, catalogue BaseEncounterPacket) that the duel responder pair builds through 00a97c10, with the response byte 0 from this slot and 1 from the other. |
| `00a09750` | `GaqItemDuelRequest_Respond1` | send | BaseEncounterPacket |  | Hypothesis: the original server received the complementary encounter answer (family 41, catalogue BaseEncounterPacket) with response byte 1 built by 00a97c10; the accept/decline meaning of the byte is not proven here. |
