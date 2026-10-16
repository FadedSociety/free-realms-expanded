# System: net.packet.job_customization

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a4e90` | `JobCustomizationBasePacket::JobCustomizationBasePacket` | JobCustomizationBasePacket | Base constructor for family 155. Receive router 009471a0 constructs sub-opcode 1; outgoing finalize path 00a84fe0 constructs sub-opcode 2 through 00a842a0. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4f50` | `BeginJobCustimzationPacket::BeginJobCustimzationPacket` | BeginJobCustimzationPacket | Constructs opcode-155/sub-1 BeginJobCustimzationPacket in receive router 009471a0. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008b7fe0` | `JobCustomizationBasePacket::ScalarDeletingDtor` | JobCustomizationBasePacket | Releases a JobCustomizationBasePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b8040` | `BeginJobCustimzationPacket::ScalarDeletingDtor` | BeginJobCustimzationPacket | Releases a BeginJobCustimzationPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00a842a0` | `FinalizeJobCustomzationPacketCtor` | FinalizeJobCustomzationPacket | Builds the client's FinalizeJobCustomization request before its 18 equipment slots are copied in and it is sent (see 00a84fe0/00a84eb0). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a843f0` | `FinalizeJobCustomzationPacketScalarDeletingDtor` | FinalizeJobCustomzationPacket | Vtable slot 0 of the finalize-job-customization packet. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a4f00` | `JobCustomizationBasePacket::~JobCustomizationBasePacket` | JobCustomizationBasePacket | Base teardown step for opcode-155 job customization packets; class binding rests on the sibling family constructor. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4fb0` | `BeginJobCustimzationPacket::~BeginJobCustimzationPacket` | BeginJobCustimzationPacket | Teardown of the begin-customization packet; class binding comes from the router case. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00a84c00` | `PackJobCustomizationPayload` | FinalizeJobCustomzationPacket | Serializes the 18 equipment slot choices plus the job id that the client sends in its FinalizeJobCustomization request. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a84eb0` | `— ⚠reject` |  | Wire path of the job-customization finalize action. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84fe0` | `— ⚠reject` | JobCustomizationDataSource | Commits the player's current job-customization slot selection to the server. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a4e90` | `JobCustomizationBasePacket::JobCustomizationBasePacket` | send |  | 155 | Hypothesis: the peer sends family-155/sub-1 to the client and accepts the client sub-2 finalize request. Client paths 009471a0 and 00a84fe0 establish both directions; original customization rules are not recovered. |
| `008a4e90` | `JobCustomizationBasePacket::JobCustomizationBasePacket` | recv |  | 155 | Hypothesis: the peer sends family-155/sub-1 to the client and accepts the client sub-2 finalize request. Client paths 009471a0 and 00a84fe0 establish both directions; original customization rules are not recovered. |
| `008a4f50` | `BeginJobCustimzationPacket::BeginJobCustimzationPacket` | recv |  | 1 | Hypothesis: the peer sends family-155/sub-1 to the client path in 009471a0. This is a client dispatch finding, not evidence of original server internals. |
| `00a842a0` | `FinalizeJobCustomzationPacketCtor` | send |  | 2 | Hypothesis: the client finalized its job customization by sending JobCustomizationBasePacket (155) sub-opcode 2 with the chosen equipment slot item ids; the original server applied the new job and answered with updates. Depends on 00a84eb0 (the actual send path) and on the reply handling outside this shard. |
| `00a84c00` | `PackJobCustomizationPayload` | send | PacketTunneledClientPacket |  |  |
| `00a84eb0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server accepted JobCustomizationBasePacket (155) sub-opcode 2 as the payload of a class-5 tunneled packet, validated the 18 slot item ids against the player's inventory and applied the chosen job. Depends on 00a84fe0 (caller) and on the absent reply handler. |
| `00a84fe0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: same contract as 00a84eb0 - one class-5 tunneled packet carrying JobCustomizationBasePacket/2 with 19 dwords (job id plus 18 slot item ids). Depends on 00a84eb0 and on the job-customization reply path outside this shard. |
