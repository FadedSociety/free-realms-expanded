# System: net.packet.inspect

12 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a3b00` | `BaseInspectPacket::BaseInspectPacket` | BaseInspectPacket | Shared inspect-family header constructor used by client start/stop requests and the receive-side WorldInspectReplyPacket. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008a3bd0` | `StopInspectPacket::StopInspectPacket` | StopInspectPacket | Builds the stop-inspect request the client sends when the player closes an inspect window. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008b7790` | `WorldInspectReplyPacket::WorldInspectReplyPacket` | WorldInspectReplyPacket | Initializes the world-inspect reply packet object. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b77f0` | `WorldInspectReplyPacket::ScalarDeletingDtor` | WorldInspectReplyPacket | Releases a WorldInspectReplyPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008d1860` | `StartInspectPacket::StartInspectPacket` | StartInspectPacket | Constructs the StartInspectPacket object while processing inbound family 149/subtype 1 traffic: caller 009471a0 is the packet router and reaches this constructor in its opcode-0x95 case. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `009347d0` | `StopInspect` |  | Stops the active inspect interaction and releases its local state. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `00c1abf0` | `CommandPacketDirectInspect::ScalarDeletingDtor` | CommandPacketDirectInspect | Performs scalar deleting destruction for the direct-inspect packet. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a3b70` | `BaseInspectPacket::~BaseInspectPacket` | BaseInspectPacket | Destroys inspect packet objects after the request was sent or the reply applied. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008a3c30` | `StopInspectPacket::~StopInspectPacket` | StopInspectPacket | Destroys the stop-inspect request object on cleanup/exception paths. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008a3c80` | `WorldInspectReplyPacket::~WorldInspectReplyPacket` | WorldInspectReplyPacket | Destroys the world-inspect reply object (the inspected entity's data) after the UI consumed it. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008d18f0` | `StartInspectPacket::~StartInspectPacket` | StartInspectPacket | Non-deleting destructor of the object built by the start-inspect constructor 008d1860 (same one-string layout at +0x18); its vtable slot-0 scalar deleting destructor 008dd230 calls this body first, binding the class. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c1ab80` | `— ⚠reject` | CommandPacketDirectInspect | Builds a direct-inspect command packet. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a3b00` | `BaseInspectPacket::BaseInspectPacket` | send | BaseInspectPacket | 149 | Hypothesis: the original server accepted inspect start/stop requests and sent inspection replies on family 149. This depends on stop caller 009347d0 and the client reply decoder 009471a0/008b7790. |
| `008a3b00` | `BaseInspectPacket::BaseInspectPacket` | recv | BaseInspectPacket | 149 | Hypothesis: the original server accepted inspect start/stop requests and sent inspection replies on family 149. This depends on stop caller 009347d0 and the client reply decoder 009471a0/008b7790. |
| `008a3bd0` | `StopInspectPacket::StopInspectPacket` | send | StopInspectPacket |  | Hypothesis: the original server treated 149/2 as the end of the inspect session and stopped streaming profile updates for the target. Depends on catalogue row 149/2 and caller 009347d0. |
| `008d1860` | `StartInspectPacket::StartInspectPacket` | recv | StartInspectPacket |  | Hypothesis: the original server received 149/1 (start-inspect) and answered with the inspected entity's profile data on the same family, mirroring the stop-inspect request; the client-side constructor and the catalogue row are proven, the request's usage is inferred from the class name and caller. Depends on the 009471a0 router and the S0219 inspect-family constructor 008a3b00. |
| `009347d0` | `StopInspect` | send | StopInspectPacket |  |  |
| `00c1ab80` | `— ⚠reject` | send |  | 45 | Hypothesis: server receives a direct-inspect command with two payload words; inferred from this constructor only, not original server logic. |
