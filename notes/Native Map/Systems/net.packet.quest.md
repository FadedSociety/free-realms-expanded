# System: net.packet.quest

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a86340` | `BaseQuestPacketCtor` | BaseQuestPacket | Common constructor run by all 17 quest sub-packet constructors (QuestAdd, QuestComplete, QuestObjective*, QuestEnd and the client's QuestReply/QuestEndReply/QuestStartBreadcrumb sent by 00a97d70/00a97ef0/00a97f90). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a864c0` | `QuestEndReplyPacketCtor` | QuestEndReplyPacket | Builds the client's reply to a quest-end request before it is sent by 00a97ef0. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a87ba0` | `BaseQuestPacket::ScalarDeletingDtor` | BaseQuestPacket | Releases a quest packet object when its owner drops it; memory management only, no quest rule. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87c00` | `QuestReplyPacket::ScalarDeletingDtor` | QuestReplyPacket | Releases a quest reply packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87c60` | `QuestEndReplyPacket::ScalarDeletingDtor` | QuestEndReplyPacket | Releases a quest end reply packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87cc0` | `QuestStartBreadcrumbPacket::ScalarDeletingDtor` | QuestStartBreadcrumbPacket | Releases a quest breadcrumb packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00c144f0` | `ProgressiveQuestRequestStartQuestPacket::ProgressiveQuestRequestStartQuestPacket` | ProgressiveQuestRequestStartQuestPacket | Builds the request that starts a progressive quest for a quest/step pair; constructed by the command 00c24400. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14670` | `ProgressiveQuestRequestClaimPrizePacket::ProgressiveQuestRequestClaimPrizePacket` | ProgressiveQuestRequestClaimPrizePacket | Builds the request that claims the progressive-quest prize; constructed by the command 00c245a0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a863b0` | `BaseQuestPacketDtorBody` | BaseQuestPacket | Compiler-split destructor body of the BaseQuestPacket family (the body only resets to the base vtable because no member needs destruction). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86400` | `QuestReplyPacketCtor ⚠downgrade` | QuestReplyPacket | Builds the client's reply to a quest offer/update (accept/decline plus a selector value) before it is sent by 00a97d70. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00c144a0` | `ProgressiveQuestRequestStartQuestPacket::~ProgressiveQuestRequestStartQuestPack…` | ProgressiveQuestRequestStartQuestPacket | Cleanup of the start-quest request; its callers are the command 00c24400 (which constructs the packet through 00c144f0) and that command's unwind funulet 015f7db0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14560` | `ProgressiveQuestRequestClaimSlotPacket::~ProgressiveQuestRequestClaimSlotPacket` | ProgressiveQuestRequestClaimSlotPacket | Cleanup of the claim-slot request; its callers are the command 00c244d0 (constructor 00c145b0) and its unwind funulet 015f7de0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c14620` | `ProgressiveQuestRequestClaimPrizePacket::~ProgressiveQuestRequestClaimPrizePack…` | ProgressiveQuestRequestClaimPrizePacket | Cleanup of the claim-prize request; its callers are the command 00c245a0 (constructor 00c14670) and its unwind funulet 015f7e10. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a86580` | `— ⚠reject` | QuestStartBreadcrumbPacket | Builds the packet that starts a quest breadcrumb (tracked objective pointer) before it is sent by 00a97f90. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00c145b0` | `— ⚠reject` | ProgressiveQuestRequestClaimSlotPacket | Builds the request that claims a progressive-quest reward slot; constructed by the command 00c244d0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a86340` | `BaseQuestPacketCtor` | send |  | 49 | Hypothesis: the original server pushed quest state to the client as class-49 packets whose second u16 is the subtype (1 QuestInfo, 2 QuestReply, 3 QuestAdd, 4 QuestComplete, 5 QuestFailed, 6 QuestAbandoned, 7/8/9/10/11 objective events, 12 CompletedQuestCount, 13 QuestEnd, 14 QuestEndReply, 15 QuestStartBreadcrumb, 16 QuestEndBlocked per the OSFR packet table); the client's own quest requests reu… |
| `00a86340` | `BaseQuestPacketCtor` | recv |  | 49 | Hypothesis: the original server pushed quest state to the client as class-49 packets whose second u16 is the subtype (1 QuestInfo, 2 QuestReply, 3 QuestAdd, 4 QuestComplete, 5 QuestFailed, 6 QuestAbandoned, 7/8/9/10/11 objective events, 12 CompletedQuestCount, 13 QuestEnd, 14 QuestEndReply, 15 QuestStartBreadcrumb, 16 QuestEndBlocked per the OSFR packet table); the client's own quest requests reu… |
| `00a864c0` | `QuestEndReplyPacketCtor` | send |  | 0xe | Hypothesis: the client confirmed the end of a quest with class 49 sub-opcode 14 carrying a quest id; the original server then finalized the quest rewards. Depends on 00a97ef0 and on the absent server-side quest bookkeeping. |
| `00c144f0` | `ProgressiveQuestRequestStartQuestPacket::ProgressiveQuestRequestStartQuestPacket` | send |  | 2 | Hypothesis: on this request the server started the progressive-quest chain step identified by the two payload words and pushed the quest state (the client's own quest-binding code lives outside this shard); the payload semantics (quest id, slot/step) are not proven here. |
| `00c14670` | `ProgressiveQuestRequestClaimPrizePacket::ProgressiveQuestRequestClaimPrizePacket` | send |  | 4 | Hypothesis: the server granted the final prize of the progressive-quest chain and answered with the reward/updated quest state; the two payload words are presumably quest id and prize index, which is not proven here. |
| `00a86400` | `QuestReplyPacketCtor ⚠downgrade` | send |  | 2 | Hypothesis: the client answered a quest with class 49 sub-opcode 2 carrying a quest id and a byte selection; the original server applied the answer and replied with the quest-state packets. Depends on 00a97d70 and on the server-side quest state machine, which is not present in this binary. |
| `00a86580` | `— ⚠reject` | send |  | 0xf | Hypothesis: the client asked the server to start a breadcrumb/tracked-quest step with class 49 sub-opcode 15 plus a quest handle; the original server replied with objective state. Depends on 00a97f90 and on the quest objective handlers outside this shard. |
| `00c145b0` | `— ⚠reject` | send |  | 3 | Hypothesis: the server reserved/claimed the indicated slot in the progressive-quest reward list; the payload words likely carry the quest id and slot index, but that naming is not proven here. |
