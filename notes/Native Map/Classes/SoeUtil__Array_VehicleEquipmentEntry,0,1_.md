# Class: SoeUtil::Array<VehicleEquipmentEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc0a0` | `AllocateStorage` | ✅ | virtual | Uses the existing capacity at +0x0c in its growth/reuse decision, writes the selected capacity through param_3, returns the existing pointer at +4 when reused, or allocates param_2 entries at 0x18 bytes each with four-b… |
| `008bc140` | `FreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `008bc160` | `SwapOrDestroyStorage` | ✅ | virtual | If the destroy flag is set, calls param_2's virtual +0x0c; otherwise swaps storage pointer, capacity and size fields between the arrays. |
