# System: audio.ambient

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f6c10` | `AmbientRandomEffect::DtorBody` | AmbientRandomEffect | Unwind-path cleanup of an AmbientRandomEffect object. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f6cb0` | `AmbientRandomEffect::ScalarDeletingDtor` | AmbientRandomEffect | Deletes an AmbientRandomEffect object (delete path, optionally freeing storage). | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f74a0` | `AmbientRandomEffect::CopyCtor` | AmbientRandomEffect | Copy-constructs an AmbientRandomEffect configuration record from another record. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f75e0` | `AmbientRandomEffect::ScalarDeletingDtor_Alt` | AmbientRandomEffect | Deletes an AmbientRandomEffect object (delete path, optionally freeing storage). | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f7640` | `AmbientRandomEffect::DefaultCtor` | AmbientRandomEffect | Default-constructs an AmbientRandomEffect object with its default field values. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f7c70` | `ListRemoveAmbientRandomEffect` |  | Removes an AmbientRandomEffect from its owner list and frees it through the list's free slot. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f8170` | `AmbientRandomEffectsRebuild` |  | Rebuilds the owner's AmbientRandomEffect list from a source collection and refreshes the keyed entries. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f8370` | `Owner3Lists_DtorBody` |  | Teardown of the composite owner of background-track, room-type and ambient-effect lists. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f6c20` | `AmbientRandomEffect::ReadFloat50 ?` | AmbientRandomEffect | Virtual slot [1] of AmbientRandomEffect::vftable: returns the float at this+0x50 on the x87 stack (fld dword [ecx+0x50]; ret, 4 bytes). | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f6c90` | `AmbientRandomEffect::RaiseFloat50 ? ⚠rename` | AmbientRandomEffect | Virtual slot [2] of AmbientRandomEffect::vftable, thiscall with one float argument (ret 4): when the argument is greater than the float constant at DAT_01762a30 (comiss against it), it stores the argument at this+0x50; … | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f6f80` | `AmbientRandomEffect::InitVtableOnly ?` | AmbientRandomEffect | Installs AmbientRandomEffect::vftable at the object's leading word; constructor versus destructor role is unresolved. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |

