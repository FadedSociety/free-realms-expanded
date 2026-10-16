# Class: ItemSearchEngine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2cfa0` | `ItemSearchEngine::ItemSearchEngine` | ✅ | ctor | Constructor: calls the base constructor 00c30040, then installs the ItemSearchEngine vftable at this+0 and returns this. |
| `00c2d000` | `ItemSearchEngine::~ItemSearchEngine` | 🟢 | dtor | Destructor body: resets the ItemSearchEngine vftable and calls 00c2fa70 (the engine's member teardown), then returns. The export additionally lists it as a callee of EH unwind funclets (Unwind@0159b8c6, Unwind@016021ab)… |
| `00c2d060` | `ItemSearchEngine::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0] of 0x01830478): resets the vtable, calls the member teardown 00c2fa70 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2d100` | `ItemSearchEngine::GetEntryValue` | 🟢 | virtual | Virtual method (vtable slot [2]) with signature (id, out): performs the same item-table hash lookup as 00c2d0c0 and, when the entry exists, writes *(entry+0x28) through the out pointer and returns true (1), otherwise fa… |
| `00c2d150` | `ItemSearchEngine::NotifyEntry` | 🟢 | virtual | Virtual method (vtable slot [3]) with signature (id, value): performs the item-table hash lookup and, when the entry exists, calls virtual slot [2] (offset +8) of the interface at *(client+0x10) with (*(entry+8), value)… |
| `00c2d780` | `ItemSearchEngine::IndexItem ? ⚠downgrade` | 🟡 | virtual | Virtual method (vtable slot [1]) with signature (itemId, kind): via the client singleton (00769710) it requires the item manager at *(client+4)+0x3e0, resolves the item definition with 00c2d0c0(itemId), then through the… |
