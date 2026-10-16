# Class: Chat::DisplayEasyChatTextDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012818d0` | `Chat::DisplayEasyChatTextDisplayAction::DisplayEasyChatTextDisplayAction` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Chat::DisplayEasyChatTextDisplayAction::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea300. |
| `01281960` | `Chat::DisplayEasyChatTextDisplayAction::GetLogTag` | ✅ | virtual | Returns a pointer to the literal kDisplayEasyChatTextDisplayAction : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:140 (mov eax, imm32; ret). |
| `01281970` | `Chat::DisplayEasyChatTextDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |
