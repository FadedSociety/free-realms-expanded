# System: ui.chat

22 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008edac0` | `ChatShowSystemMessage` |  | Shows an incoming system message in the chat sink. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `00901140` | `StartChatLuaCall` |  | Starts chat through the Lua handler ChatHandler:StartChat and reports a missing-function error when the handler call fails. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00901a50` | `OnReceiveEmote` |  | Lua callback for an emote event, the emote counterpart of OnReceiveSay (009012b0). No emote packet class in the catalogue is linked, so the wire side is open. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00902730` | `BaseClient::SetChatText` | BaseClient | Sets the chat text through the Lua ChatHandler:SetChatText handler and reports a missing-function error otherwise. Vtable slot 33 is shared with PlayClient; named for BaseClient. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00b11810` | `OnReceiveChannel` |  | Presents a received channel message through the chat UI callback path. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b11ab0` | `OnReceiveWhisper` |  | Presents a received whisper and exposes its sender/message to the chat hook. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b120e0` | `TellEchoHandler` |  | Handles the parsed tell-echo packet and presents the sender's local confirmation. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b12b70` | `CharSequenceEditDistance` |  | Computes the character edit-distance score used by ClientChatProcessor's duplicate-message gate; the caller normalizes it by message length. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00b8af00` | `QuickChatInternalDs::ColumnName` | ClientQuickChatManager::InternalDs | Exposes quick-chat row fields through the internal data source. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8afc0` | `QuickChatCategoryDs::ColumnName` | ClientQuickChatManager::InternalEmoteIt… | Exposes the quick-chat emote category row schema. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8c400` | `InternalEmoteItemCategoriesDs::GetNestedFieldFC0` | ClientQuickChatManager::InternalEmoteIt… | Provides one value from the emote-category data source. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00be89f0` | `AIAlertNotification_Handle` |  | Handles an AI-alert (play-time) notification: logs it and drives the chat-log alert button and message through Lua events. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008edba0` | `DisplaySystemMessageDeduped` |  | Suppresses an identical system message repeated within 6 seconds before calling DisplaySystemMessage. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008f0cc0` | `ShowFlaggedSystemMessage` |  | When the flag at this+0x8e is set, sends the message through the deduplicated system-message path with color string 0x990000. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `00b14200` | `StoreChatHistoryEntry` |  | Maintains the bounded chat history associated with the key at this+0x4c. | [[Native Map/Shards/S0335 00b109b0-00b150b0|S0335]] |
| `00c167f0` | `SetChatColorFgBgSz` |  | Sets chat text colours (foreground, background) and size for a chat channel; the name is the table row 0x0182e0d8 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16d80` | `ProcessQuickChatChannelCommand` |  | Sends the quick-chat text on a named channel; the name is the table row 0x0182e3e0 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1b5a0` | `SendGuildSayChat ? ⚠rename` |  | Guild chat send path: both callers are guild message handlers (00b1d7d0, 00b1ee10) and the only domain helper it drives is FUN_00b10160, which owns the "GuildSay" chat-channel string; the label rests on that callee's ro… | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009012b0` | `— ⚠reject` |  | Lua callback for a chat line on a say-type channel (OnReceiveSay), with the channel name object supplied by the caller. Callers are lazy initialisers for the channel IStrings WorldSay, WorldShout, WorldTrade, WorldLfg, … | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00b8c390` | `— ⚠reject` | ClientQuickChatManager::InternalDs | Performs internal quick-chat data-source row cleanup. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00c16d00` | `— ⚠reject` |  | Executes the quick-chat entry with the given index from the quick-chat menu; the name is the table row 0x0182e3c0 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16d40` | `— ⚠reject` |  | Executes the quick-chat entry in the group-chat variant; the name is the table row 0x0182e3c8 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b120e0` | `TellEchoHandler` | recv | TellEchoPacket |  |  |
