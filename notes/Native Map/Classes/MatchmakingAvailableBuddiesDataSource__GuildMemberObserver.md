# Class: MatchmakingAvailableBuddiesDataSource::GuildMemberObserver

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7f580` | `MatchmakingAvailableBuddiesDataSource::GuildMemberObserver::~GuildMemberObserver` | 🟢 | dtor | Destructor body: rewrites the guild-member observer's vtable pointer, clears its name registration with FUN_008356b0(&DAT_0175b400) and runs the base destructor FUN_00835610. |
| `00a7f5e0` | `MatchmakingAvailableBuddiesDataSource::GuildMemberObserver::NotifyGuildMembersC…` | ✅ | virtual | Same notification as 00a7f560 for the guild observer: takes the owner at this+0x1c, clears the owner's data-changed byte at +0x258 and tail-jumps to the owner's virtual slot 4 (vtable offset 0x10). |
| `00a7fe80` | `MatchmakingAvailableBuddiesDataSource::GuildMemberObserver::ScalarDeletingDtor` | ✅ | dtor | Deleting destructor of the guild-member observer: calls the observer destructor body 00a7f580 and frees the object with _free when bit 0 of param_2 is set. |
| `00a809a0` | `MatchmakingAvailableBuddiesDataSource::GuildMemberObserver::GuildMemberObserver` | ✅ | ctor | Constructor: runs the base construction FUN_0081b470, installs the GuildMemberObserver vtable, stores the owner argument at +0x1c and registers the name BaseClient.GuildMembers through FUN_008356b0. |
