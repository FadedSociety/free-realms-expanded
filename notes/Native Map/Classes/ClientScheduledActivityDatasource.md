# Class: ClientScheduledActivityDatasource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfb1a0` | `ClientScheduledActivityDatasource::GetColumnName` | ✅ | virtual | Returns scheduled activity time fields followed by the base activity identity, label, category, image, membership, and filename fields. |
| `00cfbda0` | `ClientScheduledActivityDatasource::RefreshRows ?` | 🟡 | virtual | Destroys embedded table state, reorders records with FUN_00cfb6c0, then emits each ordered record through a datasource callback. |
| `00cfd270` | `ClientScheduledActivityDatasource::~ClientScheduledActivityDatasource` | ✅ | dtor | Restores the scheduled datasource vtables, destroys its embedded table holder, removes remaining scheduled-row nodes, destroys the map, and releases the holder object. |
| `00cfd590` | `ClientScheduledActivityDatasource::ClientScheduledActivityDatasource` | ✅ | ctor | Initializes the shared GUI-data-source base, installs scheduled-activity datasource vtables at the object and two embedded offsets, then initializes its 512-bucket map. |
| `00cfd610` | `ClientScheduledActivityDatasource::ScalarDeletingDtor` | ✅ | dtor | Calls the scheduled-activity datasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |
