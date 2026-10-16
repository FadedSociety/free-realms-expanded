# Class: SoeUtil::Array<AbilityEffectTooltipEntry,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a74060` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. Takes the pointer argument; computes (this+0x13) rounded down to a multiple of 4 and skips the free when the pointer equals it (the inline buffer). Otherwise calls FreeMemory(pointer, 0) (push 0, push … |
| `00a74290` | `AbilityEffectTooltipEntryArray10::DtorBody` | ✅ | dtor | Stores the Array<AbilityEffectTooltipEntry,10,1> vtable at this, calls 00a740e0 with the count at +8 (truncate all elements), then frees the buffer at +4 through 00401c00 (alignment 4) only when it is not the inline buf… |
| `00a74360` | `AbilityEffectTooltipEntryArray10::ReserveBuffer` | ✅ | virtual | Virtual slot [1]. Takes the requested count. When it is at most 10 it stores 10 at the count out-pointer (second stack argument) and returns the inline buffer address ((this+0x13) rounded down to a multiple of 4). Other… |
| `00a746c0` | `AbilityEffectTooltipEntryArray10::Ctor` | ✅ | ctor | Installs the Array<AbilityEffectTooltipEntry,0,1> vtable first, zeroes the buffer (+4), count (+8) and capacity (+0xc), then installs the Array<AbilityEffectTooltipEntry,10,1> vtable as the final vptr. Returns nothing. |
| `00a74720` | `AbilityEffectTooltipEntryArray10::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a74290 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a75130` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] of Array<AbilityEffectTooltipEntry,10,1>. Takes a source array argument (count at +8). Replaces this array's contents: truncates through 00a740e0, appends the source range through 00a74a40, and calls 00… |
