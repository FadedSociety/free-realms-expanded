# Class: InventoryNavigationDataSource::CategoryNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7b2f0` | `— ⚠reject` | ⚪ | ctor | Constructor (138 bytes): stores the three arguments at +0x10, +0x14 and +0x18, sets +4 to 1, +8 and +0xc to -1, installs the CategoryNode vtable at [this], installs the SoeUtil::Array<int,0,1> vtable at +0x1c, zeroes th… |
| `00a7b380` | `InventoryNavigationDataSource::CategoryNode::Slot0_ReturnThis` | ✅ | virtual | Three bytes (8b c1 c3): moves ECX to EAX and returns, so it returns this unchanged. Slot 0 of this vtable is not a destructor here. |
| `00a7c220` | `CategoryNode::GetColumnValue` | ✅ | virtual | Column value getter of a category row node. Resolves the category definition through FUN_01045060(this+0x10). Column 3 writes the definition field +4 through the requester's sink ((**(param_2+4))+4 vtable slot 1), colum… |
