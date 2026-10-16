# Class: InGamePurchaseDataSource::StoreBundleGroups

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997af0` | `StoreBundleGroupsFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreBundleGroups, one stack arg (index), no calls. Switch: 0 GroupId, 1 Name, 2 ImageSetId, 3 TintValue; any other index returns "<error>". |
| `0099f480` | `StoreBundleGroups::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x254 and jumps directly to 0099D440; it contains no separate row-population body. |
| `009a3940` | `StoreBundleGroups::GetValue` | ✅ | virtual | Looks up the selected bundle-group record, switches on field selector 0 through 3, and writes the selected value or conversion to the output. |
| `009a6c30` | `StoreBundleGroupsDestructor` | ✅ | dtor | Restores StoreBundleGroups vtables in its embedded subobjects and calls the GuiUserDataTable destructor for the owned table. |
| `009a6cc0` | `StoreBundleGroups::GetField22C` | ✅ | virtual | Returns the dword stored at this plus 0x22C. |
| `009a6ce0` | `StoreBundleGroupsDeletingDtorAdjustor` | ✅ | dtor | Subtracts 0x254 from this and jumps to 009a7890, the StoreBundleGroups scalar deleting destructor. |
| `009a77f0` | `InGamePurchaseDataSource::StoreBundleGroups::StoreBundleGroups` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner/context word param_3 at [0x96], installs StoreBundleGroups::vftable at [0], [0xc], [0xd], [0x95] and zer… |
