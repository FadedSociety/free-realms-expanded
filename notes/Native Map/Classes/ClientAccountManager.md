# Class: ClientAccountManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00705610` | `AccountManagerStateToString` | ✅ | function | Maps account-manager state values to the corresponding state-name strings, including server-list and character-creation states. |
| `00705a60` | `InvokeAccountManagerService ? ⚠rename` | 🟡 | method | Checks two flags, manager state, and the input value, calls an interface virtual, and logs through the account-manager logger. |
| `007073a0` | `ClientAccountManagerListNodeAtIndex` | 🟢 | method | Walks the indexed list at manager offset 0x22290 using node link offset 0x62b0 and returns the node or null. |
| `0070c8b0` | `ClientAccountManagerSetState` | ✅ | method | If the state changes, saves old state at +0x22274, writes the new state at +0x22270, and conditionally formats and logs the transition. |
| `0070ea10` | `ClientAccountManager::OnRequestFriendsList` | ✅ | virtual | Logs result and friend fields, copies entries into a linked list, then sets state 0xff. |
| `0070f490` | `ClientAccountManager::Initialize` | ✅ | method | Loads config and loading-screen state, initializes fields, loads four web-service error tables, and returns false if a table fails. |
| `0070fdc0` | `ClientAccountManager::ShowWebServiceErrorMessage` | ✅ | method | Stores continuation state and logs it. If the supplied row is null, logs Unknown web service error and skips popup/state changes. Otherwise selects text or message ID from the row, emits CharacterCreatePopups:Show and t… |
| `0070ff40` | `ClientAccountManager::ShowErrorMessage` | ✅ | method | Stores continuation state, starts popup UI, fills message/display parameters, sets popup fields, then requests state 0x33. |
| `007100c0` | `ClientAccountManager::OnRequestCharacterCreate` | ✅ | virtual | Logs result and appearance fields, maps failures to messages/states, and on success stores returned GUID then sets state 0x29. |
| `00710e20` | `ClientAccountManager::OnRequestCheckName` | ✅ | virtual | Logs names/suggestions, maps result codes to locale keys, and sets state 0x20 or 0x21 on available-name result. |
| `00711220` | `ClientAccountManager::OnRequestCharacterLogin` | ✅ | virtual | Logs result, GUID, server, address, ticket and key; failure displays popup, success stores credentials and sets state 0x2e. |
| `00711ca0` | `ClientAccountManager::OnRequestServerList` | ✅ | virtual | Logs servers and online/locked flags, copies only online unlocked entries, marks availability, and enters state 3. |
| `007140c0` | `ClientAccountManager::CreatePlayer` | ✅ | method | Looks up starting-set and clothing/model-customization mappings for the supplied creation values, logs unresolved definition IDs, fills a temporary character record with appearance data, inserts it into lookup/list stru… |
| `00715e30` | `ClientAccountManager::RemoveTrackedActorNode` | 🟢 | method | For a non-null node, passes the value at node+0x04 to 00985550; scans the chain headed at this+0x22290 via node+0x62b0 and unlinks a match with 00715d80; then scans this+0x53978 via each node's +0x04 link and calls 0070… |
| `00715ec0` | `ClientAccountManager::Run` | ✅ | method | Per-frame state machine of the account manager: loops while the state dword at this+0x22270 is below 0xfd and switches on it (cases 3,5,6,7,8,10,0xb..0x34), driving the login / server-select / character-select / charact… |
| `00719330` | `ClientAccountManager::OnAccountInfo` | ✅ | virtual | Result callback of the account-info query (this, context, result, payload pointer). For result 1 it copies payload[0], payload[1] and payload[2] into this+0x22280 (member flag), this+0x22284 (maximum characters) and thi… |
| `00719550` | `ClientAccountManager::OnRequestCharacterList` | ✅ | virtual | Reply callback for the account character list (this, context, result, list pointer). For result 1 or 3 it clears the selected-character pointer at *(this+0x22268)+200, zeroes the selection index this+0x53968, drains the… |
| `007198b0` | `— ⚠reject` | ⚪ | virtual | Character-delete reply callback (this, flag byte, guid low dword, guid high dword). It always logs OnRequestCharacterDelete: result=0x%0x and Guid=%llu. When the flag byte is nonzero it forgets the selection: clears *(t… |
| `0071a530` | `ClientAccountManager::Constructor` | ✅ | ctor | Initializes account data, character-detail list, arrays, strings, error maps, session fields, and configuration; logs when session id is zero. |
| `0071a8f0` | `ClientAccountManager::Destructor` | ✅ | dtor | Releases manager interfaces, strings, maps, arrays, character lists, data managers, and asset requester state. |
