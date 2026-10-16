# Class: ClientPointOfInterestManager::InternalDs

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b97c40` | `ClientPointOfInterestManager_InternalDs_GetColumnName` | ✅ | virtual | Column-name table of the point-of-interest data source: a switch over the column index returns the literal names Id, Name, Zone ID, X, Y, Z, Heading, IconId, NotificationType, SubName, BreadcrumbQuestId and AtlasName, a… |
| `00b97cf0` | `InternalDs_Thunk_slot2_GetRowCount` | 🟢 | virtual | Adjustor thunk of 8 bytes: adds 4 to this and tail-jumps to FUN_010c8f30, the generic data source implementation this interface slot forwards to. |
| `00b97d00` | `ClientPointOfInterestManager_InternalDs_RefreshAllRows` | 🟢 | virtual | Calls the virtual at slot 3 of the helper object at this+0x34, then iterates the whole point-of-interest table from FUN_0102fb90 through the next-row accessor FUN_0102fd90, invoking the virtual at slot 1 of the same hel… |
| `00b987b0` | `— ⚠reject` | ⚪ | virtual | Column value accessor: rejects column indices at or above the row count or missing rows, looks the point-of-interest row up through FUN_0102fb50 by the id at row+0, and then switches on the column: 1 and 9 resolve a loc… |
| `00b98ce0` | `ClientPointOfInterestManager_InternalDs_Destruct` | ✅ | dtor | Destructor body of the point-of-interest data source: stores the InternalDs vtable into the three interface slots at +0, +0x30 and +0x34 and calls the base GuiUserDataTable destructor FUN_0091bf60 inside an SEH frame. |
| `00b98d70` | `ClientPointOfInterestManager_InternalDs_Construct` | ✅ | ctor | Constructor of the point-of-interest data source: runs the base IDataAccessTable/GuiUserDataTable constructor FUN_00922890 with param_2, stores the InternalDs vtable into the three interface slots at +0, +0x30 and +0x34… |
| `00b98df0` | `ClientPointOfInterestManager_InternalDs_DeletingDestruct` | ✅ | dtor | Scalar deleting destructor: calls the data source destructor 00b98ce0 and then _free(this) when bit 0 of param_2 is set. |
