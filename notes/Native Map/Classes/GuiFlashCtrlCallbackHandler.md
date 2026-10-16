# Class: GuiFlashCtrlCallbackHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083f2f0` | `CbHandlerSlot3ChunkedLoad ?` | 🟡 | virtual | Virtual slot 3 of GuiFlashCtrlCallbackHandler (366 bytes, SEH frame, 4 KB stack buffer). Calls 0083e960 on a stack-held argument; if it returns a non-null object with a non-zero first word, reads a file in 0x1000-byte c… |
| `0083f800` | `LookupStringOrNotFound` | 🟢 | virtual | Virtual slot 0 of GuiFlashCtrlCallbackHandler (283 bytes, SEH frame). Returns 0 when arg1 is null or the lookup object at FUN_00769710 has no table (+0x10 zero). Otherwise builds two SoeUtil::IString temporaries (IStrin… |
| `00840df0` | `CbHandlerSlot4GameSceneRect ?` | 🟡 | virtual | Virtual slot 4 of GuiFlashCtrlCallbackHandler (961 bytes, five float arguments and a count). Looks up the GuiFlashCtrl entry with 0083e960 and returns if it is absent. Gets the client rectangle of the 'GameScene' contro… |
| `008411c0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 13 of GuiFlashCtrlCallbackHandler, one C-string argument (522 bytes). Takes the larger of two lengths from 0083cfd0 and skips past it, then compares the rest of the name against 'images.txt' and 'ImageSetMa… |
