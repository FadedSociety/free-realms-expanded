# Class: Engine::DeckDeckSelected

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012741e0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckDeckSelected::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01274250` | `Engine::DeckDeckSelected::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeckSelected. |
| `01274270` | `Engine::DeckDeckSelected::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeckSelected : 0 : ..\common\...\deckhandlers.cc:509. |
| `01274280` | `Engine::DeckDeckSelected::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01276190` | `Engine::DeckDeckSelected::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckDeckSelected::vftable, 230 bytes. Gets the thread-owned object via FUN_0126dad0; reads the named field DAT_01811404 as a string (FUN_012eb250) and passes it to virtual slot 0x60 of the object. |
