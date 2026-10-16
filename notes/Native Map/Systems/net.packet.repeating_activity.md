# System: net.packet.repeating_activity

44 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad9620` | `BaseRepeatingActivityPacket::BaseRepeatingActivityPacket` | BaseRepeatingActivityPacket | Initializes the shared opcode-143 base fields used by repeating-activity packet objects; callers include outbound request constructors and the receive dispatcher. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad98d0` | `RepeatingActivityConsecutiveMsgPacket::RepeatingActivityConsecutiveMsgPacket` | RepeatingActivityConsecutiveMsgPacket | Constructs the type-3 consecutive-message object consumed by 00adbf00 and the wheel notification path 00adba80. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad99e0` | `RepeatingActivitySetConsecutivePacket::RepeatingActivitySetConsecutivePacket` | RepeatingActivitySetConsecutivePacket | Builds the client request that sets an activity's consecutive-day value; the only caller (00adcb30) sends it to the server when connected. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9eb0` | `BaseRepeatingActivityPacket::ScalarDeletingDtor` | BaseRepeatingActivityPacket | Heap-delete path used when a base repeating-activity packet is destroyed through its vtable. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9f10` | `RepeatingActivityStatePacket::ScalarDeletingDtor` | RepeatingActivityStatePacket | Heap-delete path for state messages received by the client. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9f70` | `RepeatingActivityConsecutiveMsgPacket::ScalarDeletingDtor` | RepeatingActivityConsecutiveMsgPacket | Heap-delete path for consecutive-day messages received by the client. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9fd0` | `RepeatingActivityClearRequest::RepeatingActivityClearRequest` | RepeatingActivityClearRequest | Builds the type-4 request that asks the server to clear a repeating activity; callers 00adc9b0 and 00adca40 send it when the client is connected. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada040` | `RepeatingActivityClearRequest::ScalarDeletingDtor` | RepeatingActivityClearRequest | Heap-delete path for clear requests. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada0a0` | `RepeatingActivitySetConsecutivePacket::ScalarDeletingDtor` | RepeatingActivitySetConsecutivePacket | Heap-delete path for set-consecutive requests. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada1a0` | `RepeatingActivityBaseAdminPacket::ScalarDeletingDtor` | RepeatingActivityBaseAdminPacket | Heap-delete path for the admin message base. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada280` | `RepeatingActivityAddCountPacket::ScalarDeletingDtor` | RepeatingActivityAddCountPacket | Heap-delete path for addcount requests. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada2e0` | `RepeatingActivityAddCountResultPacket::RepeatingActivityAddCountResultPacket` | RepeatingActivityAddCountResultPacket | Constructs the type-7 result object that the receive dispatcher parses before 00adb250 logs its fields. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada370` | `RepeatingActivityAddCountResultPacket::ScalarDeletingDtor` | RepeatingActivityAddCountResultPacket | Heap-delete path for addcount result messages. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada450` | `RepeatingActivitySetCountPacket::ScalarDeletingDtor` | RepeatingActivitySetCountPacket | Heap-delete path for setcount requests. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada530` | `RepeatingActivityGetCountRemainingPacket::ScalarDeletingDtor` | RepeatingActivityGetCountRemainingPacket | Heap-delete path for count-remaining queries. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada610` | `RepeatingActivityCountRemainingInfoPacket::ScalarDeletingDtor` | RepeatingActivityCountRemainingInfoPack… | Heap-delete path for count-remaining info messages. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada9b0` | `RepeatingActivityAddPacket::RepeatingActivityAddPacket` | RepeatingActivityAddPacket | Constructs the type-1 add/update object parsed by 00adbf00 and applied to the activity list by 00adb990. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adaa50` | `RepeatingActivityAddPacket::~RepeatingActivityAddPacket` | RepeatingActivityAddPacket | Non-deleting destructor of the add message; the scalar deleting dtor 00adad40 calls it before freeing. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adad40` | `RepeatingActivityAddPacket::ScalarDeletingDtor` | RepeatingActivityAddPacket | Heap-delete path for type-1 add messages. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adbf00` | `RepeatingActivity_DispatchMessage` |  | Receive-side dispatcher registered from BaseClient switch 009471a0. It switches on the byte at +8, not the preceding 16-bit field. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc3c0` | `RepeatingActivity_SendClearRequest` |  | Frames and sends the type-4 clear request; called by 00adc9b0 (clear-all) and 00adca40 (clear-by-name) when the client is connected. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc4d0` | `RepeatingActivity_SendSetConsecutive` |  | Frames and sends the type-5 set-consecutive request (caller 00adcb30). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc5e0` | `RepeatingActivity_SendAddCount` |  | Frames and sends the type-6 addcount admin request (caller 00adcbb0). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc6f0` | `RepeatingActivity_SendSetCount` |  | Frames and sends the type-8 setcount admin request (caller 00adcc40). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc800` | `RepeatingActivity_SendGetCountRemaining` |  | Frames and sends the type-9 count-remaining admin query (caller 00adccd0). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad97b0` | `RepeatingActivityStatePacket::~RepeatingActivityStatePacket` | RepeatingActivityStatePacket | Non-deleting destructor of the state message object; identical bytes to every other no-member packet destructor, class bound by the dispatcher pairing with constructor 00ad9740. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9940` | `RepeatingActivityConsecutiveMsgPacket::~RepeatingActivityConsecutiveMsgPacket` | RepeatingActivityConsecutiveMsgPacket | Non-deleting destructor of the consecutive-message packet; class bound by the dispatcher pairing with constructor 00ad98d0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9b70` | `RepeatingActivityAddCountResultPacket::~RepeatingActivityAddCountResultPacket` | RepeatingActivityAddCountResultPacket | Non-deleting destructor of the addcount-result packet; class bound by the dispatcher pairing with constructor 00ada2e0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9c60` | `RepeatingActivityCountRemainingInfoPacket::~RepeatingActivityCountRemainingInfo…` | RepeatingActivityCountRemainingInfoPack… | Non-deleting destructor of the count-remaining info packet; class bound by the dispatcher pairing with constructor 00ada590. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb6d0` | `RepeatingActivityStatePacket_Unserialize` | RepeatingActivityStatePacket | Validates and decodes a received type-2 state message into the packet object the dispatcher then hands to 00ada800. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb740` | `RepeatingActivityAddPacket_Unserialize` | RepeatingActivityAddPacket | Decodes a received type-1 add message before 00adb990 applies it to the client's activity list. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb7b0` | `RepeatingActivityConsecutiveMsgPacket_Unserialize` | RepeatingActivityConsecutiveMsgPacket | Decodes a received type-3 consecutive message before 00adba80 raises the wheel notification. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb820` | `RepeatingActivityCountRemainingInfoPacket_Unserialize` | RepeatingActivityCountRemainingInfoPack… | Decodes a received type-10 count-remaining info message before 00adb330 logs it. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb890` | `RepeatingActivityAddCountResultPacket_Unserialize` | RepeatingActivityAddCountResultPacket | Decodes a received type-7 addcount result before 00adb250 logs success or failure. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adbe60` | `RepeatingActivity_WriteHeader` |  | Serializes the [2-byte opcode][1-byte sub-id] header that every client-to-server repeating-activity message begins with (0x008f then the message type), inside a tunneled gateway packet. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc160` | `RepeatingActivityClearRequest_Serialize` | RepeatingActivityClearRequest | Body serializer for the type-4 clear request; called by the tunnel-and-send wrapper 00adc3c0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc1d0` | `RepeatingActivitySetConsecutivePacket_Serialize` | RepeatingActivitySetConsecutivePacket | Body serializer for the type-5 set-consecutive request; called by the tunnel-and-send wrapper 00adc4d0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad9740` | `— ⚠reject` | RepeatingActivityStatePacket | Constructs a type-2 state object; receive dispatcher 00adbf00 parses it and applies its fields through 00ada800. The base constructor is also reused by 00ada9b0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada120` | `— ⚠reject` | RepeatingActivityBaseAdminPacket | Shared admin-packet base used by outgoing type-6/8/9 requests and incoming type-7/10 result objects. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada200` | `— ⚠reject` | RepeatingActivityAddCountPacket | Builds the type-6 addcount request object used by 00adcbb0. This constructor does not establish a duplicated Guid layout; response logger 00adb250 reads +0xc/+0x10 and +0x18/+0x1c as separate groups. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada3d0` | `— ⚠reject` | RepeatingActivitySetCountPacket | Builds the type-8 'setcount' request; wrapper 00adcc40 serialises it and sends it through the gateway when connected. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada4b0` | `— ⚠reject` | RepeatingActivityGetCountRemainingPacket | Builds the type-9 query for a target's remaining count; wrapper 00adccd0 serialises it and sends it through the gateway when connected. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada590` | `— ⚠reject` | RepeatingActivityCountRemainingInfoPack… | Constructs the type-10 count-remaining info object that 00adbf00 parses and 00adb330 logs. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc270` | `— ⚠reject` | RepeatingActivityBaseAdminPacket | Shared body serializer for the admin requests type 6 (addcount), 8 (setcount) and 9 (count-remaining query), which is why three different send wrappers call it. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ad99e0` | `RepeatingActivitySetConsecutivePacket::RepeatingActivitySetConsecutivePacket` | send | PacketTunneledClientPacket |  | Hypothesis: the original server accepted a client-originated type-5 repeating-activity request and applied a consecutive-count operation. The server mutation and field semantics are not established here. Depends on requester 00adcb30, serializer 00adc1d0 and sender 00adc4d0. |
| `00ad9fd0` | `RepeatingActivityClearRequest::RepeatingActivityClearRequest` | send | PacketTunneledClientPacket |  | Hypothesis: the original server treated a type-4 request as a repeating-activity clear. Client callers use id -1 for clear-all (00adc9b0) or a matched entry id (00adca40); server scope and effects are not shown. Depends on serializer 00adc160 and sender 00adc3c0. |
| `00adc3c0` | `RepeatingActivity_SendClearRequest` | send | PacketTunneledClientPacket |  | Hypothesis: the gateway forwarded the tunneled payload to the world server, which cleared the repeating-activity state for the addressed activity; depends on the request builders 00adc9b0/00adca40 and on the serializer header 00adbe60. |
| `00adc4d0` | `RepeatingActivity_SendSetConsecutive` | send | PacketTunneledClientPacket |  | Hypothesis: the server applied the requested consecutive-day value to the target activity; depends on 00adcb30 and the body serializer 00adc1d0. |
| `00adc5e0` | `RepeatingActivity_SendAddCount` | send | PacketTunneledClientPacket |  | Hypothesis: the server incremented the stored count for the target Guid, later answering with a type-7 result logged by 00adb250; depends on 00adcbb0 and the receive path 00adbf00. |
| `00adc6f0` | `RepeatingActivity_SendSetCount` | send | PacketTunneledClientPacket |  | Hypothesis: the server overwrote the stored count for the target Guid/activity; depends on 00adcc40 and serializer 00adc270. |
| `00adc800` | `RepeatingActivity_SendGetCountRemaining` | send | PacketTunneledClientPacket |  | Hypothesis: the server answered with a type-10 info message that 00adb330 logs; depends on 00adccd0 and dispatcher case 10. |
| `00adbe60` | `RepeatingActivity_WriteHeader` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received repeating-activity traffic as a tunneled client packet whose body starts with the 2-byte id 143 and a 1-byte message type (4=clear, 5=set-consecutive, 6=addcount, 8=setcount, 9=query); depends on the send wrappers 00adc3c0-00adc800 and on the receive-side dispatcher 00adbf00. |
| `00adc160` | `RepeatingActivityClearRequest_Serialize` | send | PacketTunneledClientPacket |  | Hypothesis: the original server parsed a type-4 request from the bytes emitted here. The +0x18 field copy is bounded by available capacity and may be shorter than four bytes; the trailing +0x10 member is serialized separately. Depends on sender 00adc3c0 and constructor 00ad9fd0. |
| `00adc1d0` | `RepeatingActivitySetConsecutivePacket_Serialize` | send | PacketTunneledClientPacket |  | Hypothesis: the server parsed a type-5 body as two dwords (target/id and new consecutive count); depends on the send wrapper 00adc4d0 and constructor 00ad99e0. |
| `00ada200` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server processed a type-6 addcount command and returned a type-7 result. The request field mapping and server mutation are not proven; client log text calls the operation addcount and formats response fields as Guid, activity id and count. Depends on requester 00adcbb0, serializer 00adc270, dispatcher 00adbf00 and logger 00adb250. |
| `00ada3d0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server applied a type-8 setcount command. The exact state transition or response is not established by this constructor. Depends on requester 00adcc40, serializer 00adc270 and sender 00adc6f0. |
| `00ada4b0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server answered a type-9 count-remaining query with type-10 information consumed by 00adb330. Depends on requester 00adccd0, serializer 00adc270, sender 00adc800 and receive dispatcher 00adbf00. |
| `00adc270` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the server parsed the type-6/8/9 bodies as two dwords plus a trailing variable field (the 64-bit target Guid plus context); depends on wrappers 00adc5e0/00adc6f0/00adc800. |
