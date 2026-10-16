# System: minigame.battlemages

21 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd4d50` | `BattleMagesProxiedPlayer::~BattleMagesProxiedPlayer` | BattleMagesProxiedPlayer | Destructor body of BattleMagesProxiedPlayer; resets the vtable only. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4e70` | `BattleMagesProxiedPlayer::BattleMagesProxiedPlayer` | BattleMagesProxiedPlayer | Constructor of the proxied battle-mages player record used by its list node. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4eb0` | `BattleMagesProxiedPlayer::SetVtableThenFreeIfFlag` | BattleMagesProxiedPlayer | Scalar-deleting-style entry of the proxied player with the vtable store inlined. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5280` | `BattleMagesProxiedProjectile::~BattleMagesProxiedProjectile` | BattleMagesProxiedProjectile | Destructor body of BattleMagesProxiedProjectile. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5330` | `BattleMagesProxiedProjectile::ScalarDeletingDtor` | BattleMagesProxiedProjectile | Scalar deleting destructor of BattleMagesProxiedProjectile (vtable slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5b90` | `BattleMagesCommandProcessor::BattleMagesCommandProcessor` | BattleMagesCommandProcessor | Constructor of the battle-mages command processor, which registers two named commands. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5c20` | `BattleMagesCommandProcessor::~BattleMagesCommandProcessor` | BattleMagesCommandProcessor | Destructor body of BattleMagesCommandProcessor. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd69e0` | `BattleMagesCommandProcessor::ProcessCommand` | BattleMagesCommandProcessor | Processes supported Battle Mages command kinds. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd5ca0` | `BattleMagesCommandProcessor::CallDtorBodyThenFreeIfFlag` | BattleMagesCommandProcessor | Release path through vtable slot 1 of BattleMagesCommandProcessor (destructor-shaped, not slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd6110` | `BuildBattleMagesCommand` |  | Builds one Battle Mages command payload for transmission. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd6190` | `SetBattleMagesCommandFields` |  | Populates a Battle Mages command object used by the send path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd6230` | `BuildBattleMagesCommandFromInput → BuildBattleMagesCommandWithName` |  | Converts local input into a Battle Mages command. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd5250` | `Thunk_Member14_00fe2850 ?` |  | Jump thunk: loads ECX from [ECX+0x14] (mov ecx,[ecx+0x14]) and jumps to 00fe2850 (jmp, not call). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5260` | `Thunk_Member14_00fe08a0 ?` |  | Jump thunk: loads ECX from [ECX+0x14] (mov ecx,[ecx+0x14]) and jumps to 00fe08a0 (jmp, not call). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5270` | `Thunk_Member14_00fe0990 ?` |  | Jump thunk: loads ECX from [ECX+0x14] (mov ecx,[ecx+0x14]) and jumps to 00fe0990 (jmp, not call). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5b30` | `RegisterReloadAndCameraCommands ?` |  | Fastcall (ECX = this). Calls 00987000 with the name bm_reloaddata and id 1, then with bm_camera and id 2, passing this as the second argument. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd62c0` | `CanIssueBattleMagesCommand ? ⚠rename` |  | Guards a Battle Mages command on controller state. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b72f60` | `— ⚠reject` |  | Activity-state switch for the battle-mages minigame: entering/leaving the activity starts or stops the camera block and the proxied-player registration; the vault note 'Research/Kart Entry Register Prerequisites.md' nam… | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00cd52d0` | `— ⚠reject` | BattleMagesProxiedProjectile | Constructor of the BattleMagesProxiedProjectile proxy record. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5710` | `— ⚠reject` |  | Handles a create-proxied-projectile request for the projectile at this, creating its physics object and refreshing its transform. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5b70` | `— ⚠reject` |  | Pushes its incoming ECX as a stack argument, loads ECX from dword 0x01be1090 plus 1, calls 00986c10 and returns (bytes 51 8b 0d .. 83 c1 01 e8 .. c3). 00986c10 reads one stack argument ([esp+0x14] after its four pushes). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00cd6110` | `BuildBattleMagesCommand` | send | PacketTunneledClientPacket |  | Hypothesis: the original service received this client-originated Battle Mages payload through the tunneled envelope. This depends on the command constructor, serializer 00cd5e90, sender 00cd5fe0, and gateway helper 00925640; server semantics are unknown. |
| `00cd6190` | `SetBattleMagesCommandFields` | send | PacketTunneledClientPacket |  | Hypothesis: the original service received this client-originated Battle Mages payload through the tunneled envelope. This depends on the command constructor, serializer 00cd5e90, sender 00cd5fe0, and gateway helper 00925640; server semantics are unknown. |
| `00cd6230` | `BuildBattleMagesCommandFromInput → BuildBattleMagesCommandWithName` | send | PacketTunneledClientPacket |  | Hypothesis: this input path can cause the original service to receive a client-originated Battle Mages payload. This depends on 00cd5e90, 00cd5fe0, and 00925640; the command and server effect remain unknown. |
| `00b72f60` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the server treats the RegisterPlayer message as join/leave of the battle-mages activity keyed to the authenticated session; it must not grant gameplay authority from this alone. |
