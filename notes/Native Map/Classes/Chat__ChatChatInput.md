# Class: Chat::ChatChatInput

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01281a20` | `Chat::ChatChatInput::ChatChatInput` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Chat::ChatChatInput::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `01281a90` | `Chat::ChatChatInput::GetName` | ✅ | virtual | Returns the literal string SendChat (mov eax, imm32; ret). |
| `01281ab0` | `Chat::ChatChatInput::GetLogTag` | ✅ | virtual | Returns a pointer to the literal SendChat : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:156 (mov eax, imm32; ret). |
| `01281ac0` | `Chat::ChatChatInput::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |
