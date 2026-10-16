# System: game.objectives

22 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009bd970` | `AddObjectiveRecord` |  | Adds a newly received objective and notifies the UI for the matching objective kind. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `00c43090` | `ClientObjective_ClearDisableById` |  | Body of the objective packet dispatcher 009be130 case 8 (community wire 45/8 ClientClear): disable one registered client objective by id. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c430e0` | `ClientObjective_FailById` |  | Body of the objective packet dispatcher 009be130 case 9 (community wire 45/9 ClientCompleteFailed): clear the completed mark of one client objective. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43140` | `ClientObjectiveList_OnObjectiveEvent` |  | The dispatch behind Lua Ui.OnObjectiveEvent: deliver a UI event name to every live, unfinished client objective (LookAt/UIEvent/FirstMovement/HousingFixtureMove). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45c30` | `ObjectiveCategory_GetPrefixKey` |  | Pick the localized 'primary / secondary / job / bonus / elite' prefix used when an objective row is labelled in the journal/table. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00cf35e0` | `ClientObjectiveLookAt::Ctor` | ClientObjectiveLookAt | Initializes a client objective that can complete when the camera aligns with a target or supplied world position. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf36a0` | `ClientObjectiveLookAt::CheckCompletion` | ClientObjectiveLookAt | Completes the client look-at objective after its distance and angle conditions pass. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3890` | `ClientObjectiveLookAt::Ctor` | ClientObjectiveLookAt | Initializes an actor-targeted look-at objective. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3920` | `ClientObjectiveUIEvent::Ctor` | ClientObjectiveUIEvent | Initializes an objective that listens for one named client UI event. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3a00` | `ClientObjectiveFirstMovement::Ctor` | ClientObjectiveFirstMovement | Initializes a first-movement objective with a selected input mode. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3a70` | `ClientObjectiveHousingFixtureMove::CheckFixtureAction` | ClientObjectiveHousingFixtureMove | Completes a fixture-move objective when its configured fixture/action pair matches the observed pair. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf3aa0` | `ClientObjectiveHousingFixtureMove::Ctor` | ClientObjectiveHousingFixtureMove | Initializes an objective for a configured housing-fixture action. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c43180` | `ClientObjectiveList_NotifySlot2` |  | Second objective-event fan-out used by the Lua-side binding 009aaf60. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c431c0` | `ClientObjectiveList_NotifySlot1` |  | Third objective-event fan-out used by the Lua-side binding 009aaf80. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009bd8f0` | `— ⚠reject` |  | Synchronizes encounter goal records into local objective state. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `00c43490` | `— ⚠rename` |  | Purge disabled objectives and poke the live ones at minigame start/load; the caller 009bf5e0 is the MiniGame:BeginLoad / HandlerMiniGameStart path. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c435c0` | `— ⚠reject` |  | Body of the objective dispatcher 009be130 case 6 when the wire carries a target GUID (community 45/6 ObjectiveLookAtPacket): create and register the by-actor look-at objective. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c436b0` | `— ⚠reject` |  | Body of the objective dispatcher 009be130 case 6 when the wire carries a target position instead of a guid: create and register the by-location look-at objective. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c437a0` | `— ⚠reject` |  | Body of the objective dispatcher 009be130 case 10 (community 45/10 UIEvent): register the objective that completes when the named Lua/UI event fires (for example onStoryBookClose). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43870` | `— ⚠reject` |  | Body of the objective dispatcher 009be130 case 11 (community 45/11 ObjectiveFirstMovementPacket): register the objective that completes on the player's first movement (keyboard or mouse). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c43940` | `— ⚠reject` |  | Body of the objective dispatcher 009be130 case 12 (community 45/12): register the objective that completes when the player moves the given housing fixture. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00cf3900` | `— ⚠reject` | ClientObjectiveUIEvent | Completes a UI-event objective when the received event name matches its stored event string. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

