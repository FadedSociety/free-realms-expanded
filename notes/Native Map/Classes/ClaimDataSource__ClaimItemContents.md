# Class: ClaimDataSource::ClaimItemContents

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c691c0` | `ClaimDataSource::ClaimItemContents::FieldName` | ✅ | virtual | Virtual method at slot 1 of the class vtable: switches on the field index and returns the column name - 0 ItemId, 1 ItemName, 2 ItemQuantity, 3 ItemImageSetId, 4 ItemTint. |
| `00c696c0` | `— ⚠reject` | ⚪ | virtual | Virtual method at slot 4 of the item-contents data source: requires the table at this+0x228 plus the count at this+0x2a8, then for row param_2 resolves the item record through FUN_0095d070(row entry dword at +8 of the 0… |
| `00c698a0` | `— ⚠reject` | ⚪ | dtor | Destructor body: rewrites the four ClaimItemContents vtables at +0, +0x30, +0x34 and +0x254, destroys the embedded ClaimItem at +0x254 through FUN_01060db0, then restores the InternalDsBase vtable and runs the base dest… |
| `00c69960` | `ClaimDataSource::ClaimItemContents::ScalarDeletingDtorThunk` | ✅ | virtual | Eleven-byte adjustor thunk at slot 0 of the secondary vtable: sub ecx,0x254 then jmp 00c69ad0, i.e. it re-bases this onto the complete item-contents object and jumps into that class's scalar deleting destructor. |
| `00c69970` | `ClaimDataSource::ClaimItemContents::GetCount` | ✅ | virtual | Virtual method at slot 2: returns the dword count at this+0x2a8, the number of claim-item entries in the contents array. |
| `00c69a30` | `ClaimDataSource::ClaimItemContents::ClaimItemContents` | ✅ | ctor | Constructor taking a name pointer and an owner: runs the GUI data-table base constructor FUN_00922890(param_2), stores the InternalDsBase vtable at +0x254 and the owner at +0x258, installs the four ClaimItemContents vta… |
| `00c69ad0` | `ClaimDataSource::ClaimItemContents::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c698a0 and calls _free on this when bit 0 of the flag is set. |
