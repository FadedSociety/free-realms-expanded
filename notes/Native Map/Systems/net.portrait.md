# System: net.portrait

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b4f7d0` | `AppendU16ThenU32` |  | Serializes a 16-bit then 32-bit field into the packet buffer used by 00b4fc30. | [[Native Map/Shards/S0345 00b4b3e0-00b50190|S0345]] |
| `00b4fc30` | `BuildAndSendTunneledRecord16_32` |  | Sends a tunneled client-to-world record carrying the 16-bit and 32-bit fields; the portrait upload path 00b4fd60 calls it when the engine flag at +0x280 is set. | [[Native Map/Shards/S0345 00b4b3e0-00b50190|S0345]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b4fc30` | `BuildAndSendTunneledRecord16_32` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the world server accepts a tunneled client-to-world record carrying a 16-bit and a 32-bit portrait field; depends on 00b4f7d0 (field layout) and on 00b4fd60 (the only caller). Not decoded from any capture. |
