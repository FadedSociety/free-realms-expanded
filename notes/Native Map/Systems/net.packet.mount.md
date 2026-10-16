# System: net.packet.mount

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b3730` | `PacketMountRequest::PacketMountRequest` | PacketMountRequest | Constructor of the client's mount request packet (mount family 168 = MountBasePacket, sub-opcode 1), built by the single caller 00934970; the vault's mounts note records the same sub-opcode constructor pattern for the f… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `00bff390` | `PacketMountUpgradeRequest::Ctor` | PacketMountUpgradeRequest | Initializes a mount upgrade request packet object. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bff4a0` | `PacketMountUpgradeRequest::ScalarDeletingDtor` | PacketMountUpgradeRequest | Implements deleting cleanup for the mount upgrade packet object. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008b3730` | `PacketMountRequest::PacketMountRequest` | send | MountBasePacket | 1 | Hypothesis: sub-opcode 1 of mount opcode 168 is the client's mount request - the catalogue ships subs 2, 3 (PacketDismountRequest), 4, 5, 6 and 8 but not 1, and the vault's mounts note records enum entry 1 = PacketMountRequest. The sole caller 00934970 is a send path (it logs the request, builds the packet and calls the gateway helper while the client connection is up), so the original server had… |
| `00bff390` | `PacketMountUpgradeRequest::Ctor` | send | MountBasePacket |  | Hypothesis: opcode 168/sub-opcode 14 denotes a mount upgrade request; this is a client packet constructor only and gives no evidence of original server handling. |
