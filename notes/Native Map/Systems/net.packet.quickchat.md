# System: net.packet.quickchat

22 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b8ad10` | `BaseQuickChatPacket::CtorWithSubtype` | BaseQuickChatPacket | Initializes a quick-chat packet with a caller-selected subtype. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8ad80` | `BaseQuickChatPacket::Dtor` | BaseQuickChatPacket | Unwinds the quick-chat packet object's inheritance layers. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8add0` | `BaseQuickChatPacket::ScalarDeletingDtor` | BaseQuickChatPacket | Provides MSVC scalar-deleting destruction for the packet base. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8aeb0` | `BaseQuickChatPacket::DtorBase` | BaseQuickChatPacket | Performs base cleanup for quick-chat packet subclasses. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b90880` | `QuickChatSendChatPacketBase::Ctor` | QuickChatSendChatPacketBase | Constructs the base representation used by quick-chat packet objects. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b908e0` | `QuickChatSendChatPacketBase::CtorWithSubtype` | QuickChatSendChatPacketBase | Initializes the common quick-chat packet representation for a supplied subtype. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b90950` | `QuickChatSendChatPacketBase::DeletingDtor` | QuickChatSendChatPacketBase | Scalar deleting destructor for QuickChatSendChatPacketBase. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b90970` | `QuickChatSendTellPacket::Ctor` | QuickChatSendTellPacket | Constructs the tell subtype's packet object. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b909f0` | `QuickChatSendTellPacket::DeletingDtor` | QuickChatSendTellPacket | Scalar deleting destructor for QuickChatSendTellPacket. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b90a10` | `QuickChatSendChatToChannelPacket::Ctor` | QuickChatSendChatToChannelPacket | Constructs the chat-to-channel subtype's packet object. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b90a80` | `QuickChatSendChatToChannelPacket::DeletingDtor` | QuickChatSendChatToChannelPacket | Scalar deleting destructor for QuickChatSendChatToChannelPacket. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93ad0` | `BaseQuickChatPacket::ReadBaseFields` | BaseQuickChatPacket | Inbound base-packet field parser called by the quick-chat dispatcher. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b966e0` | `BaseQuickChatPacket_Unpack` | BaseQuickChatPacket | Client receive path for the community BaseQuickChatPacket (opcode 67): it turns the wire message into either a data-table update, a whisper echo or a chat-channel message handled by the matching channel routine. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b90750` | `ReadCountedHashRecords` |  | Reader helper for the inbound BaseQuickChatPacket payload path; packet association comes from caller 00b96450. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b96450` | `QuickChatPacketBase_ReadChatPayload` |  | Reader for the name-bearing quick-chat payloads used by the subtype-1 branch of the dispatcher 00b966e0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96f80` | `QuickChatPacket_SerializeChatFields` |  | Body writer used by the chat-to-channel sender 00b97100, whose callers construct QuickChatSendChatToChannelPacket objects. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97210` | `QuickChat_SendTellPacket` |  | Send path for the tell/whisper quick-chat packet; its only caller 00b976a0 constructs QuickChatSendTellPacket (00b90970) and passes it here. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b95450` | `QuickChat_ReadDataPayload ? ⚠rename` |  | Reader for the quick-chat data payload; called from the BaseQuickChatPacket dispatcher 00b966e0 for the subtype-1 branch (QuickChatSendDataPacket shape). | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b8acb0` | `— ⚠reject` | BaseQuickChatPacket | Initializes the base quick-chat packet representation. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8ae40` | `— ⚠reject` | QuickChatSendDataPacket | Initializes a quick-chat send-data packet. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b93b50` | `— ⚠reject` |  | Subtype payload parser reached through 00b95450 and inbound dispatch 00b966e0; packet association comes from that caller chain. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b97100` | `— ⚠reject` |  | Send path used by the channel setters 00b97380/00b97480/00b97580 and by the chat toggle 00b978b0, each of which constructs a QuickChatSendChatToChannelPacket (00b90a10) in its own frame; the packet class is therefore th… | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b8ad10` | `BaseQuickChatPacket::CtorWithSubtype` | send | QuickChatSendDataPacket |  |  |
| `00b93ad0` | `BaseQuickChatPacket::ReadBaseFields` | recv | BaseQuickChatPacket |  | Hypothesis: a compatible server must send the base fields consumed by this receive path; depends on dispatcher 00b966e0, not proof of original server behavior. |
| `00b966e0` | `BaseQuickChatPacket_Unpack` | recv | BaseQuickChatPacket |  | Hypothesis: the original server sent opcode 67 messages whose subtype word selected these three shapes (1 = quick-chat phrase data table, 2 = whispered quick chat, 3 = chat-to-channel) and whose channel word reused the 0/4/6/7/8/9/10 numbering seen in the client switch; the emote/say broadcast depends on the separate senders 00b97100/00b97210 in this same shard. |
| `00b90750` | `ReadCountedHashRecords` | recv | BaseQuickChatPacket |  | Hypothesis: a compatible server must send the counted record block accepted by this reader; depends on 00b96450 and dispatcher 00b966e0, not proof of original server behavior. |
| `00b97210` | `QuickChat_SendTellPacket` | send | QuickChatSendTellPacket |  | Hypothesis: the original server routed these 67/2 tell messages to the named recipient and answered with the whisper-bearing 67 message that the subtype-2 branch of 00b966e0 consumes. |
| `00b8acb0` | `— ⚠reject` | send | QuickChatSendDataPacket |  |  |
| `00b8ae40` | `— ⚠reject` | send | QuickChatSendDataPacket |  |  |
| `00b93b50` | `— ⚠reject` | recv | QuickChatSendDataPacket |  | Hypothesis: a compatible server must send fields consumed by the subtype-1 receive path; depends on 00b95450 and 00b966e0 and does not establish original server behavior. |
| `00b97100` | `— ⚠reject` | send | QuickChatSendChatToChannelPacket |  | Hypothesis: the original server treated these 67/3 chat-to-channel messages as the quick-chat form of a channel broadcast and re-distributed them to the channel audience; the receiver side is the subtype-3 branch of 00b966e0. |
