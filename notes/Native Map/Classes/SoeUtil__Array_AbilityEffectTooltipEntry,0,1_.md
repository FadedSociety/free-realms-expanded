# Class: SoeUtil::Array<AbilityEffectTooltipEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a73f30` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1]. Same capacity policy shape as 00a71580: reads the capacity at this+0xc; with flag 0 it grows to 5/4 of a larger request or shrinks to 6/5 when capacity exceeds 4/3 of the request; with flag nonzero it … |
| `00a73fd0` | `AbilityEffectTooltipEntryArray::FreeBuffer` | ✅ | virtual | Virtual slot [2]. Reads its first stack argument and calls FreeMemory(pointer, 4) (push 4, push pointer), then returns with ret 8. |
| `00a73ff0` | `AbilityEffectTooltipEntryArray::SwapOrDelegate ?` | 🟡 | virtual | Virtual slot [3]. Same shape as 00a71640: when the byte argument is nonzero it calls vtable slot +0xc of the other object and returns; otherwise it exchanges buffer (+4) and XOR-swaps count (+8) and capacity (+0xc) with… |
| `00a740e0` | `AbilityEffectTooltipEntryArray::TruncateLast` | 🟢 | method | Takes one stack argument n. Destroys the last n elements (indices count-n up to count-1) by calling 00a2f220 on each element with ECX = element, then sets the count at this+8 to count-n. Element size not decoded here. |
| `00a74260` | `AbilityEffectTooltipEntryArray::DtorBody` | ✅ | dtor | Stores the Array<AbilityEffectTooltipEntry,0,1> vtable at this, calls 00a740e0 with the current count as its argument before freeing, frees the buffer at +4 through 00401c00 (alignment 4), and sets +4 to 0. Does not fre… |
| `00a74390` | `AbilityEffectTooltipEntryArray::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Stores the Array<AbilityEffectTooltipEntry,0,1> vtable, truncates all elements through 00a740e0, frees the buffer at +4 through 00401c00 (alignment 4), clears +4, then calls _free on this when bit 0 of… |
| `00a74a40` | `AbilityEffectTooltipEntryArray::AppendRange` | 🟢 | method | Takes a source buffer pointer and a count (stack arguments). Grows the array through 00a30280 when capacity at +0xc is short, adds the count to the count at +8, then for each new 24-byte slot (buffer+index*0x18) it copy… |
| `00a74f20` | `AbilityEffectTooltipEntryArray::AssignFrom` | 🟢 | method | Takes one stack argument, a source array object (buffer at +4, count at +8). Truncates this array to zero through 00a740e0 with this count at +8, then appends the source range through 00a74a40 (or appends an empty range… |
