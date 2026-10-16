# Class: Engine::DeckOpenDeckBuilder

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01272f70` | `Engine::DeckOpenDeckBuilder::ctor` | ✅ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckOpenDeckBuilder::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01272fe0` | `Engine::DeckOpenDeckBuilder::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string OpenDeckBuilder. |
| `01273000` | `Engine::DeckOpenDeckBuilder::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string OpenDeckBuilder : 0 : ..\common\...\deckhandlers.cc:163. |
| `01273010` | `Engine::DeckOpenDeckBuilder::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01274e30` | `Engine::DeckOpenDeckBuilder::Invoke ⚠downgrade` | 🟢 | virtual | Slot [2] of Engine::DeckOpenDeckBuilder::vftable, 608 bytes. Gets the thread-owned object via FUN_0126dad0 and calls its slot 0x0c; forwards the named fields at DAT_01811404 and DAT_017ea850 and the PlayFormat and Flags… |
