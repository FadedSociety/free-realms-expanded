# Class: OverHeadNameIconElement

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4c330` | `OverHeadNameIconElement::OverHeadNameIconElement` | ✅ | ctor | Constructor: copies a 4-dword parameter block from the static data at 01cb53d0 and calls FUN_00a4c260 (OverHeadBitmapElement constructor chain) forwarding the caller's parameters, then stores the OverHeadNameIconElement… |
| `00a4c3e0` | `OverHeadNameIconElement::~OverHeadNameIconElement` | 🟢 | dtor | Non-deleting destructor body: stores the OverHeadNameIconElement vtable back into the object and calls FUN_00a4b0a0, which resets the vtable to OverHeadBitmapElement's and then OverHeadElement's while releasing the embe… |
| `00a4c430` | `OverHeadNameIconElement::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor occupying vtable slot 0: restores the OverHeadNameIconElement vtable, runs the base-subobject cleanup FUN_00a4b0a0, and frees the object through the _free thunk when the deletion flag (pa… |
