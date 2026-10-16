# Class: FactoryPacketRequestRecipeReward

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14d40` | `FactoryPacketRequestRecipeReward::FactoryPacketRequestRecipeReward` | ✅ | ctor | Constructor: calls the factory base constructor FUN_009d90d0 with sub-opcode 0xe (14), stores FactoryPacketRequestRecipeReward::vftable at *this and zeroes the payload word at +0xc. |
| `00c14da0` | `FactoryPacketRequestRecipeReward::~FactoryPacketRequestRecipeReward` | 🟢 | dtor | Destructor body: stores BaseFactoryPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c1ac60` | `FactoryPacketRequestRecipeReward::ScalarDeletingDtor` | ✅ | dtor | Resets the inherited factory and packet vptrs, then frees the allocation when the deleting flag bit is set. |
