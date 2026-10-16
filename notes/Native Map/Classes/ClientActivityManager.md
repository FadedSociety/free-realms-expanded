# Class: ClientActivityManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be6700` | `ClientActivityManager::ClearLists` | ✅ | virtual | Calls the no-argument thunks 00ce9190 and 00ce91a0 (each loads this+0x428 into ECX and tail-jumps to 00cfbe20), then pops and frees every node of the list at this+0x16868 via 00be52a0 until empty, then every node of the… |
| `00be6bb0` | `— ⚠reject` | ⚪ | dtor | Sets an EH frame and stores the ClientActivityManager vtable at *this; calls 006fc440 and, when it returns nonzero, 006fd410('ActivityUserData.ini', 0); drains the list at this+0xc via 00be14e0; calls 00ce9aa0, 00be6700… |
| `00be6d30` | `ClientActivityManager::LoadActivityFiles` | ✅ | virtual | Calls 00725ee0 on its first argument, appends '/' (006fc030 with 0x2f) when the last character of the path buffer local_120 is not '\' or '/', then calls the three activity file loaders in order: 00be58f0 (ActivityCateg… |
| `00be70b0` | `ClientActivityManager::ClientActivityManager` | ✅ | ctor | Stores BaseDefinitionManager then ClientActivityManager vtables at *this; constructs the member hash lists through 00be5e60, 00be5ec0, 00be6e10 and 00be5f20; calls 00cea090, 00ae4430 (HashMap init), 006ffa90 and 00be21a… |
| `00be71e0` | `ClientActivityManager_ScalarDtor` | 🟢 | dtor | Calls 00be6bb0 (ClientActivityManager destructor body) on this, then calls 00401c00(this, 0) (FreeMemory) when bit 0 of the flag argument is set; returns this. |
