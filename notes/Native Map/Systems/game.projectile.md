# System: game.projectile

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b83bb0` | `ProjectileManagerInterface::ScalarDeletingDtor` | ProjectileManagerInterface | Deleting destructor of ProjectileManagerInterface: slot [0] of ProjectileManagerInterface::vftable. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b83d20` | `ProjectileManager::Ctor` | ProjectileManager | Constructor of ProjectileManager. PlayClient::Initialize calls it. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84100` | `ProjectileManager::~ProjectileManager` | ProjectileManager | Destructor body of ProjectileManager. Its scalar deleting variant is 00b84590. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84590` | `ProjectileManager::ScalarDeletingDtor` | ProjectileManager | Deleting destructor of ProjectileManager: slot [0] of ProjectileManager::vftable. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84910` | `PlayerSpawnProjectileEvent::Ctor` | PlayerSpawnProjectileEvent | Constructor of PlayerSpawnProjectileEvent. ProjectileManager::Update (00b84e90) builds these spawn events. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84cc0` | `PlayerSpawnProjectileEvent::ScalarDeletingDtor` | PlayerSpawnProjectileEvent | Deleting destructor of PlayerSpawnProjectileEvent: slot [3] (+0x0c) of PlayerSpawnProjectileEvent::vftable (01823a90). | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b84050` | `ProjectileManager::ClearAll` | ProjectileManager | Destroys every projectile node in the manager. Called by BeginPlayerZoning and by the ProjectileManager destructor 00b84100. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |

## 🟡 speculative (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b84a20` | `PlayerSpawnProjectileEvent::ReleaseOwnerIfCheckZero ?` | PlayerSpawnProjectileEvent | Virtual slot [1] of the secondary PlayerSpawnProjectileEvent vtable (01823a54). Adjusts this by -0x50 into ESI, calls FUN_00ae2470 with ECX = that object, and when the result is 0 and the adjusted pointer is non-null ca… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84a50` | `PlayerSpawnProjectileEvent::ScalarDeleteThunkAdj50 ?` | PlayerSpawnProjectileEvent | Virtual slot [2] of the secondary PlayerSpawnProjectileEvent vtable (01823a54). Adjusts this by -0x50, and when the adjusted pointer is non-null tail-jumps through its vtable slot +0x0c with argument 1 (ret 4). | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84a70` | `PlayerSpawnProjectileEvent::DeleteSelfViaSlot3 ?` | PlayerSpawnProjectileEvent | Virtual slot [10] of the primary PlayerSpawnProjectileEvent vtable (01823a90). When this is non-null it tail-jumps through its own vtable slot +0x0c with argument 1, which is the scalar deleting destructor 00b84cc0. Oth… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84a90` | `PlayerSpawnProjectileEvent::ScalarDeletingDtorThunk ?` | PlayerSpawnProjectileEvent | Thunk, 8 bytes. SUB ECX, 0x50 then JMP 00b84cc0. It adjusts this from the secondary subobject to the primary and passes the scalar deleting argument through unchanged. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84aa0` | `PlayerSpawnProjectileEvent::ResolveAndReleaseHandle ?` | PlayerSpawnProjectileEvent | Virtual slot [1] of the primary PlayerSpawnProjectileEvent vtable (01823a90). Calls FUN_00ae2470 and, when it returns non-zero and flag bit 0 of the argument is set, resolves the handle at +0x130 through the owner's (+0… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84ce0` | `PlayerSpawnProjectileEvent::BuildRequestAndDispatch ?` | PlayerSpawnProjectileEvent | Virtual slot [3] of the secondary PlayerSpawnProjectileEvent vtable (01823a54). Calls FUN_00ae2470 and, when it returns non-zero, calls its vtable +0x24. It then copies a 16-dword template from DAT_01bc2660 into a local… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b84bf0` | `— ⚠reject` | PlayerSpawnProjectileEvent | Destructor body of PlayerSpawnProjectileEvent. Its scalar deleting variant is 00b84cc0. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b84e90` | `— ⚠reject` | ProjectileManager | Advance a projectile each frame: pick/average its target positions, apply the gravity-scaled acceleration, update its velocity fields and emit the per-tick update object built through the owner's vtable. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

