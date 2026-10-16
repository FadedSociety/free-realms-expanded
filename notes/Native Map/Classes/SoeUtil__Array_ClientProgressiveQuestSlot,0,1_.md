# Class: SoeUtil::Array<ClientProgressiveQuestSlot,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdd040` | `— ⚠reject` | ⚪ | virtual | Virtual buffer reallocation (vtable slot 1): with a non-forced request it grows the requested count by 5/4 when the current capacity at +0xc is smaller, or shrinks it by 6/5 when the request is below three quarters of t… |
| `00bdd0e0` | `ArrayProgressiveQuestSlot::FreeBuffer` | 🟢 | virtual | Virtual buffer free (vtable slot 2): routes the pointer argument to FUN_00401c00 (FreeMemory) with alignment 4. |
| `00bdd100` | `ArrayProgressiveQuestSlot::SwapBuffers` | 🟢 | virtual | Virtual slot 3 of the ClientProgressiveQuestSlot array: when the third argument is non-zero it delegates to virtual slot 3 of the other array; otherwise it exchanges the buffer pointer, size and capacity fields at +4, +… |
| `00bdd3a0` | `— ⚠reject` | ⚪ | function | Calls virtual slot 1 of the array (the grow function) with the requested count and the force flag; when the returned buffer differs from the current one at +4 it copies the existing size (+8) elements of nine dwords eac… |
| `00bdd760` | `ArrayProgressiveQuestSlot::Resize` | 🟢 | function | When the capacity at +0xc is smaller than the requested count it calls the grow-and-move helper 00bdd3a0 with force=1, then stores the requested count as the size at +8 (the element-construction loop reduces to nothing … |
| `00bdd860` | `ArrayProgressiveQuestSlot::~ArrayProgressiveQuestSlot` | 🟢 | dtor | Destructor body: stores the array vtable at +0, clears the size at +8 and frees the buffer at +4 with alignment 4 through FUN_00401c00, then clears the buffer pointer. |
| `00bdd8c0` | `ArrayProgressiveQuestSlot::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0): stores the array vtable, clears the size at +8, frees the buffer at +4 with alignment 4 through FUN_00401c00 and clears the buffer pointer, then frees the object through the … |
