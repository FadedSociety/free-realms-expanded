# Class: ClientActivityCategoryDatasource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfb370` | `ClientActivityCategoryDatasource::GetColumnName` | ✅ | virtual | Returns Id, NameId, DescriptionId, ThumbnailImageFilename, or MysteryChestIcon for the corresponding column index. |
| `00cfd340` | `ClientActivityCategoryDatasource::~ClientActivityCategoryDatasource` | ✅ | dtor | Restores category datasource vtables, destroys its embedded table holder, unlinks its row nodes, destroys both integer maps, and releases the holder object. |
| `00cfd630` | `ClientActivityCategoryDatasource::ClientActivityCategoryDatasource` | ✅ | ctor | Initializes the base with the Activities.Ref.Categories source, installs category datasource vtables, then initializes two embedded integer-keyed maps. |
| `00cfd6c0` | `ClientActivityCategoryDatasource::ScalarDeletingDtor` | ✅ | dtor | Calls the category datasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |
