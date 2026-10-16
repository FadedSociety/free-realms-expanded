# Class: Chat::ChatDisplayUChatTextDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01281730` | `Chat::ChatDisplayUChatTextDisplayAction::ChatDisplayUChatTextDisplayAction` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Chat::ChatDisplayUChatTextDisplayAction::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea300. |
| `012817c0` | `Chat::ChatDisplayUChatTextDisplayAction::GetLogTag` | ✅ | virtual | Returns a pointer to the literal kDisplayUChatTextDisplayAction : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:87 (mov eax, imm32; ret). |
| `012817d0` | `Chat::ChatDisplayUChatTextDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |
