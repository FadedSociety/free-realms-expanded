# System: audio.listener

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0089f020` | `SoundListenerEntity::SoundListenerEntity` | SoundListenerEntity | Builds a sound listener entity: the class publishes the SoundListenerEntity vtable on top of the Deep::Entity base used by the other entities in this range, and its only caller 008ec2a0 is the site that allocates and in… | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0089f0b0` | `SoundListenerEntity::~SoundListenerEntity` | SoundListenerEntity | Tears down a sound listener entity; there is no direct caller in the map, so it is reached through the class's virtual destructor slot, with the deleting variant 0089f110 beside it. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0089f110` | `SoundListenerEntity::ScalarDeletingDestructor` | SoundListenerEntity | Heap-delete path for a sound listener entity; the non-deleting body is 0089f0b0. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |

