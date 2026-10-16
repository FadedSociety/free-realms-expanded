# System: engine.event

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b3040` | `GameEventSequencer::GameEventSequencer` | GameEventSequencer | Initializes GameEventSequencer: five dwords at +4 through +0x14, a 0x80-byte block at +0x18, and dwords at +0x98/+0x9c. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b56b80` | `BuildSpeakerListenerEvent` |  | Prepares a speaker/listener event for a proxied actor; caller context is pet handling, while event semantics remain open. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |

