# System: game.encounter

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a979a0` | `GroupEncounterRespondToInviteMatchmaking` |  | Accept or reject variant of the group encounter invitation answer that carries two ids and sends them in a matchmaking packet, with the same result lines as 00a97720. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b8ea0` | `CopyEncounterSettingsRecord` |  | Its sole caller, 008d2500, uses this copy for encounter-settings comparison; the record identity is inferred from that caller's role. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008c5b20` | `ResetRuntimeFields` |  | Resets per-session runtime fields of an object; its caller CancelPendingEncounter (00938fe0, accepted) suggests an encounter reset, which rests on that caller only. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `00a97720` | `GroupEncounterRespondToInvite ⚠downgrade` |  | Answers a group encounter invitation with accept or reject and prints the matching result line; refuses with the must-leave-minigame message while a minigame is active. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00c241f0` | `RequestPlayIntroEncounter` |  | Requests the introductory encounter. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a97720` | `GroupEncounterRespondToInvite ⚠downgrade` | send | BaseEncounterPacket |  |  |
