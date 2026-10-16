# System: ui.activity

21 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cfb090` | `ClientActivityDatasource::GetColumnName` | ClientActivityDatasource | Provides activity catalogue column labels to UI data binding. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfb1a0` | `ClientScheduledActivityDatasource::GetColumnName` | ClientScheduledActivityDatasource | Provides scheduled activity datasource column labels. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfb370` | `ClientActivityCategoryDatasource::GetColumnName` | ClientActivityCategoryDatasource | Provides activity category datasource column labels. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfb3d0` | `ClientActivityPortalSectionDatasource::GetColumnName` | ClientActivityPortalSectionDatasource | Provides activity portal section column labels. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfc180` | `ClientActivityDatasource::GetCell` | ClientActivityDatasource | Supplies cell data for the client activity datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfd1a0` | `ClientActivityDatasource::~ClientActivityDatasource` | ClientActivityDatasource | Releases the shared activity data-source state and owned rows. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd270` | `ClientScheduledActivityDatasource::~ClientScheduledActivityDatasource` | ClientScheduledActivityDatasource | Releases scheduled-activity datasource state and its owned rows. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd340` | `ClientActivityCategoryDatasource::~ClientActivityCategoryDatasource` | ClientActivityCategoryDatasource | Releases activity-category datasource state and its owned rows. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd420` | `ClientActivityPortalSectionDatasource::~ClientActivityPortalSectionDatasource` | ClientActivityPortalSectionDatasource | Releases portal-section datasource state and owned rows. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd4f0` | `ClientActivityDatasource::ClientActivityDatasource` | ClientActivityDatasource | Constructs the shared activity data-source base state. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd570` | `ClientActivityDatasource::ScalarDeletingDtor` | ClientActivityDatasource | Implements the class's scalar deleting destructor entry. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd590` | `ClientScheduledActivityDatasource::ClientScheduledActivityDatasource` | ClientScheduledActivityDatasource | Constructs scheduled-activity datasource storage. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd610` | `ClientScheduledActivityDatasource::ScalarDeletingDtor` | ClientScheduledActivityDatasource | Implements the class's scalar deleting destructor entry. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd630` | `ClientActivityCategoryDatasource::ClientActivityCategoryDatasource` | ClientActivityCategoryDatasource | Constructs the activity-category reference data source. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd6c0` | `ClientActivityCategoryDatasource::ScalarDeletingDtor` | ClientActivityCategoryDatasource | Implements the class's scalar deleting destructor entry. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd6e0` | `ClientActivityPortalSectionDatasource::ClientActivityPortalSectionDatasource` | ClientActivityPortalSectionDatasource | Constructs the portal-section reference data source. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cfd760` | `ClientActivityPortalSectionDatasource::ScalarDeletingDtor` | ClientActivityPortalSectionDatasource | Implements the class's scalar deleting destructor entry. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cfbd10` | `CleanupActivityDatasourceRows ? → ClientActivityDatasource::CleanupRows` |  | Cleans up an activity datasource, based on its deleting-destructor caller. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfbd70` | `CleanupScheduledDatasourceRows ? → ClientScheduledActivityDatasource::CleanupRo…` |  | Cleans up a scheduled activity datasource, based on its deleting-destructor caller. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfbda0` | `ClientScheduledActivityDatasource::RefreshRows ?` | ClientScheduledActivityDatasource | Rebuilds the visible schedule rows in sorted order. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfc0b0` | `FormatActivityCategoryPath ?` |  | Builds the datasource path for an activity category. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

