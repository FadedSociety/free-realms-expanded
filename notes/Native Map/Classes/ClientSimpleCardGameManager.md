# Class: ClientSimpleCardGameManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad1590` | `ClientSimpleCardGameManager::Ctor` | ✅ | ctor | Calls the base manager constructor, installs two manager vtable pointers, zeros internal state fields, and stores two global defaults. |
| `00ad1630` | `SetCardsEnabledFlag ? ⚠downgrade` | 🟡 | virtual | Sets byte +0x2bbb8 to 1 and returns. |
| `00ad1a30` | `StartManagerStateIfIdle` | ✅ | virtual | When +0x40 is zero, sets state +0x2ba90 to 1, stores the current clock pair at +0x5ef50/+0x5ef54, and caches a global pointer at +0x28 if absent. |
| `00ad1a80` | `HandleTcgLaunchTicket` | ✅ | virtual | Accepts launch fields only in state 5, logs and returns otherwise, updates timed state when either first field is nonzero, dispatches campaign-start UI for positive count, and stores the two ticket fields. |
| `00ad1ef0` | `ClientSimpleCardGameManager::Shutdown` | ✅ | method | Logs tickets and station, removes a matching emitter and TcgMainWindow, clears the tree and references, resets the state timestamp, hides the TCG handler, and frees two stored objects. |
| `00ad22c0` | `ClientSimpleCardGameManager::Dtor` | ✅ | dtor | Restores both manager vtable pointers, calls Shutdown, then runs the trading-card manager base destructor. |
| `00ad2360` | `ClientSimpleCardGameManager::ScalarDeletingDtor` | ✅ | dtor | Runs the manager destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
| `00ad2380` | `AdvanceTcgStateMachineAndRecords` | ✅ | function | Advances states 1 through 5, triggers asset/UI callbacks and elapsed-time checks, sends leave/sub-opcode requests through shared helpers, then walks and removes expired tree entries. |
| `00ad2d30` | `ClientSimpleCardGameManager::InitCardClient` | ✅ | method | Clears a ready flag, announces UI unlock, creates the TCG interface and Flash view, loads TradingCardMain/TCCW_Main.swf, logs realm and tickets, shows the TCG handler, and hides HUD/dock/chat. |
| `00ad3150` | `StartTcgClientInitialization` | ✅ | virtual | Sets state +0x2ba90 to 3, stores the current 64-bit clock pair, and calls the client initialization routine. |
