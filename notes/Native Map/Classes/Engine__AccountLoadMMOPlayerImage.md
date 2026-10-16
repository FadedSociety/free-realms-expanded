# Class: Engine::AccountLoadMMOPlayerImage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128f4e0` | `— ⚠reject` | ⚪ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountLoadMMOPlayerImage::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128f550` | `Engine::AccountLoadMMOPlayerImage::GetActionName` | ✅ | virtual | Returns the address of the constant string LoadMMOPlayerImage (6 bytes, mov eax,imm32 then ret). |
| `0128f570` | `Engine::AccountLoadMMOPlayerImage::GetSourceTag` | ✅ | virtual | Returns the address of the constant string LoadMMOPlayerImage : 0 : (accounthandlers.cc line 1192); 6 bytes, mov eax,imm32 then ret. |
| `0128f580` | `Engine::AccountLoadMMOPlayerImage::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
