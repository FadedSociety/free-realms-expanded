# System: net.chat

61 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0092f380` | `SendChatBubbleColor` |  | Lua-facing chat color setter also updates local client color state; its caller name and packet constructor anchor the feature. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `00b11600` | `DeserializePacketChatBody` | PacketChat | Decodes the PacketChat body used by the client's BaseChatPacket subtype-1 path. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b117a0` | `DeserializeTellEchoBody` |  | Decodes the TellEchoPacket body before the chat dispatcher formats the local tell echo. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b12ad0` | `WriteTwoU16Fields` |  | Writes the first two 16-bit fields of the PacketChat serializer's data stream. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b140c0` | `ReadPacketChatPayload` |  | Checked receive wrapper for PacketChat subtype 1. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b14130` | `ReadTellEchoPayload` |  | Checked receive wrapper for TellEchoPacket subtype 5. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b14350` | `BaseChatPacketReceiveDispatcher` |  | Receives and dispatches the client's BaseChatPacket variants, including PacketChat and TellEchoPacket. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b15450` | `ClientChatProcessor::ClientChatProcessor` | ClientChatProcessor | Creates the client chat processor; the command table registered here is what makes the nine typed chat commands dispatchable, and the four ChatSpeed values are the thresholds checked in 00b164b0. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b156d0` | `ClientChatProcessor::ScalarDeletingDtor` | ClientChatProcessor | Delete-through-base-pointer helper for the chat processor; slot [1] is the virtual destructor slot because slot [0] is the command handler 00b164b0. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b160b0` | `ClientChatProcessor::CommandGuildSay` | ClientChatProcessor | Implements guild chat, registered as command id 7 and sent on channel 9 with the guild guid filled in, which is why the 15/1 record carries a guild guid field. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `013c0860` | `UChatCommand_RequestFriendConfirmReciprocate::UChatCommand_RequestFriendConfirm…` | UChatCommand_RequestFriendConfirmRecipr… | Constructs one request-friend-confirm-reciprocate UChat command with an empty payload and the flag set to 1. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0900` | `UChatCommand_RequestFriendConfirmReciprocate::GetName` | UChatCommand_RequestFriendConfirmRecipr… | Name getter that identifies this UChat command class by its name string. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0af0` | `UChatCommand_RequestFriendConfirmReciprocate::ScalarDeletingDtor` | UChatCommand_RequestFriendConfirmRecipr… | Scalar deleting destructor of the request-friend-confirm-reciprocate UChat command. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0c00` | `UChatCommand_RequestFriendConfirmReciprocate::CopyFieldsTo` | UChatCommand_RequestFriendConfirmRecipr… | Copies this command's fields into another UChat command object (the target is cast to the request-friend-confirm class). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0c80` | `UChatCommand_RequestFriendConfirmReciprocate::TraceDestination` | UChatCommand_RequestFriendConfirmRecipr… | Trace-logs the destination of this UChat command (account id and destination name and server) on the send path. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0d70` | `UChatCommand_RequestFriendConfirmReciprocate::TraceSend` | UChatCommand_RequestFriendConfirmRecipr… | Emits the send trace line for this UChat command. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0e20` | `Engine::PendingFriendRequest::PendingFriendRequest` | Engine::PendingFriendRequest | Constructs one pending friend-request record: four text fields and a flag byte, allocated by AllocAndPushRecord (013c0590). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0eb0` | `Engine::PendingFriendRequest::~PendingFriendRequest` | Engine::PendingFriendRequest | Destructor body of the pending friend-request record, run from its scalar deleting destructor (013c0f90). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0f90` | `Engine::PendingFriendRequest::ScalarDeletingDtor` | Engine::PendingFriendRequest | Scalar deleting destructor of the pending friend-request record. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c06b0` | `UChatCommand_RequestFriendConfirmReciprocate::EncodeMessage` | UChatCommand_RequestFriendConfirmRecipr… | Encodes the request-friend-confirm-reciprocate UChat message body; the 0xd2 constant is written first. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0800` | `UChatCommand_RequestFriendConfirmReciprocate::Serialize` | UChatCommand_RequestFriendConfirmRecipr… | Outbound field encoding for the request-friend-confirm-reciprocate UChat command (mirror of Deserialize, slot [9]). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1350` | `Engine::PendingFriendRequest::SetField04` | Engine::PendingFriendRequest | Sets the first text field of the pending friend-request record during AllocAndPushRecord (013c0590). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1370` | `Engine::PendingFriendRequest::SetField20` | Engine::PendingFriendRequest | Sets the second text field of the pending friend-request record during AllocAndPushRecord (013c0590). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1390` | `Engine::PendingFriendRequest::SetField3C` | Engine::PendingFriendRequest | Sets the third text field of the pending friend-request record during AllocAndPushRecord (013c0590). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c13b0` | `Engine::PendingFriendRequest::SetField58` | Engine::PendingFriendRequest | Sets the fourth text field of the pending friend-request record during AllocAndPushRecord (013c0590). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟡 speculative (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b15340` | `SendChatPacket ? ⚠downgrade` |  | Puts a client-composed chat record on the wire as the tunnelled 15/1 PacketChat family; all ten command senders route through it. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b16350` | `ClientChatProcessor::CommandWhisper ? ⚠rename` | ClientChatProcessor | Sends a whisper-channel line for a target entity object, the shape of a client echo or auto-answer path; it is not in the command switch of 00b164b0 and has no direct caller, so it is reached through a stored callback. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00be83c0` | `ChatWStringFieldUpperProcess ?` |  | When 008f11e0 returns 1, copies the wide string at param_1+4 into a local SoeUtil::IString<unsigned_short> through 00701290, calls 00836c70 on that string, calls 00701170(1), upper-cases the constant string DAT_01bead08… | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `013c0230` | `NameLookupOrInsert ? ⚠rename` |  | When the object at param_2 has no text in +0x14, it takes the text from FUN_013a3a30(); otherwise it copies param_2 itself. It then tests that text with FUN_013bf950(iVar2, param_1, text). When the test fails it calls F… | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c06a0` | `SetFlagByte48 ?` |  | Stores the 1-byte argument param_2 at this+0x48 and returns (bytes 8a 44 24 04 / 88 41 48 / c2 04 00: mov al,[esp+4]; mov [ecx+0x48],al; ret 4). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0910` | `UChatCommand_RequestFriendConfirmReciprocate::GetSharedDword ?` | UChatCommand_RequestFriendConfirmRecipr… | Virtual slot [12] of the class: returns the 32-bit global DAT_01cbdd74 (bytes a1 74 dd cb 01 c3: mov eax,[0x01cbdd74]; ret). Slot [13] writes the same global. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0920` | `UChatCommand_RequestFriendConfirmReciprocate::SetSharedDword ?` | UChatCommand_RequestFriendConfirmRecipr… | Virtual slot [13] of the class: stores its 32-bit argument into the global DAT_01cbdd74 (bytes 8b 44 24 04 a3 74 dd cb 01 c2 04 00: mov eax,[esp+4]; mov [0x01cbdd74],eax; ret 4). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0ad0` | `RegisterFactory0x1fe ?` |  | Bytes 68 60 0a 3c 01 / 68 fe 01 00 00 / e8 / 8b c8 / e8 / c3: pushes the address of FUN_013c0a60 and the key 0x1fe, calls FUN_013ce130(0x1fe, FUN_013c0a60), moves the returned object into ecx and calls FUN_013ce230 on i… | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0b10` | `SetStringAt4C ? → UChatCommand_RequestFriendConfirmReciprocate::SetStringAt4C` |  | Assigns the argument string to the std::string at param_1+0x4c (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 4c / e8 / c2 04 00). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0b30` | `SetStringAt68 ?` |  | Assigns the argument string to the std::string at param_1+0x68 (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 68 / e8 / c2 04 00). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0d90` | `GetSubobjectAt4 ?` |  | Returns this+4 (bytes 8d 41 04 c3: lea eax,[ecx+4]; ret). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0da0` | `GetSubobjectAt20 ?` |  | Returns this+0x20 (bytes 8d 41 20 c3: lea eax,[ecx+0x20]; ret). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0db0` | `GetSubobjectAt3C ?` |  | Returns this+0x3c (bytes 8d 41 3c c3: lea eax,[ecx+0x3c]; ret). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0dc0` | `GetSubobjectAt58 ?` |  | Returns this+0x58 (bytes 8d 41 58 c3: lea eax,[ecx+0x58]; ret). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0dd0` | `SetFlagByte74 ?` |  | Stores the 1-byte argument at this+0x74 and returns (bytes 8a 44 24 04 / 88 41 74 / c2 04 00: mov al,[esp+4]; mov [ecx+0x74],al; ret 4). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c13d0` | `FieldPairMatchEitherOrder ? ⚠downgrade` |  | Two-field key match in either order; the list scan in 013bfce0 is its only visible caller. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1480` | `SetByteAt3D ? → LoginCommand_RegisterUsername::SetCreatingFlag` |  | Stores the 1-byte argument at this+0x3d (bytes 8a 44 24 04 / 88 41 3d / c2 04 00: mov al,[esp+4]; mov [ecx+0x3d],al; ret 4). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## ⚪ unknown (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b14fa0` | `— ⚠reject` | PacketChat | Serializes the client PacketChat request before the shared chat send helper wraps it in a gateway packet. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b151d0` | `— ⚠reject` | ClientChatProcessor | Shutdown path of the client chat processor that owns the command registrations, the spam list and the received-chat-node map. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b15710` | `— ⚠reject` | ClientChatProcessor | Implements the plain say chat line: the packet it builds is the 15/1 PacketChat request on channel 0. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b15870` | `— ⚠reject` | ClientChatProcessor | Implements the yell command, which is registered as command id 3 and sent on channel 5. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b159d0` | `— ⚠reject` | ClientChatProcessor | Implements the trade-chat command, registered as command id 6 and sent on channel 6. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b15b30` | `— ⚠reject` | ClientChatProcessor | Implements the looking-for-group command, registered as command id 4 and sent on channel 7. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b15c90` | `— ⚠reject` | ClientChatProcessor | Implements the members-only chat command, registered as command id 8 and sent on channel 10; the dispatcher 00b164b0 additionally gates this handler on the membership flag at (*(this+4))+0xc58. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b15df0` | `— ⚠reject` | ClientChatProcessor | Implements the area chat command, registered as command id 5 and sent on channel 8; channel 8 is the one the serializer 00b14fa0 gives a trailing extra field. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b15f50` | `— ⚠reject` | ClientChatProcessor | Implements group chat, registered as command id 1 and sent on channel 4. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b16220` | `— ⚠reject` | ClientChatProcessor | Implements the tell/whisper command, registered as command ids 2 for both tell and whisper, using the recipient record the command layer already resolved. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b164b0` | `— ⚠reject` | ClientChatProcessor | The chat command dispatcher: the command processor invokes this virtual with the parsed command id and argument, so this is where every typed chat command becomes a packet. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `01281de0` | `— ⚠reject` | Chat::ChatSendTextToLobbyDisplayAction | Chat log update for a text sent to lobby chat: records the text, disposition, conversation and sender names in an UpdateChatLog request. | [[Native Map/Shards/S0667 0127d1b0-01281de0|S0667]] |
| `013bfce0` | `— ⚠reject` |  | Walks the list of entries whose begin and end pointers are at the object's +0x20 and +0x24 and tests each entry with FUN_013c13d0 (FieldPairMatchEitherOrder) against two strings built from FUN_013a3a20/FUN_013a3a30 and … | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0590` | `— ⚠reject` |  | Creates one pending record from four text arguments and a flag, then appends it to the pending list at this+0x1c. The 0x78 record type is inferred from its constructor FUN_013c0e20, whose vtable is Engine::PendingFriend… | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0770` | `— ⚠reject` | UChatCommand_RequestFriendConfirmRecipr… | Inbound field decoding for the request-friend-confirm-reciprocate UChat command. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0940` | `— ⚠reject` | UChatCommand_RequestFriendConfirmRecipr… | Destructor body of the request-friend-confirm-reciprocate UChat command, run from the scalar deleting destructor and from EH unwind. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c09f0` | `— ⚠reject` |  | Allocates a 0x84-byte block with FUN_00d833a6; if the allocation succeeds it constructs a UChatCommand_RequestFriendConfirmReciprocate with FUN_013c0860 (otherwise the result is 0). It then calls virtual slot [0x48/4 = … | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0a60` | `— ⚠reject` |  | Allocates a 0x84-byte block with FUN_00d833a6 and, when that succeeds, constructs a UChatCommand_RequestFriendConfirmReciprocate with FUN_013c0860 and returns it; returns 0 on allocation failure. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c0b50` | `— ⚠reject` |  | Builds a temporary std::string with FUN_01249400(temp, this+0x4c, &DAT_01800bfc, cookie), then calls FUN_012325d0(param_2, temp, this+0x68) to combine it with the string at +0x68 into the output param_2, frees the temp … | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `0092f380` | `SendChatBubbleColor` | send | CommandPacketSetChatBubbleColor |  |  |
| `00b11600` | `DeserializePacketChatBody` | recv | PacketChat |  |  |
| `00b117a0` | `DeserializeTellEchoBody` | recv | TellEchoPacket |  |  |
| `00b12ad0` | `WriteTwoU16Fields` | send | PacketChat |  |  |
| `00b140c0` | `ReadPacketChatPayload` | recv | PacketChat |  |  |
| `00b14130` | `ReadTellEchoPayload` | recv | TellEchoPacket |  |  |
| `00b14350` | `BaseChatPacketReceiveDispatcher` | recv | BaseChatPacket |  | Hypothesis: the original server supplied type-15 chat packet variants that this client parses and dispatches. This depends on the receive switch, concrete packet constructors, and the pinned packet catalogue; original server behavior is not directly observed. |
| `00b14350` | `BaseChatPacketReceiveDispatcher` | recv | PacketChat |  | Hypothesis: the original server supplied type-15 chat packet variants that this client parses and dispatches. This depends on the receive switch, concrete packet constructors, and the pinned packet catalogue; original server behavior is not directly observed. |
| `00b14350` | `BaseChatPacketReceiveDispatcher` | recv | ChatPacketDebugChat |  | Hypothesis: the original server supplied type-15 chat packet variants that this client parses and dispatches. This depends on the receive switch, concrete packet constructors, and the pinned packet catalogue; original server behavior is not directly observed. |
| `00b14350` | `BaseChatPacketReceiveDispatcher` | recv | ChatPacketFromStringId |  | Hypothesis: the original server supplied type-15 chat packet variants that this client parses and dispatches. This depends on the receive switch, concrete packet constructors, and the pinned packet catalogue; original server behavior is not directly observed. |
| `00b14350` | `BaseChatPacketReceiveDispatcher` | recv | TellEchoPacket |  | Hypothesis: the original server supplied type-15 chat packet variants that this client parses and dispatches. This depends on the receive switch, concrete packet constructors, and the pinned packet catalogue; original server behavior is not directly observed. |
| `00b160b0` | `ClientChatProcessor::CommandGuildSay` | send | PacketChat |  | Hypothesis: guild chat reached the original server as 15/1 on channel 9 with the guild guid attached, and the server verified that the sender belonged to that guild before fanning out. Depends on note 088, note 090 (guild lifecycle) and 00b18380, which the dispatcher calls before this handler. |
| `00b15340` | `SendChatPacket ? ⚠downgrade` | send | PacketChat |  | Hypothesis: the original server accepted this 15/1 message, re-checked the sender against the claimed channel (guild membership for channel 9, member status for channel 10) and fanned it out to the channel recipients; the client-side spam gate at 00b164b0 means the server could not assume the client throttled. Depends on note 088 and the 129 guild family handling behind 00b18380. |
| `00b16350` | `ClientChatProcessor::CommandWhisper ? ⚠rename` | send | PacketChat |  | Hypothesis: this path emitted a whisper-channel record (channel 3) to the original server, which for a message from one player to another would treat the guids and names as the routing key. Depends on note 088 and the 15/5 TellEcho path that supplies the target object. |
| `00b14fa0` | `— ⚠reject` | send | PacketChat |  | Hypothesis: the original server accepted the client's PacketChat requests. This depends on the native send-helper path and the pinned 15/1 packet record; no original server implementation is available. |
| `00b15710` | `— ⚠reject` | send | PacketChat |  | Hypothesis: a say line reached the original server as a 15/1 record with channel 0 and was echoed to nearby players. Depends on note 088 and 00b15340. |
| `00b15870` | `— ⚠reject` | send | PacketChat |  | Hypothesis: the yell line reached the original server as 15/1 on channel 5 and was broadcast wider than say; the wider reach is the server's decision and is not visible in this handler. Depends on note 088 and 00b15340. |
| `00b159d0` | `— ⚠reject` | send | PacketChat |  | Hypothesis: trade chat reached the original server as 15/1 on channel 6 and was delivered to players with the trade channel enabled. Depends on note 088 and 00b15340. |
| `00b15b30` | `— ⚠reject` | send | PacketChat |  | Hypothesis: an LFG line reached the original server as 15/1 on channel 7 and was delivered to players with that channel on. Depends on note 088 and 00b15340. |
| `00b15c90` | `— ⚠reject` | send | PacketChat |  | Hypothesis: members-only chat reached the original server as 15/1 on channel 10 and was delivered only to subscribed members; the client's own membership flag at (*(this+4))+0xc58 is the only gate seen here. Depends on note 088 and 00b15340. |
| `00b15df0` | `— ⚠reject` | send | PacketChat |  | Hypothesis: area chat reached the original server as 15/1 on channel 8 together with the extra trailing field the serializer emits for that channel. Depends on note 088 and 00b15340. |
| `00b15f50` | `— ⚠reject` | send | PacketChat |  | Hypothesis: group chat reached the original server as 15/1 on channel 4 and the server resolved the group from its own party state, since the client only sends the channel plus its own guid pair. Depends on note 088, note 089 (party lifecycle) and 00b15340. |
| `00b16220` | `— ⚠reject` | send | PacketChat |  | Hypothesis: a tell reached the original server as 15/1 on channel 1 with both guids and names filled; the server matched the recipients by guid and did not route it to any channel audience. Depends on note 088 and 00b15340. |
