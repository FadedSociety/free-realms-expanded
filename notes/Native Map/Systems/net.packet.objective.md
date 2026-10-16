# System: net.packet.objective

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009afc10` | `ObjectiveUIEventPacket::Ctor` | ObjectiveUIEventPacket | Constructs packet storage; direction is not shown by constructor. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009b0ac0` | `ObjectiveUIEventPacket_ScalarDeletingDtor` | ObjectiveUIEventPacket | Scalar deleting destructor; body checked against slot zero. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009be130` | `DispatchObjectivePacket` |  | Routes received objective-family messages into client objective state, UI, movement, and housing-fixture handling. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `00cff7d0` | `ObjectiveClientCompletePacket::ObjectiveClientCompletePacket` | ObjectiveClientCompletePacket | Constructs the client objective-completion packet object. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cff890` | `ObjectiveClientCompletePacket::ScalarDeletingDtor` | ObjectiveClientCompletePacket | Implements scalar deletion for the objective completion packet class. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cffba0` | `SendObjectiveCompletion` |  | Reports completion of a registered client objective and marks the local objective completed. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009afc80` | `ObjectiveUIEventPacket::Dtor` | ObjectiveUIEventPacket | Packet-member cleanup body reached from ObjectiveUIEventPacket deleting destructor 009b0ac0; this body restores BasePacket only. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00cff7d0` | `ObjectiveClientCompletePacket::ObjectiveClientCompletePacket` | send | PacketTunneledClientPacket |  |  |
| `00cffba0` | `SendObjectiveCompletion` | send | PacketTunneledClientPacket |  | Hypothesis: a compatible original server unwrapped PacketTunneledClientPacket, dispatched inner client family/subopcode 45/7, and interpreted the included objective identifier as a completion report. This inference depends on constructor 00cff7d0, serializers 00cff950 and 00cff9f0, tunnel send 00cffa70, and the mapping in Research/Original Opening Microtutorial Recovery.md; no original server cod… |
