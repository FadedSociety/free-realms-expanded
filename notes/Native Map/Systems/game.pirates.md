# System: game.pirates

10 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a644e0` | `ShipCombat_RefreshShipEffectObjects` |  | Maintains effect objects associated with entity records; the caller 00a64960 is stage setup, while the per-entity resource fields and attachment chain are inferred from this function's offsets and helpers. The exact eff… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a64960` | `Pirates_SetupShipCombatStage` |  | Idempotent entry step of the Pirates minigame stage, reached from the minigame state machine 00a6bff0; the flag +0x3ac makes a repeated call a no-op. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a64ef0` | `ShipCombatZonePickupLocationConfig::PickupInstance_CopyCtor` | ShipCombatZonePickupLocationConfig::Pic… | Element copy constructor used by the pickup-instance array resize path 00a65050; the array vtable identifies the element name, but does not independently establish a separate enclosing class relationship. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a65030` | `ShipCombatCannonConfig_ScalarDtor` |  | Deletes a ShipCombatCannonConfig element; 00a62030 is that element's destructor body (it stores SoeUtil::List<ShipCombatCannonConfig,-1> and IString member vtables), so the element type is inherited from that callee. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a656b0` | `ShipCombatCannonConfig_CopyCtor` |  | Element copy constructor used when the cannon-config array is reallocated (00a66ad0) and when packets carry cannon configs (00a67e30, 00a68950); the element type is inferred from those neighbours and from the member lis… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a668f0` | `ProxiedShip_CtorFromSpawnData` |  | Builds the proxied-ship object from spawn data on the CreateProxiedShip path; the only caller 00a673f0 stores the spawn record into it, so the object identity is inherited from that caller rather than proven by RTTI her… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a673f0` | `FactoryResultRegister` |  | Factory callback used by hash-map insertion 00a67b40. That map helper is called by inbound CreateProxiedShip handler 00a6c3b0, so the proxied-ship role is supported by the caller chain rather than a local string or RTTI… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a67470` | `Wrapper_00a67470` |  | Mechanical forwarding wrapper for destructor body 00a66780; no local class name or vtable slot is established. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a66780` | `— ⚠reject` |  | Destructor of the object that owns the HashListMap<int,ShipCombatShipItem> and HashListMap<int,ShipCombatFlagItem> members; called from the wrappers 00a66a80/00a67470 and from the delete paths 00a67750/00a67c40/00a68330… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a66a80` | `— ⚠reject` |  | One of the two destructor entry points of the ship-combat processor object (the other is 00a67470); both are three-instruction forwarders, so the name stays mechanical. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

