# System: net.packet.reward

27 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88a70` | `RewardDeferredStars_ReadRow` |  | Reads one entry of the subtype 7 deferred-stars table; the reward findings note records that the handler sums the second word of each row. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88ad0` | `RewardBasePacket_Ctor` | RewardBasePacket | Creates the default-subtype reward-family packet object; the note records that this constructor sets base opcode 50 (0x32). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88b40` | `RewardBasePacket_CtorWithSubtype` | RewardBasePacket | Shared base constructor for the concrete reward packets: subtype 1 bundle (00b88c40), 2 non-bundled item (00b88d10), 6 deferred bank (00b88e10) and 7 granted bank (00b893a0). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88c40` | `RewardBundlePacket::RewardBundlePacket` | RewardBundlePacket | Builds the packet that carries a reward bundle to the client; its reader is 00b8a300 and the bundle body is parsed by 008e7930. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88cb0` | `RewardBundlePacket_SetByte20` | RewardBundlePacket | Sets the success/flag byte that the constructor clears and the reader 00b8a300 manipulates (byte at packet+0x20 stored from the bundle read). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88d10` | `RewardNonBundledItem::RewardNonBundledItem` | RewardNonBundledItem | Builds the packet that tells the client about a single non-bundled item reward; reader 00b89590 and item notification 00b899f0 belong to this path. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88e10` | `RewardPacketNonMemberRewardDeferred::RewardPacketNonMemberRewardDeferred` | RewardPacketNonMemberRewardDeferred | Builds the packet that carries the deferred bank deltas (subtype 6) read by 00b89600/00b892b0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88ff0` | `RewardBasePacket::ScalarDeletingDtor` | RewardBasePacket | Deleting destructor for reward-family packet objects. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89010` | `RewardBundlePacket::ScalarDeletingDtor` | RewardBundlePacket | Deleting destructor for the subtype-1 bundle packet created by 00b88c40. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89070` | `RewardNonBundledItem::ScalarDeletingDtor` | RewardNonBundledItem | Deleting destructor for the subtype-2 item packet created by 00b88d10. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89090` | `RewardPacketNonMemberRewardDeferred::ScalarDeletingDtor` | RewardPacketNonMemberRewardDeferred | Deleting destructor for the packet built by 00b88e10 and read by 00b89600. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89190` | `RewardBasePacket_ReadHeader` | RewardBasePacket | Consume the 50/0x32 base opcode plus the byte subtype that every reward-family packet starts with, as recorded in the reward findings note. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b892b0` | `RewardPacketNonMemberRewardDeferred_ReadBody` | RewardPacketNonMemberRewardDeferred | Parse the deferred stars/coins deltas; the reward findings note identifies +0x20 as the deferred XP delta and +0x24 as the coin delta and records that the handler adds them to its cached totals. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89340` | `RewardDeferredStars_ReadList` |  | Read the per-row stars table that the granted-bank handler sums (second word per row); the findings note names this address as the list reader. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b893a0` | `RewardPacketNonMemberRewardGranted::RewardPacketNonMemberRewardGranted` | RewardPacketNonMemberRewardGranted | Builds the packet that carries the granted bank amounts plus the per-row stars list (reader 00b89c00/00b894a0). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b894a0` | `RewardPacketNonMemberRewardGranted_ReadBody` | RewardPacketNonMemberRewardGranted | Parse the membership bank grant: one coin amount plus the stars rows; the findings note names this address as the subtype 7 reader. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89500` | `RewardPacketNonMemberRewardGranted::ScalarDeletingDtor` | RewardPacketNonMemberRewardGranted | Deleting destructor for the subtype-7 packet created by 00b893a0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88db0` | `RewardNonBundledItem::~RewardNonBundledItem` | RewardNonBundledItem | Release an item packet object; also called by its deleting destructor 00b89070. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88e70` | `RewardNonMemberDeferred_DestroyBase` |  | Unwind/cleanup step for subtype-6 packet temporaries and the reward dispatcher 00b8a640. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b891f0` | `RewardNonBundledItem_ReadBody` | RewardNonBundledItem | Parse the item payload of the RewardNonBundledItem packet; the entry point is 00b89590. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89410` | `RewardPacketNonMemberRewardGranted_Destroy` | RewardPacketNonMemberRewardGranted | Release a granted-bank packet; also reached from EH unwind funclets and from the reward dispatcher 00b8a640. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89590` | `RewardNonBundledItem_Read` | RewardNonBundledItem | Deserialize the item payload dispatched for subtype 2 by the reward dispatcher 00b8a640. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89600` | `RewardPacketNonMemberRewardDeferred_Read` | RewardPacketNonMemberRewardDeferred | Deserialize the deferred bank deltas dispatched for subtype 6. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89c00` | `RewardPacketNonMemberRewardGranted_Read` | RewardPacketNonMemberRewardGranted | Deserialize the membership bank grant dispatched for subtype 7. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a300` | `RewardBundlePacket_Read` | RewardBundlePacket | Deserialize the reward bundle dispatched for subtype 1; the bundle body layout is established by 008e7930 per the findings note. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88cc0` | `RewardBasePacket_DestroyBase ? ⚠rename` |  | Unwind/cleanup step used by construction failure of subtype 2 packets and by the reward dispatcher 00b8a640. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88bc0` | `— ⚠reject` | RewardBasePacket | Base cleanup for every reward-family packet; reached from the derived destructors and from eleven EH unwind funclets. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b88a70` | `RewardDeferredStars_ReadRow` | recv | RewardBasePacket |  | Hypothesis: the server sent subtype 7 with a granted-coin value followed by rows of {unresolved word, stars}; the handler sums the second word, so the original server computed the per-row star awards and the totals came from the client-side sum. |
| `00b88ad0` | `RewardBasePacket_Ctor` | recv | RewardBasePacket | 50 | Hypothesis: the zero subtype leaves the packet family opcode 50 with subtype 0, which the dispatcher 00b8a640 does not handle in its 1/2/6/7 table, so the original server most likely never sent subtype 0 and this constructor only produces a local default object. |
| `00b88b40` | `RewardBasePacket_CtorWithSubtype` | recv | RewardBasePacket | 50 |  |
| `00b88c40` | `RewardBundlePacket::RewardBundlePacket` | recv | RewardBundlePacket |  | Hypothesis: the original server sent opcode 50 subtype 1 to grant a bundle of items/currency, and the bundle object was filled by the shared bundle reader 008e7930 before this packet object was dispatched. |
| `00b88d10` | `RewardNonBundledItem::RewardNonBundledItem` | recv |  | 2 | Hypothesis: the original server sent opcode 50 subtype 2 to hand the client a single item (with icon/name fields) that the item notification helper 00b899f0 then presents. |
| `00b88e10` | `RewardPacketNonMemberRewardDeferred::RewardPacketNonMemberRewardDeferred` | recv |  | 6 | Hypothesis: the original server sent opcode 50 subtype 6 with two deltas plus an unresolved first word; the client added them to its cached bank totals (per the reward findings note), implying the server used this packet for non-member deferred accrual rather than wallet updates. |
| `00b89190` | `RewardBasePacket_ReadHeader` | recv | RewardBasePacket |  | Hypothesis: every packet in this family starts with base opcode 50 (0x32) and a one-byte subtype; the community catalogue names the base class RewardBasePacket with base_opcode 50, and the client dispatcher 00b8a640 switches on the subtype. |
| `00b892b0` | `RewardPacketNonMemberRewardDeferred_ReadBody` | recv | RewardBasePacket |  | Hypothesis: the original server sent subtype 6 whenever a non-member accrued deferred stars/coins, and the client accrued them locally into its bank display caches instead of updating the wallet. |
| `00b89340` | `RewardDeferredStars_ReadList` | recv | RewardBasePacket |  | Hypothesis: the original server computed the per-row star grants for a membership/reward panel and the client only summed them for display; totals were packet-driven, not wallet updates, per the findings note. |
| `00b893a0` | `RewardPacketNonMemberRewardGranted::RewardPacketNonMemberRewardGranted` | recv |  | 7 | Hypothesis: the original server sent subtype 7 to show the membership stars/coins reward panel, with granted coins followed by the rows the client sums for the stars total. |
| `00b894a0` | `RewardPacketNonMemberRewardGranted_ReadBody` | recv | RewardBasePacket |  | Hypothesis: the original server sent subtype 7 with the granted coin amount and the per-row star grants after the client had accrued deferred amounts, producing the membership reward panel. |
| `00b891f0` | `RewardNonBundledItem_ReadBody` | recv | RewardBasePacket |  | Hypothesis: subtype 2 carries one item id and its notification fields; the client then plays the item notification through 00b899f0 with the fixed notification type 8 recorded in the reward findings note. |
| `00b89590` | `RewardNonBundledItem_Read` | recv | RewardBasePacket |  | Hypothesis: the dispatcher passes the exact remaining body bytes, so full consumption is the normal success condition; trailing data would indicate an unexpected longer subtype-2 body from the server. |
| `00b89600` | `RewardPacketNonMemberRewardDeferred_Read` | recv | RewardBasePacket |  | Hypothesis: the client kept these two delta words cached at manager+0x3c/+0x40 and added to them on every subtype-6 packet (per the findings note), so the server could keep piling up deferred amounts without an absolute balance. |
| `00b89c00` | `RewardPacketNonMemberRewardGranted_Read` | recv | RewardBasePacket |  | Hypothesis: the dispatcher hands the whole subtype-7 body to this reader; the handler afterwards sums the row list and calls ShowXpCoinBankReward, per the reward findings note. |
| `00b8a300` | `RewardBundlePacket_Read` | recv | RewardBundlePacket |  | Hypothesis: the original server sent subtype 1 with a success byte, coins, XP, an unresolved word, a reward kind and the entry table; the client then queued a player reward event (per the dispatcher note). |
