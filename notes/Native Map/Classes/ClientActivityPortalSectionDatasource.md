# Class: ClientActivityPortalSectionDatasource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfb3d0` | `ClientActivityPortalSectionDatasource::GetColumnName` | ✅ | virtual | Returns Id, NameId, or DescriptionId for indices 0, 1, and 2; other indices return an error label. |
| `00cfd420` | `ClientActivityPortalSectionDatasource::~ClientActivityPortalSectionDatasource` | ✅ | dtor | Restores portal-section datasource vtables, destroys its embedded table holder, unlinks remaining rows, destroys the integer map, and releases the holder object. |
| `00cfd6e0` | `ClientActivityPortalSectionDatasource::ClientActivityPortalSectionDatasource` | ✅ | ctor | Initializes the base with Activities.Ref.PortalSections, installs portal-section datasource vtables, then initializes the embedded integer-keyed map. |
| `00cfd760` | `ClientActivityPortalSectionDatasource::ScalarDeletingDtor` | ✅ | dtor | Calls the portal-section datasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |
