# System: ui.dialog

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f5ae0` | `ShowDialogFromPacket` |  | Presents a parsed dialogue payload (text, title, up to five choices) on the HandlerDialog UI object and shows it. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |
| `009f6890` | `EndDialogAndHide` |  | Closes the active HandlerDialog, conditionally tunnels an EndDialog command, and invokes the selected manager state callback. | [[Native Map/Shards/S0283 009f39b0-009f8370|S0283]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009f6890` | `EndDialogAndHide` | send | PacketTunneledClientPacket |  | Hypothesis: when the manager connection is available, the peer receives a tunneled CommandPacketEndDialog. This depends on constructor 009f5760 (sub-opcode 4) and the outbound tunnel path through 009f6630; no original server-side close behavior is recovered. |
