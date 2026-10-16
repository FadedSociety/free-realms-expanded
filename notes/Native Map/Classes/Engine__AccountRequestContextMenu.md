# Class: Engine::AccountRequestContextMenu

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128f390` | `Engine::AccountRequestContextMenu::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRequestContextMenu::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128f400` | `Engine::AccountRequestContextMenu::GetActionName` | ✅ | virtual | Returns the address of the constant string RequestContextMenu (6 bytes, mov eax,imm32 then ret). |
| `0128f420` | `Engine::AccountRequestContextMenu::GetSourceTag` | ✅ | virtual | Returns the address of the constant string RequestContextMenu : 0 : (accounthandlers.cc line 1168); 6 bytes, mov eax,imm32 then ret. |
| `0128f430` | `Engine::AccountRequestContextMenu::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
