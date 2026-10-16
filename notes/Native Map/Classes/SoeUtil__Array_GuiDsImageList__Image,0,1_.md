# Class: SoeUtil::Array<GuiDsImageList::Image,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c132d0` | `ImageArrayResize` | ✅ | virtual | Virtual slot 1 (reallocate) of the embedded image array: computes a new capacity from the requested count param_2 and the current capacity at +0xc (grow by 5/4 when growing, shrink by 6/5 only when the request is below … |
| `00c13370` | `ImageArrayFreeBuffer` | ✅ | virtual | Virtual slot 2 (release) of the embedded image array: calls FUN_00401c00(param_1, 4), i.e. frees the element buffer with alignment 4 through the memory manager. |
| `00c13390` | `ImageArrayMoveAssign` | ✅ | virtual | Virtual slot 3 (move/swap) of the embedded image array: when param_3 is non-zero it forwards to the source object's virtual slot +0xc; otherwise it swaps the three fields of *this with the source (element buffer at +4, … |
| `00c13400` | `ImageArrayPopBack` | 🟢 | function | Removes param_2 elements from the end of the image array: computes the new size = size - param_2, walks the elements from the new size to the old size and calls each non-null element's virtual slot +0x10 with argument 0… |
| `00c13490` | `ImageListArrayDtor` | 🟢 | dtor | Destructor body of the embedded image array: stores the array vtable, releases every element by calling the pop-back routine FUN_00c13400 with the current size, frees the element buffer through FUN_00401c00(ptr, 4) and … |
| `00c134c0` | `ImageArrayScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the image array): stores the array vtable, releases all elements with FUN_00c13400, frees the element buffer with FUN_00401c00(ptr,4), clears the pointer and frees the object… |
