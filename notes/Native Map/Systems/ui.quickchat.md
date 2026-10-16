# System: ui.quickchat

24 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b933f0` | `ClientQuickChatManager::InternalDs::GetField` | ClientQuickChatManager::InternalDs | Serves a quick-chat data-source field to the UI table. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b935c0` | `ClientQuickChatManager::InternalEmoteItemCategoriesDs::GetField` | ClientQuickChatManager::InternalEmoteIt… | Serves emote-item category fields to the quick-chat UI source. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b96050` | `ClientQuickChatManager_RefreshPhraseData` | ClientQuickChatManager | vtable slot 1 of ClientQuickChatManager; refreshes the quick-chat data source and the Emotes/Favorites category rows after a data update. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96070` | `ClientQuickChatManager_InternalDs_Destruct` | ClientQuickChatManager::InternalDs | Destructor of the data source that backs BaseClient.QuickChatData; called from the scalar deleting destructor 00b96c80. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b960f0` | `ClientQuickChatManager_InternalEmoteItemCategoriesDs_Destruct` | ClientQuickChatManager::InternalEmoteIt… | Destructor of the data source that backs BaseClient.EmoteItemCategories; called from the scalar deleting destructor 00b96d20. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96c80` | `ClientQuickChatManager_InternalDs_DeletingDestruct` | ClientQuickChatManager::InternalDs | vtable slot 0 of ClientQuickChatManager::InternalDs. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96ca0` | `ClientQuickChatManager_InternalEmoteItemCategoriesDs_Construct` | ClientQuickChatManager::InternalEmoteIt… | Creates the data source registered under the name BaseClient.EmoteItemCategories by the manager constructor 00b96db0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96d20` | `ClientQuickChatManager_InternalEmoteItemCategoriesDs_DeletingDestruct` | ClientQuickChatManager::InternalEmoteIt… | vtable slot 0 of ClientQuickChatManager::InternalEmoteItemCategoriesDs. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96db0` | `ClientQuickChatManager_Construct` | ClientQuickChatManager | Creates the manager that serves the quick-chat phrase menu and the emote item categories to the chat window. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96f60` | `ClientQuickChatManager_DeletingDestruct` | ClientQuickChatManager | vtable slot 0 of ClientQuickChatManager. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b976a0` | `QuickChat_SendWhisperAndEcho` |  | Builds and sends the tell form of a quick-chat row and echoes the local whisper line to the chat window. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00cdf760` | `QuickChatAutoCompleteDataSource::Constructor` | QuickChatAutoCompleteDataSource | Constructs the quick-chat autocomplete data source. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdf8c0` | `QuickChatAutoCompleteDataSource::ScalarDeletingDestructor` | QuickChatAutoCompleteDataSource | MSVC scalar deleting destructor for QuickChatAutoCompleteDataSource. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b90c70` | `ParseCaretSeparatedResource` |  | Called by ClientQuickChatManager_Construct when loading Resources\EmoteItemCategories.txt; feature attribution is caller-derived. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b95280` | `Thunk_ThisPlusFF8_00b93f70` |  | Forwards a call to the SoeUtil string helper for the text-index subobject embedded at offset +0xff8 of the quick-chat manager; the +0xff8 object is the one constructed by 00b963c0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96170` | `QuickChat_BuildEmoteFavoritesRows` |  | Builds or refreshes the Emotes/Favorites quick-chat category rows (the two literals are the row names); called from the chat window code 00c19600/00c19650. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b96c00` | `ClientQuickChatManager_InternalDs_Construct ⚠downgrade` | ClientQuickChatManager::InternalDs | Creates the data source registered under the name BaseClient.QuickChatData by the manager constructor 00b96db0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97480` | `QuickChatSendChannel4` |  | Channel setter used from the chat window code 00c16d40; the receive-side dispatcher 00b966e0 maps channel 4 to its GroupSay handler. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00cdf820` | `ReleaseAutocompleteStringAndLists → QuickChatAutoCompleteDataSource::Destructor…` |  | Cleanup body reached by the QuickChatAutoCompleteDataSource scalar deleting destructor. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b97380` | `QuickChatSendChannel0 ? ⚠downgrade` |  | Channel setter used from the chat window code 00c16d00; the receive-side dispatcher 00b966e0 maps channel 0 to its WorldSay handler. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b95140` | `— ⚠reject` | ClientQuickChatManager | Rebuilds the quick-chat phrase labels after the phrase data was refreshed; reached from ClientQuickChatManager vtable slot 1 (00b96050), whose this is passed straight through, which is how the class binding is known. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b965d0` | `— ⚠reject` | ClientQuickChatManager | Destructor of the manager that owns BaseClient.QuickChatData, BaseClient.EmoteItemCategories and the autocomplete data source. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97580` | `— ⚠reject` |  | Channel setter used from the chat window code 00c1af80; the receive-side dispatcher 00b966e0 maps channel 9 to its GuildSay handler. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b978b0` | `— ⚠reject` |  | The chat tab enable/disable path: it pins the four channel numbers the shipped client uses for Area, Trade, LFG and Members-Only chat. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b976a0` | `QuickChat_SendWhisperAndEcho` | send | QuickChatSendTellPacket |  | Hypothesis: the original server routed the 67/2 tell to the target player and the client echoed it locally through this same path when the target was the local character; depends on the tell sender 00b97210 and the receiver 00b966e0. |
| `00b97480` | `QuickChatSendChannel4` | send | QuickChatSendChatToChannelPacket |  | Hypothesis: group chat selection; the original server delivered it to the sending character group, which is the GroupSay rule reconstructed in Research/Pieces/088. |
| `00b97380` | `QuickChatSendChannel0 ? ⚠downgrade` | send | QuickChatSendChatToChannelPacket |  | Hypothesis: this is the say channel selection, so the original server broadcast the message to the sender visible set (the WorldSay policy in Research/Pieces/088); depends on the channel-number decoding in 00b966e0. |
| `00b97580` | `— ⚠reject` | send | QuickChatSendChatToChannelPacket |  | Hypothesis: guild chat selection; the original server delivered it to the character guild (GuildSay rule in Research/Pieces/088). |
| `00b978b0` | `— ⚠reject` | send | QuickChatSendChatToChannelPacket |  | Hypothesis: the original server used the same four channel numbers when it addressed the channel toggles, so the number written into the 67/3 packet is the wire channel id rather than a local UI index; the receive-side numbering in 00b966e0 agrees for 6, 7, 8 and 10. |
