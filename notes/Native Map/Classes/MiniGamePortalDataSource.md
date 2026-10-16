# Class: MiniGamePortalDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c477c0` | `MiniGamePortalDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the minigame-portal table: 17 names - TypeId, TypeName, TypeDescription, MiniGameGroupId, IconId, BackgroundIconId, IsMembersOnly, IsMicro, LockStatus, IsActive, CategoryId, CategoryName, HasPlayed… |
| `00c47910` | `MiniGamePortalDataSource_GetRowCount` | 🟢 | virtual | Virtual slot 2: when the populated byte at +0x22c is clear it first runs the population helper 00c478b0, then tail-jumps to the base implementation FUN_010c8f30 (row count of the base table). |
| `00c47930` | `MiniGamePortalDataSource_GetValue` | ✅ | virtual | Value getter of the portal table: ensures the rows are populated, rejects row >= base row count or column >= 0x11, resolves the row object (row id from the base source, then FUN_009acf80) and serves 17 columns - TypeId … |
| `00c47c90` | `— ⚠reject` | ⚪ | ctor | Base constructor body of the portal data source: installs the three MiniGamePortalDataSource vtable pointers and runs the base data-source constructor FUN_0091bf60; it neither registers a data-source name nor initialise… |
| `00c47d00` | `MiniGamePortalDataSource_Construct` | ✅ | ctor | Constructor of the portal data source: registers it under the name BaseClient.MiniGamePortal (FUN_00922890), installs the three MiniGamePortalDataSource vtable pointers, stores the two arguments at +0x254/+0x258 and cle… |
| `00c47d90` | `MiniGamePortalDataSource_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the portal data source destructor 00c47c90 and frees the object with _free when the low bit of the flag argument is set. |
