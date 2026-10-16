# System: engine.player_event

10 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5510` | `PlayerEvent::~PlayerEvent` | PlayerEvent | Destructor of PlayerEvent; the in-body vtable restore names the class and the member teardown call runs on this. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a5570` | `PlayerEvent::ScalarDeletingDestructor` | PlayerEvent | Compiler-generated scalar deleting destructor for PlayerEvent; the /GS frame-cookie expression is bookkeeping, not a teardown argument. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `0093f1c0` | `PlayerEventHandler::HandleEvent` | PlayerEventHandler | Routes player event masks through target and player-event handling; exact event names remain unknown. | [[Native Map/Shards/S0249 0093daf0-00942c60|S0249]] |
| `0093f630` | `ActorEventHandler::HandleEvent` | ActorEventHandler | Routes actor events and updates tooltip state; the event-mask meanings remain unknown. | [[Native Map/Shards/S0249 0093daf0-00942c60|S0249]] |
| `00aa1e40` | `EncounterCompleteEvent::EncounterCompleteEvent` | EncounterCompleteEvent | Game event object raised when an encounter completes; the dispatch of its payload happens through the overridden slot 1 (00aa1f90) and the member layout matches the addresses that override passes. | [[Native Map/Shards/S0313 00a9d3c0-00aa24e0|S0313]] |
| `00aa1f10` | `EncounterCompleteEvent::~EncounterCompleteEvent` | EncounterCompleteEvent | Teardown of the encounter-complete event; reached only from its scalar deleting destructor 00aa1fd0. | [[Native Map/Shards/S0313 00a9d3c0-00aa24e0|S0313]] |
| `00aa1f90` | `EncounterCompleteEvent::Apply` | EncounterCompleteEvent | The override where the encounter-complete event hands its payload to the client (FUN_009bf4d0 writes them into an object reached through the singleton); the base hook it replaces is the shared stub 007db980 that returns… | [[Native Map/Shards/S0313 00a9d3c0-00aa24e0|S0313]] |
| `00aa1fd0` | `EncounterCompleteEvent::ScalarDeletingDtor` | EncounterCompleteEvent | Vtable slot 3 of EncounterCompleteEvent (the slot GameEvent uses for its destructor). | [[Native Map/Shards/S0313 00a9d3c0-00aa24e0|S0313]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bf6840` | `PlayerRequestHeadShotEvent::DtorBody` | PlayerRequestHeadShotEvent | Destructor body of PlayerRequestHeadShotEvent (vtable-store-first order, the destructor shape); the base body is 007e7980. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bf68c0` | `— ⚠reject` | PlayerRequestHeadShotEvent | PlayerRequestHeadShotEvent::vftable[1]. When stack arg2 bit 0 is set: reads this+0x48 and +0x4c, calls 00769710 then 00984960 on them; if the result is nonzero and its byte at +0x145 lacks mask 0x40, calls 00769710 agai… | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |

