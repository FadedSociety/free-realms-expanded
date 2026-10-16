# System: game.player_event

24 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae20d0` | `PlayerHitpointEventBaseDtor` | PlayerHitpointEvent | Base destructor body for PlayerHitpointEvent. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2130` | `PlayerHitpointDeltaEventBaseDtor` | PlayerHitpointDeltaEvent | Base destructor body for PlayerHitpointDeltaEvent. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2190` | `PlayerHitpointEventCtor` | PlayerHitpointEvent | Creates a player hitpoint event instance. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2260` | `PlayerHitpointEvent::ScalarDeletingDtor` | PlayerHitpointEvent | Deleting destructor for PlayerHitpointEvent. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2350` | `PlayerHitpointDeltaEvent::Apply` | PlayerHitpointDeltaEvent | Applies a hitpoint delta event and updates related context state. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00957ef0` | `PlayerEffectEvent::ScalarDeletingDtor` | PlayerEffectEvent | Scalar deleting destructor for PlayerEffectEvent. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |

## 🟡 speculative (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00957df0` | `PlayerEffectEventDtor ?` | PlayerEffectEvent | Restores PlayerEffectEvent and embedded interface vtables, releases the owned interface pointer when non-null, restores PlayerEvent base vtable, and calls base cleanup. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957e90` | `PlayerEffectEvent::IsFinished ?` | PlayerEffectEvent | Calls the embedded object's virtual slot 1; returns true when absent or when its field at +0x1c equals -1. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00957eb0` | `PlayerEffectEvent::ApplyEmbeddedEffect ?` | PlayerEffectEvent | If an embedded object exists and its virtual slot 5 returns true, calls its slots 2 and then the event object's slot 3 with argument one. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959a30` | `PlayerEffectEvent::DeletingDtorAdjustor ? → PlayerEffectEvent::ScalarDeletingDt…` | PlayerEffectEvent | Subtracts 0x50 from ECX and jumps to 00957ef0; bytes identify an adjustor thunk. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959bf0` | `PlayerAnimationEventCtor ?` | PlayerAnimationEvent | Calls PlayerEvent base setup, installs PlayerAnimationEvent vtable, and copies three dwords and three bytes from constructor arguments into fields 0x50-0x5e. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959ca0` | `PlayerAnimationEventDtor ?` | PlayerAnimationEvent | Restores PlayerAnimationEvent then PlayerEvent vtables and invokes base event cleanup. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959d20` | `PlayerAnimationEvent::ScalarDeletingDtor ?` | PlayerAnimationEvent | Calls the event destructor and frees the allocation when deletion flag bit 0 is set. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959d40` | `PlayerRequestHeadShotEventCtor ?` | PlayerRequestHeadShotEvent | Calls a base-event initializer, installs PlayerRequestHeadShotEvent vtable, and stores constructor arguments at fields 0x48 and 0x4c. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959dc0` | `PlayerRequestHeadShotEvent::ScalarDeletingDtor ?` | PlayerRequestHeadShotEvent | Calls 00bf6840 and frees this object when deletion flag bit 0 is set. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959e50` | `PlayerFadeEventCtor ?` | PlayerFadeEvent | Calls PlayerEvent base setup, installs PlayerFadeEvent vtable, and stores one constructor argument at field 0x50. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959ee0` | `PlayerFadeEventDtor ?` | PlayerFadeEvent | Restores PlayerFadeEvent then PlayerEvent vtables and calls base event cleanup. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959f80` | `PlayerFadeEvent::ScalarDeletingDtor ?` | PlayerFadeEvent | Calls PlayerFadeEvent destructor and frees the allocation when deletion flag bit 0 is set. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959fa0` | `PlayerRemoveEventCtor ?` | PlayerRemoveEvent | Calls PlayerEvent base initializer and installs PlayerRemoveEvent vtable. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095a020` | `PlayerRemoveEventDtor ?` | PlayerRemoveEvent | Restores PlayerRemoveEvent then PlayerEvent vtables and calls base event cleanup. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095a080` | `PlayerRemoveEvent::ScalarDeletingDtor ?` | PlayerRemoveEvent | Calls the event destructor and frees the allocation when deletion flag bit 0 is set. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0096b320` | `PlayerRemoveEvent::ReleaseContextHandles ?` | PlayerRemoveEvent | Virtual slot 1 of PlayerRemoveEvent::vftable with a byte flag argument. It does not use its receiver: it obtains an object from 00ae2470 and, when nonnull, calls 007e7f00 and 00969910 on handles at object+0x6c8 and obje… | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00959f40` | `— ⚠reject` | PlayerFadeEvent | Obtains a global context; when present and deletion flag bit 0 is set, invokes its slot 9 with this+0x50, then calls 0076bbe0 using that value. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00ae2280` | `— ⚠reject` | PlayerHitpointEvent | Applies a hitpoint event to the current player/context. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

