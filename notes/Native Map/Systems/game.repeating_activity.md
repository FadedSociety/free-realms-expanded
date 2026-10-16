# System: game.repeating_activity

26 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad95d0` | `RepeatingActivityPlayer::~RepeatingActivityPlayer` | RepeatingActivityPlayer | Destructor body of the client-side RepeatingActivityPlayer container (the object whose list of ClientRepeatingActivity entries is walked by 00ada790/00ada800/00adb400). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ad9de0` | `RepeatingActivityPlayer::ScalarDeletingDtor` | RepeatingActivityPlayer | Heap-delete path for RepeatingActivityPlayer objects. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adabc0` | `ClientRepeatingActivity::ClientRepeatingActivity` | ClientRepeatingActivity | Creates a real activity entry when the server's type-1 add message names an id the client does not have yet (caller 00adb990). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adac80` | `ClientRepeatingActivity::~ClientRepeatingActivity` | ClientRepeatingActivity | Non-deleting destructor of a client activity entry. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adad60` | `ClientRepeatingActivity::ScalarDeletingDtor` | ClientRepeatingActivity | Heap-delete path for client activity entries. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adad80` | `ClientRepeatingActivity::ClientRepeatingActivity` | ClientRepeatingActivity | Duplicates a ClientRepeatingActivity entry for append through 00adb900; string helper behavior is not named beyond the observed calls. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb400` | `ActivityList_LogAll` |  | Debug/diagnostic dump of the client's repeating-activity list, presumably a console or script command implementation. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008c06d0` | `ClientRepeatingActivity::ScalarDeletingDestructor` | ClientRepeatingActivity | Runs ClientRepeatingActivity destructor then conditionally deletes it. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `00ada6f0` | `GroupHandler::SetWheel` | GroupHandler | Updates one repeating-activity entry's wheel state (counts plus two refresh deadlines) and notifies the Lua/UI layer; called from the state-message consumer 00ada800 and the add-message consumer 00adb990. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada790` | `ActivityList_FindByName` |  | Name lookup used by the counting, deadline and clear paths (00ada830, 00ada8b0, 00adca40) to map an activity name to its ClientRepeatingActivity entry. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada800` | `ActivityList_ApplyStateMessage` |  | Applies a received type-2 repeating-activity state message to the client's local activity entry. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada830` | `ActivityList_CountForName` |  | Aggregates a repeating activity's count across the entries of a client-side activity list. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ada880` | `ActivityList_FindByValue58` |  | Small lookup on the repeating-activity list keyed by the element's +0x58 field (semantics of that field not established). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adab00` | `ClientRepeatingActivity::ClientRepeatingActivity ⚠downgrade` | ClientRepeatingActivity | Creates a placeholder entry in the client's repeating-activity list before real data arrives. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb530` | `ClientRepeatingActivity_CopyCtorWrapper` | ClientRepeatingActivity | Copy-construction entry point used by the list append helper 00adb900. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb900` | `ActivityList_AppendCopy` |  | Appends a copied activity entry to the list used by receive add/update handler 00adb990. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb990` | `RepeatingActivity_OnAddMessage` |  | Applies a received type-1 repeating-activity add/update message to the client's local activity list. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adc9b0` | `RepeatingActivity_RequestClearAll` |  | Clears every repeating activity for the local player (no id argument), using the same type-4 message as the by-name path. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adca40` | `ActivityList_ClearByName` |  | Client clear action that sends a request and resets count, consecutive and selected local fields. It does not reset the second deadline pair at +0x50/+0x54. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adcb30` | `RepeatingActivity_RequestSetConsecutive ? ⚠downgrade` |  | Client request that overwrites an activity's consecutive-day value (used by the daily-wheel streak logic and GM tooling). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adcbb0` | `RepeatingActivity_RequestAddCount ? ⚠downgrade` |  | Client request implementing the addcount admin command used by GM tooling/console. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adcc40` | `RepeatingActivity_RequestSetCount ? ⚠downgrade` |  | Client request implementing the setcount admin command. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adccd0` | `RepeatingActivity_RequestCountRemaining ? ⚠downgrade` |  | Client request implementing the count-remaining admin query. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ada8b0` | `— ⚠reject` |  | Computes the soonest wheel/activity refresh from the client's local entries, used to decide when to re-enable the activity. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb250` | `— ⚠reject` |  | Logs the outcome of an addcount admin command received as a type-7 result message (dispatcher case 7). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adb330` | `— ⚠reject` |  | Logs the answer to a count-remaining query received as a type-10 info message (dispatcher case 10). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00adc9b0` | `RepeatingActivity_RequestClearAll` | send | PacketTunneledClientPacket |  | Hypothesis: the server treated id -1 as clear-all for the sending character; depends on 00adc3c0 and constructor 00ad9fd0. |
| `00adca40` | `ActivityList_ClearByName` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received a type-4 clear request and updated corresponding server state. This function proves the client sends and resets local fields, not the server outcome. Depends on clear wrappers 00adc9b0/00adc3c0 and serializer 00adc160. |
| `00adcb30` | `RepeatingActivity_RequestSetConsecutive ? ⚠downgrade` | send | PacketTunneledClientPacket |  | Hypothesis: the server stored the consecutive value and echoed updated state; depends on 00adc4d0 and on the receive-side type-2/3 handlers. |
| `00adcbb0` | `RepeatingActivity_RequestAddCount ? ⚠downgrade` | send | PacketTunneledClientPacket |  | Hypothesis: the server executed addcount for the addressed Guid/activity and answered with a type-7 result (logged by 00adb250); depends on 00adc5e0 and dispatcher case 7. |
| `00adcc40` | `RepeatingActivity_RequestSetCount ? ⚠downgrade` | send | PacketTunneledClientPacket |  | Hypothesis: the server overwrote the stored count and answered with a result message in the same family; depends on 00adc6f0 and dispatcher. |
| `00adccd0` | `RepeatingActivity_RequestCountRemaining ? ⚠downgrade` | send | PacketTunneledClientPacket |  | Hypothesis: the server answered with a type-10 info message logged by 00adb330; depends on 00adc800 and dispatcher case 10. |
