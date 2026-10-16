# Class: Engine::DeckCloseDeckBuilder

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012730c0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckCloseDeckBuilder::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273130` | `Engine::DeckCloseDeckBuilder::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string CloseDeckBuilder. |
| `01273150` | `Engine::DeckCloseDeckBuilder::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string CloseDeckBuilder : 0 : ..\common\...\deckhandlers.cc:199. |
| `01273160` | `Engine::DeckCloseDeckBuilder::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01274530` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::DeckCloseDeckBuilder::vftable, 33 bytes (decoded: call FUN_012e0ba0; call FUN_0126dad0; if the result is non-null call its vtable slot at +0x10; mov eax,1; ret 8). Returns 1 with two stack argument b… |
