# Class: Chat::ChatSendTextToLobbyDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01281b30` | `Chat::ChatSendTextToLobbyDisplayAction::ChatSendTextToLobbyDisplayAction` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Chat::ChatSendTextToLobbyDisplayAction::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea300. |
| `01281bc0` | `Chat::ChatSendTextToLobbyDisplayAction::GetLogTag` | ✅ | virtual | Returns a pointer to the literal kSendTextToLobbyDisplayAction : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:30 (mov eax, imm32; ret). |
| `01281bd0` | `Chat::ChatSendTextToLobbyDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |
| `01281de0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. Reads entry 2 as the ChatDisposition integer and entry 3 as the ConversationID integer (type checks assert and fall back to 0), entry 1 as a string (ChatLogText), entry 4 as an integer used with FUN_01… |
