# Class: PromotionalBundleDatasource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c69cd0` | `PromotionalBundleDatasource::FieldName` | ✅ | virtual | Virtual method at slot 1: switches on the field index and returns Name (0 and 8), Description (1), ImageSetId (2), TintValue (3), ItemListDetails (4), BundleId (5), ItemId (6), Quantity (7). |
| `00c6a2d0` | `PromotionalBundleDatasource::GetFieldValue ⚠downgrade` | 🟢 | virtual | Virtual method at slot 4, the bundle field reader: resolves the bundle id through virtual slot +0x10 of the object at this+4, then looks the entry up in the bundle table with FUN_00c69e10 and answers field param_2 - 0 a… |
| `00c6a540` | `PromotionalBundleDatasource::~PromotionalBundleDatasource` | ✅ | dtor | Destructor body: rewrites the three PromotionalBundleDatasource vtables at +0, +0x30 and +0x34, destroys the embedded SoeUtil::HashListMap through FUN_009c8580, then runs the base destructor FUN_0091bf60 with the stack … |
| `00c6a5d0` | `PromotionalBundleDatasource::GetCount` | ✅ | virtual | Virtual method at slot 2: returns the dword at this+0x238, the number of bundle rows the data source exposes. |
| `00c6a680` | `PromotionalBundleDatasource::PromotionalBundleDatasource` | ✅ | ctor | Constructor taking a name pointer and an owner: runs FUN_00922890(param_2) for the GUI data-table base, stores the owner at +0x254, installs the three PromotionalBundleDatasource vtables and constructs the bundle hash m… |
| `00c6a700` | `PromotionalBundleDatasource::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c6a540 and calls _free on this when bit 0 of the flag is set. |
