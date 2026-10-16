# Class: Engine::LobbyJoinMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d9e0` | `Engine::LobbyJoinMatch::LobbyJoinMatch` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyJoinMatch::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127da50` | `Engine::LobbyJoinMatch::GetName` | ✅ | virtual | Returns the literal string JoinMatch (mov eax, imm32; ret). |
| `0127da70` | `Engine::LobbyJoinMatch::GetLogTag` | ✅ | virtual | Returns a pointer to the literal JoinMatch : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:505 (mov eax, imm32; ret). |
| `0127da80` | `Engine::LobbyJoinMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280690` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: gets the key at DAT_018c9f18 (FUN_012eb290), the Password text and the DeckID text (FUN_012eb250, default DAT_0175b400), gets GroupID (FUN_012eb290), then calls the mana… |
