# System: engine.datasource

34 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a3f940` | `WhatsNewDataSource::GetFieldValue` | WhatsNewDataSource | Field accessor that exposes one announcement (What's New) row to the client's data-binding system by field id; the header row is part of the same object that the announcement handler 00a40aa0 fills from AnnouncementData… | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a406c0` | `WhatsNewDataSource::~WhatsNewDataSource` | WhatsNewDataSource | Teardown of a What's New data source (the objects the announcements subsystem registers as BaseClient.Announcements/BaseClient.MemberPanel); reached through the scalar deleting destructor 00a40850. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a407c0` | `WhatsNewDataSource::WhatsNewDataSource` | WhatsNewDataSource | Data source registered by the announcements subsystem as BaseClient.Announcements and BaseClient.MemberPanel; the array pointer it stores is the Array<WhatsNewInfo> whose rows the GetFieldValue override 00a3f940 reads t… | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a40850` | `WhatsNewDataSource::ScalarDeletingDtor` | WhatsNewDataSource | Heap-delete entry point for the What's New data source objects held as RefPtrs by the announcements subsystem. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a41490` | `NudgeDataSource::GetFieldName` | NudgeDataSource | Field-name table of the nudge data source: it is the client's own naming of every nudge record column, which is the schema the NudgeUpdatePacket/announcement records are displayed through. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a41780` | `NudgeDataSource::GetField1C` | NudgeDataSource | NudgeDataSource accessor. The class binding is direct from vtable 01814280; the meaning of the returned owner-side dword is not established. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a41870` | `NudgePollShowWindow` |  | The nudge subsystem's timer tick that opens the nudge window once a nudge's delay has elapsed; the message name is the only in-function anchor, the surrounding object being the nudge subsystem constructed at 00a42010. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a419e0` | `NudgeDataSource::GetFieldValue` | NudgeDataSource | Column accessor that exposes nudge records (typically delivered by the 0xcb/4 nudge update path) to the UI binding system under the names returned by the field-name table 00a41490. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a41ee0` | `NudgeDataSource::~NudgeDataSource` | NudgeDataSource | Teardown of the data source registered as BaseClient.Nudges; reached through the scalar deleting destructor 00a41fe0. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a41f60` | `NudgeDataSource::NudgeDataSource` | NudgeDataSource | Data source registered as BaseClient.Nudges by the nudge subsystem constructor 00a42010 and read back by the data source machinery through the vtable. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a41fe0` | `NudgeDataSource::ScalarDeletingDtor` | NudgeDataSource | Heap-delete entry point for the BaseClient.Nudges data source held as a RefPtr by the nudge subsystem. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a1ad0` | `NameValueDataSource::ScalarDeletingDestructor` | NameValueDataSource | Heap-delete path for the NameValueDataSource object (a key/value data source used by UI bindings); the non-deleting destructor lives at 0098dcf0. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `009fa000` | `PopulateSelectedDataSourceValues` |  | Repopulates values for the selected key; caller 009fa350 stores the selection before invoking this routine. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00a41910` | `FormatIso8601Date` |  | Date rendering used by the nudge data source when it exposes the StartDate and EndDate columns (00a419e0 calls it for field ids 4 and 5), which shows those wire fields are six-dword UTC time blocks. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a42470` | `NotifyNudgesMatchingKey` |  | Turns a game event key (param_1) into nudge tracking notifications: the two tracking-type columns and their parameter columns come from the field-name table 00a41490, so the integer semantics are the client's own naming. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a42540` | `NotifyNudgesTrackingType2` |  | Tracking-type-2 notification sweep, i.e. the class of nudge condition the client reports with kind 2; called from 00b342f0. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00cc39a0` | `ClearFlag2B0AndTailDispatch` |  | Clears the object's byte flag at +0x2b0 and delegates to its slot at vtable offset 0x10; caller-specific meaning is unresolved. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc3a10` | `RefreshAndSetFlag258` |  | Runs the nested object's virtual operation and marks byte offset 0x258 set. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc3c70` | `RebuildFilteredLinkedList` |  | Rebuilds a filtered node list from the source chain; the caller-controlled byte +0x2b1 decides whether records with a clear source byte are excluded. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc55d0` | `ClearFlag25CAndTailDispatch` |  | Clears a byte flag before delegating to the object's slot at vtable offset 0x10. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b8c440` | `SetRowFlag ?` |  | Sets a boolean-like field on a matching data-source row. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00b8c470` | `ClearRowFlag ?` |  | Clears a boolean-like field on a matching data-source row. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00c21610` | `LoadLobbyGameUiConfig ? ⚠rename` |  | Loads configuration fields for lobby game UI from serialized input. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## ⚪ unknown (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b0e1c0` | `—` |  | Loads the second argument's offset-4 value through an indirect datasource call using a nested object at offset 8. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0e1e0` | `— ⚠reject` |  | Reads a stored handle, obtains two fixed helper values, and passes them with the handle to a virtual function at offset 0x14. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b8c4a0` | `— ⚠reject` |  | Updates one row's index and renumbers the remaining active rows. | [[Native Map/Shards/S0358 00b8a640-00b906a0|S0358]] |
| `00c1fb40` | `— ⚠reject` |  | Looks up and type-checks an item-stats GUI data source by name. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c1fbb0` | `— ⚠reject` |  | Resolves and type-checks a single-item-definition GUI data source by name. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c1fc20` | `— ⚠reject` |  | Resolves and type-checks a tint-group GUI data source by name. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00cbcfa0` | `— ⚠reject` |  | If the selected value differs from the stored value at index 0x9a, stores it and invokes virtual callbacks on the object and two linked objects, with FUN_00cbc6d0 on the second object. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbd000` | `— ⚠reject` |  | Checks the selected entry's two keys against the arguments; on a match it calls FUN_00cbc6d0, creates an entry from two supplied values, and forwards a nonnegative result to FUN_010cb3a0. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbefe0` | `— ⚠reject` |  | Creates an exception frame, calls FUN_0091bf60 with the stack-cookie-derived argument, restores the exception list, and returns. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf120` | `— ⚠reject` |  | Creates an exception frame, calls FUN_0091bf60 with the stack-cookie-derived argument, restores the exception list, and returns. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc3a00` | `— ⚠reject` |  | Clears the object's byte flag at +0x2b0 and delegates to its slot at vtable offset 0x10; caller-specific meaning is unresolved. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a42470` | `NotifyNudgesMatchingKey` | send | PacketTunneledClientPacket |  | Hypothesis: nudge progress is reported to the server as keyed tracking events (a definition pair plus a tracking type), not as free text; the server then decided whether the nudge was complete. Depends on the field-name table 00a41490 and the sender 00a42330. |
| `00a42540` | `NotifyNudgesTrackingType2` | send | PacketTunneledClientPacket |  | Hypothesis: type-2 nudges are reported in bulk when some client-side condition happens (the caller 00b342f0), each carrying its definition pair; the server's per-record reaction is not observable. |
