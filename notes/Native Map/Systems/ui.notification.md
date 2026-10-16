# System: ui.notification

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a94640` | `DisplayErrorMessage` |  | Displays an error notification; no packet or server-side error policy is handled here. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a94780` | `DisplayAlertMessage` |  | Displays a colored alert; the default yellow value and optional argument are explicit in the body. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a94900` | `DisplaySystemMessage` |  | Formats a client system message with an optional follow-up callback. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a95940` | `NotifyAchievementCompleted` |  | Shows the client achievement-completion notification; award rules are not handled here. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00adba80` | `WheelMessage_Notify` |  | Raises the daily-wheel bonus-spin notification to the script/UI layer when a type-3 consecutive message arrives (dispatcher case 3). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00cb7190` | `NotificationQueueListCtor` | SoeUtil::List<NotificationQueue::Entry,… | Constructor for the NotificationQueue entry list; the final vftable is the List<NotificationQueue::Entry,8> one. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb7640` | `NotificationQueue::GiveTime` | NotificationQueue | Per-tick timer for the notification queue: starts the head notification after 100 ms and expires it after its duration at +0x20; logs a timeout when no end callback arrives within 10000 ms. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a92570` | `PopulateAndSubmitNotificationArgs` |  | Caller 00a931b0 uses this helper while preparing UpdateObjective presentation; this target does not identify the notification class. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a94ab0` | `FormatPlayerMessage` |  | Shared formatter used by friend, guild, trade and other callers; feature associations come from those callers. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cb7320` | `— ⚠reject` |  | Invokes the HandlerNotification:OnNotification script callback for a queued notification entry, with the entry's parameters. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb74f0` | `— ⚠reject` |  | Thiscall with four arguments: walks the list from receiver+4 following node+0x28; for each node tests node+0x18 against 1 and 2 (calling 0070b470 with 0 or 1 when below) and passes the node through 00979450 with the arg… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

