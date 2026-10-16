# System: ui.guild

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1cb70` | `HandleGuildDeleteNotification` |  | Client-side guild deletion notification handler. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1cd30` | `HandleGuildErrorPacket` |  | Client-side guild error packet handling; the specific server error semantics are not recovered. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d5b0` | `HandleGuildInviteAcceptedNotification` |  | Client-side handler for a successful guild-invite notification. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1b6f0` | `EmitPlayerGuildMessage` |  | Emits one guild notification whose text is selected by the message id from the text table; the parameter names player/guild plus the call site in the dispatcher 00b1ee10 (case 0x13, GuildNameAcceptedPacket) place it in … | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1b930` | `— ⚠reject` |  | Guild membership notification emitter: the parameter names player/member/guild match the '%s has joined the guild' family, and its only caller is the member-status handler 00b1d7d0. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b1cb70` | `HandleGuildDeleteNotification` | recv | BaseGuildPacket |  |  |
| `00b1cd30` | `HandleGuildErrorPacket` | recv | GuildErrorPacket |  |  |
| `00b1d5b0` | `HandleGuildInviteAcceptedNotification` | recv | GuildInviteAcceptNotificationPacket |  |  |
