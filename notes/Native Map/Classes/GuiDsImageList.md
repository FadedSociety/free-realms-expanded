# Class: GuiDsImageList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c13500` | `— ⚠reject` | ⚪ | ctor | Constructor: runs the GUI base constructor FUN_00822990 on param_2, stores GuiDsImageList::vftable at *this and initialises the embedded image array at +0x30 (array vtable at +0x30, buffer/size/capacity at +0x34, +0x38 … |
| `00c13570` | `GuiDsImageList::~GuiDsImageList` | 🟢 | dtor | Destructor body: resets the embedded array vtable at +0x30, releases its elements with FUN_00c13400, frees the element buffer with FUN_00401c00(ptr,4), clears the buffer pointer and calls the GUI base destructor FUN_008… |
| `00c137a0` | `GuiDsImageList::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuiDsImageList): calls the destructor body FUN_00c13570 (which empties and frees the embedded array and runs the GUI base destructor) and frees the object through _free when … |
