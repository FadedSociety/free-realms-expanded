# System: engine.string

12 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ece50` | `BaseClient::LookupCodeStringWithArgs` | BaseClient | Client code-string table lookup: key to localized text with a fallback for missing keys, shared by BaseClient and PlayClient. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008ed320` | `ResolveStringIdOrPlaceholder` |  | Resolves a numeric string id to display text, using a placeholder when the id is unknown. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009898a0` | `IStringMemberDeletingDestructor` |  | Scalar deleting wrapper around the IString-member cleanup routine. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `009898c0` | `IStringMemberDeletingDestructor` |  | Scalar deleting wrapper around the second IString-member cleanup routine. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b3cff0` | `FormatAndTransformTextBuffers ? ⚠rename` |  | Builds the text values used by the in-game-purchase configuration path. | [[Native Map/Shards/S0342 00b39c10-00b3f310|S0342]] |
| `00be81a0` | `ScalarDeleteWrap_00be8120 ?` |  | Calls 00be8120 (IString<unsigned_short> release body) on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ed220` | `— ⚠reject` | BaseClient | Client code-string lookup by numeric id with a log on miss. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `0098deb0` | `— ⚠reject` |  | Provides the common wide-string assignment operation used by UI field getters. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098e1f0` | `— ⚠reject` |  | Formats a 64-bit unsigned value into a wide output string. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098e220` | `— ⚠reject` |  | Formats one scalar value into a wide output string. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `009a0c10` | `— ⚠reject` |  | The byte comparison order is mechanically established by the raw return sequence; the strings? domain is unknown. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00be8120` | `— ⚠reject` | SoeUtil::IString<unsigned_short> | Destructor body for the wide string class: drops one reference to its shared buffer. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |

