# Class: UI::UIServiceBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01239a70` | `UIServiceBase_ResetContainers ?` | 🟡 | virtual | Virtual slot [1] of the UI service vtables: compacts the pointer vector at +0x40/+0x44 with _memmove_s when begin and end differ, erases the map at +0x50 through FUN_012394f0, resets its header and count at +0x54, then … |
| `0123a140` | `UI::UIServiceBase::DtorBody` | ✅ | dtor | Stores UI::UIServiceBase::vftable into the object, tears down the tree through FUN_0123a0d0, frees the buffer at +0x40 when non-null and zeroes +0x40..+0x48, calls FUN_01238ae0, releases the string buffer at +0x10 when … |
| `0123a270` | `UI::UIServiceBase::Ctor` | ✅ | ctor | Calls FUN_01230d10 (base constructor stub), stores UI::UIServiceBase::vftable, initialises the string at +0x10..+0x24 to empty with capacity 15, zeroes the vector and map fields at +0x2c..+0x48 and the flag at +0x60, th… |
| `0123a320` | `UI::UIServiceBase::ScalarDeletingDtor` | ✅ | dtor | Calls the destructor body FUN_0123a140 and, when bit 0 of the flag argument is set, frees the object with _free; returns the object pointer. |
