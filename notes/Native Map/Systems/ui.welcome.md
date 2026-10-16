# System: ui.welcome

42 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ace460` | `GetWhatsNewColumnName` | WelcomeInfo::WhatsNewDataSource | Supplies column labels for the What's New table. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace4a0` | `GetInfoColumnName` | WelcomeInfo::InfoDataSource | Supplies the two column labels for the Welcome Info table. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace4f0` | `GetClaimCodeColumnName` | WelcomeInfo::ClaimCodeDataSource | Supplies the Welcome Info claim-code table's column labels. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace560` | `GetLatestContentColumnName` | WelcomeInfo::LatestContentDataSource | Supplies the Latest Content table's column labels. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace920` | `GetField230` | WelcomeInfo::WhatsNewDataSource | Getter on the What's New data source interface. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace930` | `GetField230` | WelcomeInfo::ClaimCodeDataSource | Getter on the claim-code data source interface. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace940` | `GetField230` | WelcomeInfo::LatestContentDataSource | Getter on the latest-content data source interface. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf550` | `LoadWeightedWelcomeTips` |  | Selects a weighted welcome tip for the Info data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf890` | `GetLatestContentCellValue` | WelcomeInfo::LatestContentDataSource | Supplies a Latest Content cell value to the UI data-source interface. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acfc20` | `GetInfoCellValue` | WelcomeInfo::InfoDataSource | Provides treasure-ticket count and text values to the Welcome Info UI table. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0100` | `ParseWelcomeText` |  | Builds local Welcome screen table content from the welcome resource. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0a70` | `WelcomeInfo::WhatsNewDataSource::Ctor` | WelcomeInfo::WhatsNewDataSource | Constructs the What's New table data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0b10` | `WelcomeInfo::WhatsNewDataSource::Dtor` | WelcomeInfo::WhatsNewDataSource | Cleans up the What's New table data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0ba0` | `WelcomeInfo::WhatsNewDataSource::ScalarDeletingDtor` | WelcomeInfo::WhatsNewDataSource | Scalar deleting destructor in the data-source vtable. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0bc0` | `WelcomeInfo::InfoDataSource::Ctor` | WelcomeInfo::InfoDataSource | Constructs the Welcome Info data source and its local tip data. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0c70` | `WelcomeInfo::InfoDataSource::Dtor` | WelcomeInfo::InfoDataSource | Cleans the Welcome Info data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0d00` | `WelcomeInfo::InfoDataSource::ScalarDeletingDtor` | WelcomeInfo::InfoDataSource | Scalar deleting destructor in the data-source vtable. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0d20` | `WelcomeInfo::ClaimCodeDataSource::Ctor` | WelcomeInfo::ClaimCodeDataSource | Constructs the Welcome Claim Codes data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0db0` | `WelcomeInfo::ClaimCodeDataSource::Dtor` | WelcomeInfo::ClaimCodeDataSource | Cleans the Welcome Claim Codes data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0e40` | `WelcomeInfo::ClaimCodeDataSource::ScalarDeletingDtor` | WelcomeInfo::ClaimCodeDataSource | Scalar deleting destructor in the data-source vtable. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0e60` | `WelcomeInfo::LatestContentDataSource::Ctor` | WelcomeInfo::LatestContentDataSource | Constructs the Welcome Latest Content data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0ef0` | `WelcomeInfo::LatestContentDataSource::Dtor` | WelcomeInfo::LatestContentDataSource | Cleans the Welcome Latest Content data source. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0f80` | `WelcomeInfo::LatestContentDataSource::ScalarDeletingDtor` | WelcomeInfo::LatestContentDataSource | Scalar deleting destructor in the data-source vtable. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |

## 🟢 likely (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00acf2c0` | `DestroyWideStringEntryRange` |  | Cleans removed What's New entries. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf3a0` | `ReallocateWhatsNewEntries` |  | Resizes the parsed What's New entry array. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf470` | `DestroyWideAndNarrowStringRange` |  | Removes claim-code records from the Welcome data-source array. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf970` | `EraseWhatsNewEntries` |  | Compacts the What's New entries after a duplicate or replaced row is found. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acfa90` | `EnsureWhatsNewEntryAtIndex` |  | Ensures storage for a What's New row at the requested index. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acfb50` | `ReallocateClaimCodeEntries` |  | Resizes the Claim Codes data-source array. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acfe80` | `AppendWhatsNewEntry` |  | Appends a row to the WelcomeInfo data-source array. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acff50` | `EnsureDefaultWhatsNewEntry` |  | Creates a default What's New row when the welcome parser needs one. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad04f0` | `AppendClaimCodeSourceEntry` |  | Appends a row to the Claim Codes data-source array. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0720` | `InsertWhatsNewEntries` |  | Creates space for new Welcome data-source entries. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0790` | `ApplyWelcomeScreenPayload` |  | Applies parsed Welcome-screen content to its data sources. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0a60` | `AdjustorThunk_00ad0880` |  | Adjusts the receiver before forwarding to the Message of the Day handler. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0fa0` | `CreateWelcomeDataSources` |  | Creates the What's New, Info, Claim Codes, and Latest Content data-source objects. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00acf280` | `— ⚠reject` | WelcomeInfo::WhatsNewDataSource | Clears one Subject or Body cell in the What's New table. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf300` | `— ⚠reject` |  | Creates a What's New row during array growth or parsing. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acf4b0` | `— ⚠reject` |  | Constructs a claim-code row from source data. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acfcf0` | `— ⚠reject` | WelcomeInfo::ClaimCodeDataSource | Provides claim-code row values to the Welcome UI table. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0580` | `— ⚠reject` |  | Maintains claim-code rows when the Welcome data-source changes. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad0880` | `— ⚠reject` |  | Maintains the Message of the Day row in WelcomeInfo's local content table. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |

