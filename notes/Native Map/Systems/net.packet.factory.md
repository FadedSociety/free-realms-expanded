# System: net.packet.factory

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c14d40` | `FactoryPacketRequestRecipeReward::FactoryPacketRequestRecipeReward` | FactoryPacketRequestRecipeReward | Builds the request that asks for the reward of a crafting recipe (Factory family); constructed by the command 00c23fc0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c1ac60` | `FactoryPacketRequestRecipeReward::ScalarDeletingDtor` | FactoryPacketRequestRecipeReward | Performs scalar deleting destruction for the recipe-reward request packet. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c14da0` | `FactoryPacketRequestRecipeReward::~FactoryPacketRequestRecipeReward` | FactoryPacketRequestRecipeReward | Cleanup of the recipe-reward request; its callers are the command 00c23fc0 and its unwind funulet 015f7c70. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c14d40` | `FactoryPacketRequestRecipeReward::FactoryPacketRequestRecipeReward` | send |  | 14 | Hypothesis: the server resolved the recipe reward table for the referenced recipe and answered with the reward list; the payload word is filled by the command, not by this constructor. |
