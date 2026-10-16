# Class: ClientLeaderBoardManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b85d50` | `ClientLeaderBoardManager::LoadLeaderBoardUrl ⚠downgrade` | 🟢 | method | Reads the WebResources/GameLeaderBoardUrl configuration string into a 512-byte buffer through FUN_006febe0 and assigns it via SoeUtil::IStringAssign, then creates the HTTP request/timer object (allocation of 0x93b8 byte… |
| `00b85e80` | `ClientLeaderBoardManager::RequestLeaderBoard` | ✅ | method | Builds the leaderboard request URL: with a non-null URL string it initializes an IString (FUN_006f9e60), appends the passed string (FUN_00402770), logs it as 'ClientLeaderBoardManager::RequestLeaderBoard, sending: %s' t… |
| `00b865b0` | `ClientLeaderBoardManager::ClientLeaderBoardManager` | ✅ | ctor | Constructor: installs TinyHttp::HttpManagerHandler::vftable then ClientLeaderBoardManager::vftable, stores param_2 at +4, zeroes +8/+0xc/+0x10, initializes the embedded response member (00b864d0), allocates a 0x25c-byte… |
| `00b86670` | `— ⚠reject` | ⚪ | dtor | Destructor body: reinstalls ClientLeaderBoardManager::vftable, frees the heap object at +8 (FUN_00d32f60 then _free), releases and clears the ref-counted data-source pointer at +0x34 (virtual slot 10 plus FUN_0081ad80),… |
| `00b86840` | `ClientLeaderBoardManager::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of ClientLeaderBoardManager: runs the destructor body 00b86670 and frees the manager object through the _free thunk when bit 0 of param_2 is set. |
| `00b86880` | `ClientLeaderBoardManager::OnGetResult` | ✅ | virtual | HTTP result callback (vtable slot 1): clears the embedded response fields at +0x14/+0x18/+0x1c and the entry array through 00b86200, and when a body buffer is supplied it appends a NUL to the collected bytes, assigns th… |
