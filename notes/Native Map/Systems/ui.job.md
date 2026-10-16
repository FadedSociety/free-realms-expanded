# System: ui.job

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ff660` | `BaseClient::UpdateJobSkillProgress` | BaseClient | Job-skill UI bridge: forwards one skill-progress entry of a job (id param_2, element index param_3) to the Lua handler HandlerJob:UpdateSkillProgress. Same vtable slot 38 is shared by PlayClient and BaseClient; named fo… | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00900370` | `PushSetJobInfo` |  | Pushes job summary data to the Lua job browser handler through ':SetJobInfo'; called by PlayClient::HandleJobBrowser (caller label 0090c500). | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00900a20` | `PushJobSkillPoints` |  | Sends two job skill-point values to the Lua job browser handler through ':SetJobSkillPoints'; called from BaseClient::HandleClientUpdate (caller 009392c0). | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `0090c500` | `PlayClient::HandleJobBrowser` | PlayClient | Updates the job browser UI with available job profiles and selected job state. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `00a78860` | `JobTooltip_BuildRows` |  | Fills the JobTooltip row table for a job: level, progress and stars available and earned. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00ad4b80` | `ShowJobHandler` |  | Shared card manager method that opens JobHandler. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00c0a760` | `RegisterJobScreenModule` |  | Registers the client 'JobScreen' Lua module (PopulateJobList, SelectJob, PopulateJobInfo, PurchaseAbility, GetJobCount). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0a630` | `Job_SelectJob` |  | Handler registered as 'SelectJob' in the JobScreen module (0x0182a7f0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0a680` | `Job_PopulateJobInfo` |  | Handler registered as 'PopulateJobInfo' in the JobScreen module (0x0182a7f0): fills the job information panel. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0a6c0` | `Job_PurchaseAbility ? ⚠downgrade` |  | Handler registered as 'PurchaseAbility' in the JobScreen module (0x0182a7f0): spends job points on an ability. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ff900` | `— ⚠reject` | BaseClient | Job browser presentation: pushes one job's header and its ability/trait entries to Lua HandlerJobBrowser (':SetJob', ':SetJobAbility', ':SetJobTrait', ':PresentJob'). Vtable slot 39 is shared with PlayClient; named for … | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |

