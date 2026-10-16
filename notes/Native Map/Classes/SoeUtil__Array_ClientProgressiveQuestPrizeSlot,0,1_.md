# Class: SoeUtil::Array<ClientProgressiveQuestPrizeSlot,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdd170` | `ArrayProgressiveQuestPrizeSlot::Grow` | 🟢 | virtual | Virtual buffer reallocation (vtable slot 1) for the ClientProgressiveQuestPrizeSlot array: same growth policy as 00bdd040 (5/4 growth, 6/5 shrink) but with 0x1c-byte elements, allocating count*0x1c bytes with alignment … |
| `00bdd210` | `ArrayProgressiveQuestPrizeSlot::FreeBuffer` | 🟢 | virtual | Virtual buffer free (vtable slot 2): routes the pointer argument to FUN_00401c00 (FreeMemory) with alignment 4. |
| `00bdd230` | `ArrayProgressiveQuestPrizeSlot::SwapBuffers` | 🟢 | virtual | Virtual slot 3 of the ClientProgressiveQuestPrizeSlot array: when the third argument is non-zero it delegates to virtual slot 3 of the other array; otherwise it exchanges the buffer pointer, size and capacity fields at … |
| `00bdd440` | `— ⚠reject` | ⚪ | function | Calls virtual slot 1 of the array (the grow function) with the requested count and the force flag; when the returned buffer differs from the current one at +4 it copies the existing size (+8) elements of seven dwords ea… |
| `00bdd7e0` | `ArrayProgressiveQuestPrizeSlot::Resize` | 🟢 | function | When the capacity at +0xc is smaller than the requested count it calls the grow-and-move helper 00bdd440 with force=1, then stores the requested count as the size at +8. |
| `00bdd890` | `ArrayProgressiveQuestPrizeSlot::~ArrayProgressiveQuestPrizeSlot` | 🟢 | dtor | Destructor body: stores the array vtable at +0, clears the size at +8 and frees the buffer at +4 with alignment 4 through FUN_00401c00, then clears the buffer pointer. |
| `00bdd900` | `ArrayProgressiveQuestPrizeSlot::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0): stores the array vtable, clears the size at +8, frees the buffer at +4 with alignment 4 through FUN_00401c00 and clears the buffer pointer, then frees the object through the … |
