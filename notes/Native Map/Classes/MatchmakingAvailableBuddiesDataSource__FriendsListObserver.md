# Class: MatchmakingAvailableBuddiesDataSource::FriendsListObserver

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7f500` | `MatchmakingAvailableBuddiesDataSource::FriendsListObserver::~FriendsListObserver` | 🟢 | dtor | Destructor body: rewrites the observer's vtable pointer, clears its name registration with FUN_008356b0(&DAT_0175b400) and runs the base destructor FUN_00835610. |
| `00a7f560` | `MatchmakingAvailableBuddiesDataSource::FriendsListObserver::NotifyBuddiesChanged` | ✅ | virtual | Reads the owner data source from this+0x1c, clears that object's data-changed byte at +0x258, and tail-jumps (ff e2 in the machine code) to the owner's virtual slot 4 (vtable offset 0x10) to refresh it; Ghidra renders t… |
| `00a7fe60` | `MatchmakingAvailableBuddiesDataSource::FriendsListObserver::ScalarDeletingDtor` | ✅ | dtor | Deleting destructor of the friend-list observer: calls the observer destructor body 00a7f500 and frees the object with _free when bit 0 of param_2 is set. |
| `00a80930` | `MatchmakingAvailableBuddiesDataSource::FriendsListObserver::FriendsListObserver` | ✅ | ctor | Constructor: runs the base construction FUN_0081b470, installs the FriendsListObserver vtable, stores the owner argument at +0x1c (the field the change notification 00a7f560 dereferences) and registers the name BaseClie… |
