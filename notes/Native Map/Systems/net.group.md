# System: net.group

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7e1c0` | `SendGroupKickPacketForMemberRow` |  | The Lua caller 00c0b020 supplies a group-member row for the client GroupPacketGroupKick path; this code does not show a server request for that row. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7e2e0` | `SendGroupKickPacketForGuid` |  | Builds a group-kick packet for an entity resolved from a guid pair; this domain label comes from its GroupPacketGroupKick constructor callee, and no exported caller is known. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00c24d20` | `SendGroupInvite` |  | Submits a group invitation request. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7e110` | `SendGroupLeavePacketIfConnected ? ⚠downgrade` |  | Builds the client's group-leave packet and sends it through the tunnel when the connection field is present; the server-side leave effect is not visible here. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00938650` | `— ⚠reject` |  | Sends the client group-map ping packet. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `00a7dd50` | `— ⚠reject` |  | Payload writer called by 00a7dfc0; it serializes two source subobjects and fields +0x130/+0x134, but their semantic roles and packet direction are not established here. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7dea0` | `— ⚠reject` |  | Wraps a caller-supplied serialized record in the tunneled-client gateway packet and sends it; the byte at wrapper+8 is a flag, not an opcode. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7dfc0` | `— ⚠reject` |  | Wraps the caller-supplied record payload in the tunneled-client gateway packet and sends it; the byte at wrapper+8 is a flag, not an opcode. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a7e1c0` | `SendGroupKickPacketForMemberRow` | send | PacketTunneledClientPacket |  | Hypothesis: the original server interprets this tunneled GroupPacketGroupKick (family 40/sub-opcode 6 from 008dd060) as a kick request. This depends on caller 00c0b020, constructor 008dd060 and sender 00a7dfc0; body semantics and server authorization rules are unknown. |
| `00a7e2e0` | `SendGroupKickPacketForGuid` | send | PacketTunneledClientPacket |  | Hypothesis: the original server interprets this tunneled GroupPacketGroupKick (family 40/sub-opcode 6 from 008dd060) as a kick request. This depends on constructor 008dd060 and sender 00a7dfc0; body semantics and server authorization rules are unknown. |
| `00a7e110` | `SendGroupLeavePacketIfConnected ? ⚠downgrade` | send | PacketTunneledClientPacket |  | Hypothesis: the original server interprets the tunneled inner packet as a group-leave request (constructor 008dcfe0 uses family 40/sub-opcode 3). This depends on that constructor and sender 00a7dea0; there are no captured family-40 bodies to establish server policy. |
| `00a7dea0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the server receives and decodes the tunneled payload supplied by the caller. Its body meaning depends on the caller's constructed inner record (for example, 00a7e110) and is not recovered by this generic sender. |
| `00a7dfc0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the server receives and decodes the tunneled payload supplied by the caller. The group/request semantics depend on the caller's constructed record and are not recovered by this generic sender. |
