# Class: AbilityEffectsTooltipDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a73e00` | `AbilityEffectsTooltipDataSource::ColumnTitle` | ✅ | virtual | Virtual slot [1]. Switch on the int argument: 0 returns the literal Image ID, 1 returns the literal Description, any other value returns the literal <error>. Pure lookup; no state is read. |
| `00a73ee0` | `AbilityEffectsTooltipDataSource::CastCheckSetFlagNotify` | 🟢 | virtual | Virtual slot [6]. Calls 00d8d382 (a dynamic cast helper) on the pointer at this+0x18 with the GuiDataSource and AbilitiesDataSource type descriptors. When the result is non-null and its dword at +0x300 equals this+0x20,… |
| `00a74f60` | `AbilityEffectsTooltipDataSource::DtorBody` | ✅ | dtor | Stores the AbilityEffectsTooltipDataSource vtables at this, +0x30, +0x34 and +0x254, calls 00a74290 (the 10-slot entry array destructor body; receiver not shown in the pseudocode), calls 00835610 (unlabelled), then call… |
| `00a75000` | `AbilityEffectsTooltipDataSource::AdjustorToScalarDtor` | ✅ | virtual | Secondary-vtable slot [1] (018162c8). Subtracts 0x254 from ECX and tail-jumps to 00a752e0 (jmp rel32), i.e. an adjustor thunk for the scalar deleting destructor of the primary base. |
| `00a75010` | `AbilityEffectsTooltipDataSource::GetCellEntry` | 🟢 | virtual | Virtual slot [4] of the secondary vtable 01816304 (row, column, out). Returns 0 when row is negative or not below the count at +0x250. Otherwise it copies the 24-byte entry at buffer(+0x24c)+row*0x18 through 00a301f0, t… |
| `00a750e0` | `AbilityEffectsTooltipDataSource::GetAfterOptionalRefresh ? → AbilityEffectsTool…` | 🟡 | virtual | Virtual slot [2]. When the byte at this+0x240 is nonzero it calls 00a74af0 with ECX = this-0x30. Then returns the dword at this+0x250. |
| `00a75230` | `AbilityEffectsTooltipDataSource::Ctor` | ✅ | ctor | Calls the base constructor 00922890 with its own stack argument (the function also references the literal BaseClient.Abilities at 0183523c), installs the AbilityEffectsTooltipDataSource vtables, constructs the 10-slot e… |
| `00a752e0` | `AbilityEffectsTooltipDataSource::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a74f60 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
