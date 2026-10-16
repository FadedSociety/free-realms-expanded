# Class: Chat::ChatGetConversationID

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01281c30` | `Chat::ChatGetConversationID::ChatGetConversationID` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Chat::ChatGetConversationID::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `01281ca0` | `Chat::ChatGetConversationID::GetName` | ✅ | virtual | Returns the literal string GetConversationID (mov eax, imm32; ret). |
| `01281cc0` | `Chat::ChatGetConversationID::GetLogTag` | ✅ | virtual | Returns a pointer to the literal GetConversationID : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:128 (mov eax, imm32; ret). |
| `01281cd0` | `Chat::ChatGetConversationID::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |
