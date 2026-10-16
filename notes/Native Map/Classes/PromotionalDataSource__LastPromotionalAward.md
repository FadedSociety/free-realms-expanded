# Class: PromotionalDataSource::LastPromotionalAward

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c69c60` | `PromotionalDataSource::LastPromotionalAward::FieldName` | ✅ | virtual | Virtual method at slot 1: switches on the field index and returns 0 ItemId, 1 Type, 2 Name, 3 Description, 4 ImageSetId, 5 TintValue, otherwise the shared <error> literal. |
| `00c6a180` | `PromotionalDataSource::LastPromotionalAward::GetFieldValue` | ✅ | virtual | Virtual method at slot 4, the award field reader: requires this+0x228 plus a table reached through this+0x224's +0x15c, whose +0xa8 dword is the row count and +0xa4 the row array with a 0x50-byte stride. For a valid ind… |
| `00c6a4b0` | `PromotionalDataSource::LastPromotionalAward::~LastPromotionalAward` | ✅ | dtor | Destructor body: rewrites the three LastPromotionalAward vtables at +0, +0x30 and +0x34 and then runs the GUI data-table base destructor FUN_0091bf60 under an SEH frame. |
| `00c6a520` | `PromotionalDataSource::LastPromotionalAward::GetCount` | ✅ | virtual | Virtual method at slot 2: returns the dword at this+0x228, the current number of award rows the data source exposes. |
| `00c6a5e0` | `PromotionalDataSource::LastPromotionalAward::LastPromotionalAward` | ✅ | ctor | Constructor taking a name pointer and an owner: runs the GUI data-table base constructor FUN_00922890(param_2), stores the three LastPromotionalAward vtables at +0, +0x30 and +0x34 and puts param_3 at +0x254 with a zero… |
| `00c6a660` | `PromotionalDataSource::LastPromotionalAward::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c6a4b0 and calls _free on this when bit 0 of the flag is set. |
