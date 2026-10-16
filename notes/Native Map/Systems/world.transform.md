# System: world.transform

13 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b2fa0` | `SoundListenerEntity::UpdateTransformsFromSource` | SoundListenerEntity | Slot 7 of 018014e4 SoundListenerEntity::vftable is this address, so this is the listener entity's transform refresh: it takes the source's offset/orientation blocks, stores its own copies and lets 007bd8f0/007bd930 upda… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b31c0` | `GetPairFromSlot1870` |  | Accessor for a two-dword value (a position or key pair) held on the actor object at +0x1870 with a default stored in the globals at 01be4850; callers are the screenshot manager update and several 00b5/00bd/00cc world ro… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b31e0` | `DiffersFromDefaultPair` |  | Predicate that tests whether an actor's cached pair differs from the default globals used by the pair accessor 008b31c0; the single caller 00973200 uses it while deciding whether to refresh the actor's transform. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b32e0` | `GetPairFromSlot18b0` |  | Second two-dword accessor on the same actor object as 008b31c0 but from the +0x18b0 slot; the single caller 00bf9430 is unlabelled, so which value the pair holds is not established. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008ba7a0` | `OffsetResolvedVectorAndApply` |  | The adjusted vector is handed to the transform-like helper FUN_008a7910; the object and coordinate semantics are not identified here. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00961b70` | `GetSeatTransform` |  | The confirmed UpdateSeatAttachment caller supplies the seat-related context; the exact caller-side attachment policy is not repeated here. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00961c40` | `GetSeatTransformSecondary` |  | Called by UpdateSeatAttachment for a second seat transform; the exact attachment policy belongs to that caller. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00962200` | `SetModelFloatWhenChanged` |  | Sets a related model value only after a comparison detects a change. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007889a0` | `ModelEntity::CopyMatrix16AndNotify ?` | Deep::ModelEntity | Copies 16 dwords (64 bytes) from the argument into this+0x160. If this+0xc is non-null, calls its vtable slot +0x18 with the argument. If this+4 is non-null, calls 007351c0. Returns with ret 4. Virtual slot [7] of Deep:… | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `007889e0` | `ModelEntity::CallThenNotifyMatrix ?` | Deep::ModelEntity | Shared implementation of Actor::vftable[11] and Deep::ModelEntity::vftable[11]; named for the base class Deep::ModelEntity. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `00788a30` | `Actor::CopyMatrix16AndNotify ?` | Actor | Same body as 007889a0: copies 16 dwords from the argument into this+0x160, calls the vtable slot +0x18 of the object at this+0xc with the argument when non-null, and calls 007351c0 when this+4 is non-null. Virtual slot … | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009606e0` | `— ⚠reject` |  | Copies a transform-like record into object state and optionally applies a related model update; callers include mount and housing state paths. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963040` | `— ⚠reject` |  | Applies the related model heading when the stored flag changes. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |

