# System: ui.bugreport

30 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f8950` | `UiBugManager::BugCategoryDataSource::ColumnName` | UiBugManager::BugCategoryDataSource | Provides the category column header for the bug-report data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f8980` | `UiBugManager::BugSubCategoryDataSource::ColumnName` | UiBugManager::BugSubCategoryDataSource | Provides the subcategory column header for the bug-report data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f89b0` | `UiBugManager::BugSeverityDataSource::ColumnName` | UiBugManager::BugSeverityDataSource | Provides the severity column header for the bug-report data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f89c0` | `AdjustThisPlus4AndJump010C8F30` | UiBugManager::BugSeverityDataSource | Adjusts the receiver for the shared virtual implementation; that target's behavior is not assigned here. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f93f0` | `UiBugManager::BugCategoryDataSource::GetValue` | UiBugManager::BugCategoryDataSource | Supplies a category data-source field; exact column meaning is not established by this body. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f9400` | `UiBugManager::BugCategoryDataSource::SetValue` | UiBugManager::BugCategoryDataSource | Applies a changed category data-source value. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f9490` | `UiBugManager::BugSubCategoryDataSource::GetValue` | UiBugManager::BugSubCategoryDataSource | Supplies the selected bug subcategory value. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f94e0` | `UiBugManager::BugSubCategoryDataSource::SetValue` | UiBugManager::BugSubCategoryDataSource | Applies a changed bug subcategory value. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f9530` | `UiBugManager::BugSeverityDataSource::SetValue` | UiBugManager::BugSeverityDataSource | Applies a changed severity data-source value. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fa720` | `AppendRelatedIdsForKey` |  | Formats identifiers associated with a selected bug-report row. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fbb60` | `LoadBugReportTables` |  | Loads local bug-report category, mapping, and severity resources into the manager's linked/hash tables and refreshes the associated views. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc390` | `UiBugManager::BugCategoryDataSource::Dtor` | UiBugManager::BugCategoryDataSource | Cleans up the category data source and its embedded GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc3f0` | `UiBugManager::BugSubCategoryDataSource::Dtor` | UiBugManager::BugSubCategoryDataSource | Cleans up the subcategory data source and its embedded GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc450` | `UiBugManager::BugSeverityDataSource::Dtor` | UiBugManager::BugSeverityDataSource | Cleans up the severity data source and its embedded GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc4c0` | `UiBugManager::BugCategoryDataSource::Ctor` | UiBugManager::BugCategoryDataSource | Initializes the bug category data source and its GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc540` | `UiBugManager::BugCategoryDataSource::ScalarDeletingDtor` | UiBugManager::BugCategoryDataSource | Scalar deleting destructor for the category data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc560` | `UiBugManager::BugSubCategoryDataSource::Ctor` | UiBugManager::BugSubCategoryDataSource | Initializes the bug subcategory data source and its GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc5e0` | `UiBugManager::BugSubCategoryDataSource::ScalarDeletingDtor` | UiBugManager::BugSubCategoryDataSource | Scalar deleting destructor for the subcategory data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc600` | `UiBugManager::BugSeverityDataSource::Ctor` | UiBugManager::BugSeverityDataSource | Initializes the bug severity data source and its GUI table base. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc680` | `UiBugManager::BugSeverityDataSource::ScalarDeletingDtor` | UiBugManager::BugSeverityDataSource | Scalar deleting destructor for the severity data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00c06140` | `RegisterBugSubmissionModule` |  | Registers the client 'BugSubmission' Lua module with its two functions OnSelectedCategory and OnSubmitBug (the /bug report window). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fc150` | `ReleaseThreeDataSourcesAndMaps` |  | Cleanup body called by scalar-deleting wrapper 008a5f00; this range does not establish a standalone manager RTTI class. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc9c0` | `SubmitBugReport` |  | Submits the bug-report form through the client's gateway path. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fcc30` | `NotifyBugReportSubmitted ⚠downgrade` |  | Completes the client-side bug-report form flow and shows its confirmation. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f9450` | `PopulateBugCategoryRows ?` |  | Refreshes rows for one bug-report data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f95b0` | `PopulateBugSeverityRows ?` |  | Refreshes severity rows for a bug-report data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009fa350` | `— ⚠reject` |  | Updates the bug-report severity selection and refreshes its data source. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fa950` | `— ⚠reject` |  | Initializes the bug-report form from its GUI controls and data sources. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009facb0` | `— ⚠reject` |  | Builds diagnostic text for bug category and severity identifiers. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009fc6a0` | `— ⚠reject` |  | Initializes the bug-report data-source component; a standalone manager RTTI class is not established in this range. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |

