# Class: ClientActivityDatasource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfb090` | `ClientActivityDatasource::GetColumnName` | ✅ | virtual | Returns field labels for activity ID, localized names, category, image, join and feature flags, membership, image filenames, difficulty, and chest icon. |
| `00cfc180` | `ClientActivityDatasource::GetCell` | ✅ | virtual | Resolves the activity row and underlying catalogue record, then dispatches by column to return activity flags, IDs, category, counts, filenames, and related metadata. |
| `00cfd1a0` | `ClientActivityDatasource::~ClientActivityDatasource` | ✅ | dtor | Restores the datasource vtables, destroys its embedded table holder, repeatedly unlinks remaining row nodes, destroys the integer map, and releases the holder object. |
| `00cfd4f0` | `ClientActivityDatasource::ClientActivityDatasource` | ✅ | ctor | Initializes the shared GUI-data-source base, installs ClientActivityDatasource vtables at the object and two embedded offsets, then initializes the integer-keyed map. |
| `00cfd570` | `ClientActivityDatasource::ScalarDeletingDtor` | ✅ | dtor | Calls the ClientActivityDatasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |
