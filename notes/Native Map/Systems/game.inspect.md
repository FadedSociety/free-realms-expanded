# System: game.inspect

14 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad7430` | `InspectProxy::OnInspectSelect` | InspectProxy | Dispatches an action on the inspect target after successful lookup. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad7460` | `InspectProxy::UpdateProfile` | InspectProxy | Updates inspect proxy profile presentation and associated tracking entries. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad7660` | `InspectProxy::ClearProfileEntries` | InspectProxy | Releases inspect proxy profile entries. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad7c10` | `InspectProxy::OnTargetChanged` | InspectProxy | Maintains inspect proxy target references when target object changes. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad7fc0` | `InspectProxy::Dtor → InspectProxy::~InspectProxy` | InspectProxy | Cleans InspectProxy state; also reached from EH cleanup. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad8390` | `InspectProxy::ScalarDeletingDtor` | InspectProxy | Scalar deleting destructor for InspectProxy. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad8880` | `InspectProxy::InspectProxy ⚠downgrade` | InspectProxy | Builds the client-side inspect-target proxy that renders another player's model with its customization and attachment components; the BaseClient.InspectTarget. string is the label prefix it seeds through FUN_00bad8f0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## 🟡 speculative (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad66f0` | `InspectProxy::ScalarDeletingDtor ? ⚠rename` | InspectProxy | Scalar deleting destructor wrapper; exact owning class is not independently anchored in this function. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad6730` | `InspectProxy::MarkInspectReady ? ⚠rename` | InspectProxy | Marks an inspect-related object flag; field meaning inferred from class vtable context. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad73d0` | `InspectProxy::OnInspectEvent ? ⚠rename` | InspectProxy | Forwards an inspect proxy event to its target handler. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad73f0` | `InspectProxy::OnInspectReady ? ⚠rename` | InspectProxy | Performs a follow-up action after the target handler accepts the stored id. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad76d0` | `InspectProxy::ClearTarget ?` | InspectProxy | Releases inspect target state. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad7f40` | `ReadCharacterStats ?` |  | Called by InspectProxy construction to deserialize CharacterStat records. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad83b0` | `— ⚠reject` |  | Deserialization helper called by InspectProxy construction; item catalog records are updated from the stream. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

