# Class: Engine::LobbyQueryFriendsList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127df20` | `Engine::LobbyQueryFriendsList::LobbyQueryFriendsList` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryFriendsList::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127df90` | `Engine::LobbyQueryFriendsList::GetName` | ✅ | virtual | Returns the literal string QueryFriendsList (mov eax, imm32; ret). |
| `0127dfb0` | `Engine::LobbyQueryFriendsList::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryFriendsList : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:582 (mov eax, imm32; ret). |
| `0127dfc0` | `Engine::LobbyQueryFriendsList::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280b00` | `Engine::LobbyQueryFriendsList::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127ebb0() (LookupFieldIfOwnerThread) is nonzero: gets AccountID (FUN_012eb290), builds the tag OpenDialog with FUN_012ec220, sets AccountID, sets DAT_0188ce5c to DAT_0175b400, ScreenName to F… |
