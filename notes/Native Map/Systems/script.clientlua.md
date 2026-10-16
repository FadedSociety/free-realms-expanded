# System: script.clientlua

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ed9c0` | `ChatHandlerStartChatBind` |  | Checks that the chat handler exposes a StartChat function and reports a problem when it is missing. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `00ce4c30` | `AchievementDataSource::SetTotalPoints` | AchievementDataSource | Recomputes the achievement total points and passes them to the Lua handler AchievementsHandler:SetTotalPoints. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007121e0` | `TaggedValueSet` |  | Assignment of a tagged variant from a source value. Element layout per the neighbour labels 0070f0e0 and 00711960 (ClientLuaParam), not proven by this function alone. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713b70` | `TaggedValueCtorFrom` |  | Initialises a fresh 0xa0-style tagged element from a source value; the single caller 009797a0 is unlabelled. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |
| `00713cf0` | `TaggedArray0xA0_Reallocate` |  | Grows an array of 0xa0-byte tagged elements; called from 00709410 and 00cb7270. | [[Native Map/Shards/S0146 00712020-00714060|S0146]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cb6da0` | `IStringClientLuaArrayCtor ? → NotificationQueue::Entry::Ctor` |  | Stores an IString<char> vtable at param_1[0], sets param_1[1] to the empty-string static and clears [2],[3]; calls 00402ce0 with param_2; stores the Array<ClientLuaParam,0,1> vtable at [4], clears [5..7] (base, count, c… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb6e60` | `CtorWrapperDerefArgs ? → NotificationQueue::Entry::CtorForwarder` |  | Thiscall wrapper: ECX is the object to construct; dereferences the four pointer arguments (*param_2..*param_5), calls 00cb6da0 with ECX unchanged and those four values, returns the object pointer in EAX (ret 0x10). | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

