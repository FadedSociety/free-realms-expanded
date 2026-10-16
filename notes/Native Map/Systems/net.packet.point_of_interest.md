# System: net.packet.point_of_interest

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b97a20` | `PacketPointOfInterestDefinitionRequest::PacketPointOfInterestDefinitionRequest` | PacketPointOfInterestDefinitionRequest | Constructs the client request that asks the server for the point-of-interest (warpstone/atlas) destination list; sent by 00b99150. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97ad0` | `PacketPointOfInterestDefinitionReply::PacketPointOfInterestDefinitionReply` | PacketPointOfInterestDefinitionReply | Constructs the server definition reply that carries the point-of-interest rows; built by the handler 00b98b70. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97b80` | `PacketZoneTeleportRequest::PacketZoneTeleportRequest` | PacketZoneTeleportRequest | Constructs the warpstone teleport request with the destination point-of-interest id; sent by 00b991d0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97ef0` | `PacketPointOfInterestDefinitionRequest::ScalarDeletingDestructor` | PacketPointOfInterestDefinitionRequest | vtable slot 0 of PacketPointOfInterestDefinitionRequest. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97f50` | `PacketPointOfInterestDefinitionReply::ScalarDeletingDestructor` | PacketPointOfInterestDefinitionReply | vtable slot 0 of PacketPointOfInterestDefinitionReply. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97fb0` | `PacketZoneTeleportRequest::ScalarDeletingDestructor` | PacketZoneTeleportRequest | vtable slot 0 of PacketZoneTeleportRequest. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b98ad0` | `PacketZoneTeleportRequest_Serialize` | PacketZoneTeleportRequest | Body writer used by the send helper 00b99000 for the warpstone teleport request. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b97a20` | `PacketPointOfInterestDefinitionRequest::PacketPointOfInterestDefinitionRequest` | send |  | 56 | Hypothesis: the original server answered the op-56 request with the op-57 definition reply parsed by 00b98b70; the two are a matched pair in the warpstone flow. |
| `00b97ad0` | `PacketPointOfInterestDefinitionReply::PacketPointOfInterestDefinitionReply` | recv | PacketPointOfInterestDefinitionReply | 57 | Hypothesis: the original server sent op-57 with the unframed or length-prefixed record stream parsed by 00b98990; the preserved captures contain no 57 frame, so the framing is community reconstruction. |
| `00b97b80` | `PacketZoneTeleportRequest::PacketZoneTeleportRequest` | send | PacketZoneTeleportRequest | 90 | Hypothesis: the original server answered op-90 with the client update packet 38/12 carrying the destination position and rotation, as recorded in the preserved captures (Research/Pieces/077). |
| `00b98ad0` | `PacketZoneTeleportRequest_Serialize` | send | PacketZoneTeleportRequest |  | Hypothesis: the original server read exactly four payload bytes (the destination id) as the captures show, so this serializer is the writer of the captured op-90 request body. |
