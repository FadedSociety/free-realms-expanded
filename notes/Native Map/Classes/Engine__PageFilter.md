# Class: Engine::PageFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `013c1a50` | `Engine::PageFilter::PageFilter ⚠downgrade` | 🟢 | ctor | Constructor (fastcall): stores the Engine::PageFilter vtable at +0, zeroes the words at +4 and +8, sets the word at +0xc to 1 and returns this (bytes 8b c1 / 33 c9 / c7 00 ... / 89 48 04 / 89 48 08 / c7 40 0c 01 00 00 0… |
| `013c1a70` | `Engine::PageFilter::~PageFilter` | ✅ | dtor | Destructor body (fastcall): re-stores the Engine::PageFilter vtable at +0 and returns. Nothing is freed here. |
| `013c1bb0` | `Engine::PageFilter::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] of Engine::PageFilter (bytes f6 44 24 04 01 / 56 / 8b f1 / c7 06 ... / 74 09 / 56 / e8 ... / c3): stores the Engine::PageFilter vtable at +0, calls _free(this) only when bit 0 of the argument is set, an… |
| `013c2140` | `Engine::PageFilter::CopyPage` | 🟢 | virtual | Virtual slot [1] of Engine::PageFilter: validates the source vector at param_3 (end not below begin, FUN_00d83c2d on failure), builds an iterator with FUN_012c6910, and returns early when the destination vector param_2 … |
